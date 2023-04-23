use aya::programs::UProbe;
use aya::{include_bytes_aligned, Bpf};
use aya_log::BpfLogger;
use color_eyre::Result;
use sysinfo::{PidExt, ProcessExt, SystemExt};

use crate::Opts;

pub fn attach_bpf(opts: &Opts) -> Result<Bpf> {
    #[cfg(debug_assertions)]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/debug/mybee"
    ))?;
    #[cfg(not(debug_assertions))]
    let mut bpf = Bpf::load(include_bytes_aligned!(
        "../../target/bpfel-unknown-none/release/mybee"
    ))?;
    if let Err(e) = BpfLogger::init(&mut bpf) {
        // This can happen if you remove all log statements from your eBPF program.
        log::warn!("failed to initialize eBPF logger: {}", e);
    }

    let pid = {
        if let Some(pid) = opts.pid {
            pid
        } else {
            let mut system = sysinfo::System::new();
            system.refresh_all();
            let procs: Vec<_> = system.processes_by_name("mysqld").collect();
            if procs.len() != 1 {
                log::warn!("There is 0 more at least 2 mysqld processes");
                for p in procs {
                    log::warn!("{} {}", p.pid(), p.name());
                }
                return Err(color_eyre::eyre::eyre!(
                    "There is 0 more at least 2 mysqld processes"
                ));
            }
            procs[0].pid().as_u32() as i32
        }
    };
    let target = format!("/proc/{}/exe", pid);

    let program: &mut UProbe = bpf
        .program_mut("uprobe_dispatch_command")
        .unwrap()
        .try_into()?;
    program.load()?;
    program.attach(
        Some("_Z16dispatch_commandP3THDPK8COM_DATA19enum_server_command"),
        0,
        &target,
        opts.pid,
    )?;

    let program: &mut UProbe = bpf
        .program_mut("uretprobe_dispatch_command")
        .unwrap()
        .try_into()?;
    program.load()?;
    program.attach(
        Some("_Z16dispatch_commandP3THDPK8COM_DATA19enum_server_command"),
        0,
        &target,
        opts.pid,
    )?;

    let program: &mut UProbe = bpf.program_mut("uprobe_assign_ip").unwrap().try_into()?;
    program.load()?;
    program.attach(
        Some("_ZN16Security_context9assign_ipEPKci"),
        0,
        &target,
        opts.pid,
    )?;

    let program: &mut UProbe = bpf
        .program_mut("uprobe_end_connection")
        .unwrap()
        .try_into()?;
    program.load()?;
    program.attach(Some("_Z14end_connectionP3THD"), 0, &target, opts.pid)?;

    Ok(bpf)
}
