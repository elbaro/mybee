#![no_std]

pub const MAX_STATEMENT_LEN: usize = 8192 - 100; // reserve 100 for other fields

#[repr(C)]
pub struct QueryStart {
    pub tid: u64,
    pub start_time: u64,
    pub kind: u64,
    pub query_len: u64,
    pub buf: [u8; MAX_STATEMENT_LEN],
}

impl QueryStart {
    pub fn as_bytes(&self) -> &[u8] {
        let query_len = (self.query_len as usize).min(MAX_STATEMENT_LEN);
        unsafe {
            core::slice::from_raw_parts(
                self as *const Self as *const u8,
                core::mem::size_of::<u64>() * 4 + query_len,
            )
        }
    }
}

#[repr(C)]
pub struct QueryEnd {
    pub tid: u64,
    pub duration: u64,
    pub kind: u64,
    pub start_time: u64,
}
