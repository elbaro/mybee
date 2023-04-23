#![feature(addr_parse_ascii)]
pub mod ebpf;
pub mod metric;

use aya::maps::perf::AsyncPerfEventArray;
use aya::maps::HashMap;

use bytes::BytesMut;
use clap::Parser;
use color_eyre::eyre::ContextCompat;
use color_eyre::Result;
use dashmap::DashMap;
use log::info;
use metrics::histogram;
use mybee_common::{QueryEnd, QueryStart};
use sql_redactor::MySqlDialect;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;

use crate::metric::*;

#[derive(Debug, Parser)]
pub struct Opts {
    #[clap(short, long)]
    pid: Option<i32>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    color_eyre::install()?;
    metric::install().await?;
    let opt = Opts::parse();

    let mut bpf = ebpf::attach_bpf(&opt)?;
    let mut query_ring: AsyncPerfEventArray<_> = bpf
        .take_map("QUERY_RING")
        .context("failed to map QUERY_RING")?
        .try_into()?;
    let ip_map: Arc<HashMap<_, u64, [u8; 16]>> = Arc::new(HashMap::try_from(
        bpf.take_map("IP_MAP").context("failed to map IP_MAP")?,
    )?);

    let pending_queries: Arc<DashMap<(u64, u64), String>> = Arc::new(DashMap::new());

    /*
     * cpu1 -> ring -\
     * cpu2 -> ring --> mpsc -> map[stmt].record(1)
     * cpu3 -> ring -/
     *
     * cpu1 -> ring -> dashmap[sql1].record()
     * cpu2 -> ring -> dashmap[sql2].record()
     * cpu3 -> ring -> dashmap[sql3].record()
     */
    for cpu_id in aya::util::online_cpus()? {
        let ip_map = ip_map.clone();
        let mut query_buf = query_ring.open(cpu_id, None)?;
        let pending_queries = pending_queries.clone();

        tokio::spawn(async move {
            let mut buffers = (0..10)
                .map(|_| BytesMut::with_capacity(core::mem::size_of::<QueryStart>()))
                .collect::<Vec<_>>();

            loop {
                let events = query_buf.read_events(&mut buffers).await.unwrap();

                #[allow(clippy::needless_range_loop)]
                for i in 0..events.read {
                    let buffer = &mut buffers[i];

                    // QueryStart or QueryEnd
                    let kind = unsafe { (*(buffer.as_ptr() as *const QueryStart)).kind };
                    match kind {
                        1 => {
                            let event: &QueryStart =
                                unsafe { &*(buffer.as_ptr() as *const QueryStart) };

                            if let Ok(redacted) = sql_redactor::redact(
                                &MySqlDialect {},
                                core::str::from_utf8(&event.buf[..event.query_len as usize])
                                    .unwrap(),
                            ) {
                                let key = (event.tid, event.start_time);
                                // TODO: sync with TRACK_MAP in eBPF
                                pending_queries.insert(key, redacted);
                            }
                        }
                        2 => {
                            let event: &QueryEnd =
                                unsafe { &*(buffer.as_ptr() as *const QueryEnd) };
                            if let Some((_key, sql)) =
                                pending_queries.remove(&(event.tid, event.start_time))
                            {
                                let client_ip = ip_map
                                    .get(&event.tid, 0)
                                    .ok()
                                    .and_then(|ip_slice| {
                                        let len = ip_slice[0] as usize;
                                        std::net::Ipv4Addr::parse_ascii(&ip_slice[1..len + 1]).ok()
                                    })
                                    .unwrap_or_else(|| std::net::Ipv4Addr::new(0, 0, 0, 0));

                                histogram!(HISTOGRAM_LATENCY, Duration::from_nanos(event.duration), FIELD_QUERY => sql, FIELD_CLIENT_IP => client_ip.to_string());
                            }
                        }
                        _ => {
                            unimplemented!()
                        }
                    }
                }
            }
        });
    }

    info!("Waiting for Ctrl-C...");
    signal::ctrl_c().await?;
    info!("Exiting...");

    Ok(())
}
