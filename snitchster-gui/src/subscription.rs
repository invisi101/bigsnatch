use iced::futures::SinkExt;
use iced::Subscription;
use tokio::time::{sleep, Duration};
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;
use tracing::{debug, error, info};

use crate::message::Message;
use crate::proto;
use crate::proto::monitor_client::MonitorClient;

const SOCKET_PATH: &str = "/run/snitchster.sock";
const RECONNECT_DELAY: Duration = Duration::from_secs(2);

pub fn daemon_events() -> Subscription<Message> {
    Subscription::run(connect_and_stream)
}

fn connect_and_stream() -> impl iced::futures::Stream<Item = Message> {
    iced::stream::channel(256, |mut output: iced::futures::channel::mpsc::Sender<Message>| async move {
        loop {
            info!("Connecting to daemon at {}", SOCKET_PATH);

            // Convert connection result to a simpler type before any awaits
            let connection_result = connect_to_daemon().await
                .map_err(|e| e.to_string());

            match connection_result {
                Ok(mut client) => {
                    let _ = output.send(Message::DaemonConnected).await;
                    info!("Connected to daemon");

                    let request = proto::SubscribeRequest { filter: None };
                    match client.subscribe(request).await {
                        Ok(response) => {
                            let mut stream = response.into_inner();
                            loop {
                                match stream.message().await {
                                    Ok(Some(event)) => {
                                        if let Some(proto::server_event::Event::Connection(conn)) =
                                            event.event
                                        {
                                            if output
                                                .send(Message::ConnectionReceived(conn))
                                                .await
                                                .is_err()
                                            {
                                                return;
                                            }
                                        }
                                    }
                                    Ok(None) => {
                                        info!("Daemon stream ended");
                                        break;
                                    }
                                    Err(e) => {
                                        error!("Stream error: {}", e);
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!("Subscribe failed: {}", e);
                        }
                    }

                    let _ = output
                        .send(Message::DaemonDisconnected("Stream ended".into()))
                        .await;
                }
                Err(msg) => {
                    debug!("{}", msg);
                    let _ = output
                        .send(Message::DaemonDisconnected(msg))
                        .await;
                }
            }

            sleep(RECONNECT_DELAY).await;
        }
    })
}

async fn connect_to_daemon() -> Result<MonitorClient<Channel>, Box<dyn std::error::Error + Send + Sync>> {
    let channel = Endpoint::try_from("http://[::]:50051")?
        .connect_with_connector(service_fn(|_: Uri| async {
            let stream = tokio::net::UnixStream::connect(SOCKET_PATH).await?;
            Ok::<_, std::io::Error>(hyper_util::rt::TokioIo::new(stream))
        }))
        .await?;

    Ok(MonitorClient::new(channel))
}
