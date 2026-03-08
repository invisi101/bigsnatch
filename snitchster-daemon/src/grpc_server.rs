use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use tokio::net::UnixListener;
use tokio::sync::broadcast;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

use crate::dns_cache::DnsCache;
use crate::process_cache::ProcessCache;
use crate::proto;
use crate::proto::monitor_server::{Monitor, MonitorServer};

/// Shared state for tracking connected subscribers
pub struct ClientTracker {
    pub subscriber_count: AtomicUsize,
    pub had_subscriber: AtomicBool,
}

impl ClientTracker {
    pub fn new() -> Self {
        Self {
            subscriber_count: AtomicUsize::new(0),
            had_subscriber: AtomicBool::new(false),
        }
    }
}

pub struct MonitorService {
    event_tx: broadcast::Sender<proto::ConnectionEvent>,
    process_cache: Arc<ProcessCache>,
    dns_cache: Arc<DnsCache>,
    start_time: Instant,
    tracker: Arc<ClientTracker>,
}

impl MonitorService {
    pub fn new(
        event_tx: broadcast::Sender<proto::ConnectionEvent>,
        process_cache: Arc<ProcessCache>,
        dns_cache: Arc<DnsCache>,
        start_time: Instant,
        tracker: Arc<ClientTracker>,
    ) -> Self {
        Self {
            event_tx,
            process_cache,
            dns_cache,
            start_time,
            tracker,
        }
    }
}

type EventStream = std::pin::Pin<
    Box<dyn tokio_stream::Stream<Item = Result<proto::ServerEvent, Status>> + Send>,
>;

#[tonic::async_trait]
impl Monitor for MonitorService {
    type SubscribeStream = EventStream;

    async fn subscribe(
        &self,
        _request: Request<proto::SubscribeRequest>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        let mut rx = self.event_tx.subscribe();
        let tracker = self.tracker.clone();

        // Track this subscriber
        tracker.subscriber_count.fetch_add(1, Ordering::SeqCst);
        tracker.had_subscriber.store(true, Ordering::SeqCst);
        let count = tracker.subscriber_count.load(Ordering::SeqCst);
        info!("GUI client connected ({} active)", count);

        let stream = async_stream::stream! {
            loop {
                match rx.recv().await {
                    Ok(conn) => {
                        yield Ok(proto::ServerEvent {
                            event: Some(proto::server_event::Event::Connection(conn)),
                        });
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("GUI client lagged, skipped {} events", n);
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }

            // Subscriber disconnected
            let remaining = tracker.subscriber_count.fetch_sub(1, Ordering::SeqCst) - 1;
            info!("GUI client disconnected ({} remaining)", remaining);
        };

        Ok(Response::new(Box::pin(stream)))
    }

    async fn get_connections(
        &self,
        _request: Request<proto::GetConnectionsRequest>,
    ) -> Result<Response<proto::GetConnectionsResponse>, Status> {
        Ok(Response::new(proto::GetConnectionsResponse {
            connections: vec![],
            total: 0,
        }))
    }

    async fn get_status(
        &self,
        _request: Request<proto::Empty>,
    ) -> Result<Response<proto::DaemonStatus>, Status> {
        let status = proto::DaemonStatus {
            total_connections: 0,
            active_processes: self.process_cache.active_count() as u64,
            dns_cache_entries: self.dns_cache.len() as u64,
            uptime_seconds: self.start_time.elapsed().as_secs(),
            events_per_second: 0.0,
            ebpf_loaded: true,
        };
        Ok(Response::new(status))
    }
}

pub async fn serve(service: MonitorService, socket_path: &str) -> Result<()> {
    let _ = std::fs::remove_file(socket_path);

    let listener = UnixListener::bind(socket_path)?;

    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(socket_path, std::fs::Permissions::from_mode(0o666))?;

    info!("gRPC server listening on unix://{}", socket_path);

    let incoming = UnixListenerStream::new(listener);

    Server::builder()
        .add_service(MonitorServer::new(service))
        .serve_with_incoming(incoming)
        .await?;

    Ok(())
}
