use neodym_sys_common::SysResult;

pub extern "C" fn get_memory(segments: usize, size: usize, _: usize) -> SysResult {
    nd_log::trace!("system call: get_memory({:#x}, {:#x})", segments, size);
    todo!();
}

pub extern "C" fn map_memory(entries: usize, count: usize, _: usize) -> SysResult {
    nd_log::trace!("system call: map_memory({:#x}, {:#x})", entries, count);
    todo!();
}
