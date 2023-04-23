#![no_std]
#![no_main]

use aya_bpf::{
    cty::c_char,
    helpers::{
        bpf_get_current_pid_tgid, bpf_ktime_get_ns, bpf_probe_read_user, bpf_probe_read_user_buf,
    },
    macros::{map, uprobe, uretprobe},
    maps::{HashMap, PerCpuArray, PerfEventByteArray},
    programs::ProbeContext,
};
use mybee_common::{QueryEnd, QueryStart, MAX_STATEMENT_LEN};

const COM_QUERY: u32 = 3;

struct ComData {
    query:  *const c_char,
    length: u32,
}

#[map]
pub static mut SCRATCH: PerCpuArray<QueryStart> = PerCpuArray::with_max_entries(1, 0); // per-cpu
#[map]
pub static mut QUERY_RING: PerfEventByteArray = PerfEventByteArray::new(0);
#[map]
pub static mut TRACK_MAP: HashMap<u64, u64> = HashMap::with_max_entries(128, 0);
#[map]
pub static mut IP_MAP: HashMap<u64, [u8; 16]> = HashMap::with_max_entries(128, 0);

#[uprobe(name = "uprobe_dispatch_command")]
pub fn uprobe_dispatch_command(ctx: ProbeContext) -> u32 {
    match try_uprobe(ctx) {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

#[uretprobe(name = "uretprobe_dispatch_command")]
pub fn uretprobe_dispatch_command(ctx: ProbeContext) -> u32 {
    match try_uretprobe(ctx) {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

#[uprobe(name = "uprobe_assign_ip")]
pub fn uprobe_assign_ip(ctx: ProbeContext) -> u32 {
    fn imp(ctx: ProbeContext) -> Result<u32, u32> {
        let tid = bpf_get_current_pid_tgid();
        let ip_ptr = ctx.arg::<*const c_char>(1).ok_or(1_u32)?; // max 15 chars + null
        let ip_len = ctx.arg::<i32>(2).ok_or(1_u32)?.max(0).min(15);

        let mut ip = [0_u8; 16];
        ip[0] = ip_len as u8;
        unsafe {
            bpf_probe_read_user_buf(ip_ptr as *const u8, &mut ip[1..ip_len as usize + 1])
                .map_err(|_| 1u32)?;

            IP_MAP.insert(&tid, &ip, 0).map_err(|_| 1u32)?;
        }
        Ok(0)
    }

    match imp(ctx) {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

#[uprobe(name = "uprobe_end_connection")]
pub fn uprobe_end_connection(ctx: ProbeContext) -> u32 {
    fn imp(_ctx: ProbeContext) -> Result<u32, u32> {
        let tid = bpf_get_current_pid_tgid();
        unsafe {
            IP_MAP.remove(&tid).map_err(|_| 1u32)?;
        }
        Ok(0)
    }

    match imp(ctx) {
        Ok(ret) => ret,
        Err(ret) => ret,
    }
}

fn try_uprobe(ctx: ProbeContext) -> Result<u32, u32> {
    let command = ctx.arg::<u32>(2).ok_or(1_u32)?;
    if command == COM_QUERY {
        let com_data: *const ComData = ctx.arg(1).ok_or(1_u32)?;
        let query: *const c_char = unsafe {
            bpf_probe_read_user::<*const c_char>(&(*com_data).query as *const *const c_char)
        }
        .map_err(|_| 1u32)?;
        let query_len: usize =
            unsafe { bpf_probe_read_user::<u32>(&(*com_data).length as *const u32) }
                .map_err(|_| 1u32)? as usize;
        let query_len = query_len.min(MAX_STATEMENT_LEN);

        if let Some(buf_ptr) = unsafe { SCRATCH.get_ptr_mut(0) } {
            unsafe {
                let buf: &mut QueryStart = &mut *buf_ptr;
                bpf_probe_read_user_buf(query as *const u8, &mut buf.buf[..query_len])
                    .map_err(|_| 1u32)?;
                buf.tid = bpf_get_current_pid_tgid();
                buf.start_time = bpf_ktime_get_ns(); // record start end_time
                buf.kind = 1;
                buf.query_len = query_len as u64;
                TRACK_MAP
                    .insert(&buf.tid, &buf.start_time, 0)
                    .map_err(|_| 1u32)?;
                QUERY_RING.output(&ctx, buf.as_bytes(), 0);
            }
        }
    }

    Ok(0)
}

fn try_uretprobe(ctx: ProbeContext) -> Result<u32, u32> {
    let tid = bpf_get_current_pid_tgid();

    if let Some(&start_time) = unsafe { TRACK_MAP.get(&tid) } {
        unsafe {
            TRACK_MAP.remove(&tid).map_err(|_| 1u32)?;
        }
        let data = QueryEnd {
            tid,
            duration: unsafe { bpf_ktime_get_ns() } - start_time,
            kind: 2,
            start_time,
        };
        unsafe {
            QUERY_RING.output(
                &ctx,
                core::slice::from_raw_parts(
                    &data as *const QueryEnd as *const u8,
                    core::mem::size_of::<QueryEnd>(),
                ),
                0,
            );
        }
    }

    Ok(0)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}
