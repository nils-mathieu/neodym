use neodym_sys_common::SysResult;

pub extern "C" fn terminate(process: usize, _: usize, _: usize) -> SysResult {
    nd_log::trace!("system call: terminate({:#x})", process);
    todo!("implement the `terminate` system call");
}
