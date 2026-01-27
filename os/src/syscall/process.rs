//! Process management syscalls
use crate::{
    task::{exit_current_and_run_next, suspend_current_and_run_next, get_current_task_syscall_count},
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

/// Trace current task's system call counter, and do corresponding modifications.
/// This syscall has 3 different behaviors based on the [`trace_request`] argument:
/// 
/// - `trace_request == 0`: ignore `data`, consider `id` as `* const u8`, read 1 byte from it and return
/// - `trace_request == 1`: consider `id` as `* mut u8`, write `data` to it and return 0
/// - `trace_request == 2`: query the syscall count tracker (including this call) of syscall `id`, 
/// ignore `data`, and return the count
/// - otherwise: return -1
/// 
/// No safety check needed now.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    match _trace_request {
        0 => {
            let ptr = _id as *const u8;
            unsafe { (*ptr) as isize }
        }
        1 => {
            let ptr = _id as *mut u8;
            let value = _data as u8;
            unsafe { *ptr = value; }
            0
        }
        2 => {
            let syscall_id = _id;
            let count = get_current_task_syscall_count(syscall_id);
            count as isize
        }
        _ => -1,
    }
}
