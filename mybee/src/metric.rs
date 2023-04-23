use std::time::Duration;

use color_eyre::Result;
use metrics::{describe_histogram, register_histogram};
use metrics_exporter_prometheus::PrometheusBuilder;
use metrics_util::MetricKindMask;

pub const HISTOGRAM_LATENCY: &str = "latency";
pub const FIELD_QUERY: &str = "query";
pub const FIELD_CLIENT_IP: &str = "client_ip";

pub async fn install() -> Result<()> {
    let builder = PrometheusBuilder::new().idle_timeout(
        MetricKindMask::ALL,
        Some(Duration::from_secs(10 * 24 * 60 * 60)), // 10 days
    );
    let handle = builder.install_recorder()?;

    register_histogram!(HISTOGRAM_LATENCY);
    describe_histogram!(HISTOGRAM_LATENCY, metrics::Unit::Seconds, "query latency");

    // periodically calls `handle.render()` to compact the memory
    tokio::spawn(async move {
        let mut timer = tokio::time::interval(Duration::from_secs(5));
        loop {
            timer.tick().await;
            let _ = handle.render();
        }
    });

    log::info!("Prometheus http exporter at http://:9000");
    Ok(())
}
