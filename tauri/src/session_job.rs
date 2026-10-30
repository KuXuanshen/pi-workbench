//! Windows 下让终端及其子进程随会话一起结束。
use std::io;
use windows_sys::Win32::{
    Foundation::CloseHandle,
    System::JobObjects::{AssignProcessToJobObject, CreateJobObjectW, SetInformationJobObject,
        JobObjectExtendedLimitInformation, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE},
};

// 保存为整数以便在异步任务中移动；句柄只由此对象拥有。
pub struct SessionJob(usize);
impl SessionJob {
    pub fn new() -> io::Result<Self> {
        unsafe {
            let handle = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if handle.is_null() { return Err(io::Error::last_os_error()); }
            let job = Self(handle as usize);
            let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            if SetInformationJobObject(handle, JobObjectExtendedLimitInformation,
                &limits as *const _ as _, std::mem::size_of_val(&limits) as u32) == 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(job)
        }
    }
    pub fn attach(&self, child: &dyn portable_pty::Child) -> io::Result<()> {
        let process = child.as_raw_handle().ok_or_else(|| io::Error::other("missing process handle"))?;
        if unsafe { AssignProcessToJobObject(self.0 as _, process as _) } == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
}
impl Drop for SessionJob {
    fn drop(&mut self) { unsafe { CloseHandle(self.0 as _); } }
}
