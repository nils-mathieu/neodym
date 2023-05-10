use neodym_sys_common::SysResult;

pub extern "C" fn ring0(data: usize, f: usize, _: usize) -> SysResult {
    // SAFETY:
    //  This transmutation is unsafe. Too bad x)
    inner(data as *mut (), unsafe { core::mem::transmute(f) })
}

#[inline(always)]
fn inner(data: *mut (), f: extern "C" fn(data: *mut ())) -> SysResult {
    nd_log::trace!("system call: ring0({:#p}, {:#p})", data, f);
    f(data);
    SysResult(0)
}
