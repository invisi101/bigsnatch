mod dns_cache;
mod dns_parser;
mod ebpf_loader;
mod event_processor;
mod grpc_server;
mod process_cache;

use std::sync::Arc;
use std::time::Instant;

use anyhow::{bail, Context, Result};
use tokio::sync::broadcast;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use crate::dns_cache::DnsCache;
use crate::grpc_server::MonitorService;
use crate::process_cache::ProcessCache;

pub mod proto {
    tonic::include_proto!("snitchster");
}

const SOCKET_PATH: &str = "/run/snitchster.sock";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("snitchster=info".parse()?))
        .init();

    if !nix::unistd::Uid::effective().is_root() {
        bail!("snitchster-daemon must run as root (required for eBPF)");
    }

    info!("Starting snitchster daemon");

    let start_time = Instant::now();
    let process_cache = Arc::new(ProcessCache::new());
    let dns_cache = Arc::new(DnsCache::new(10000));
    let (event_tx, _) = broadcast::channel::<proto::ConnectionEvent>(4096);

    let mut ebpf = ebpf_loader::load_and_attach()
        .context("Failed to load eBPF programs")?;

    info!("eBPF programs loaded and attached");

    let proc_cache = process_cache.clone();
    let d_cache = dns_cache.clone();
    let tx = event_tx.clone();
    let _event_handle = tokio::spawn(async move {
        if let Err(e) = event_processor::run(&mut ebpf, proc_cache, d_cache, tx).await {
            error!("Event processor error: {}", e);
        }
    });

    let service = MonitorService::new(
        event_tx.clone(),
        process_cache.clone(),
        dns_cache.clone(),
        start_time,
    );
    let _grpc_handle = tokio::spawn(async move {
        if let Err(e) = grpc_server::serve(service, SOCKET_PATH).await {
            error!("gRPC server error: {}", e);
        }
    });

    info!("Daemon ready, listening on {}", SOCKET_PATH);

    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");

    Ok(())
}
