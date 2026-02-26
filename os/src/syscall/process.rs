//! Process management syscalls
use crate::{
    config::PAGE_SIZE, mm::{PageTable, PhysAddr, VirtAddr, translated_byte_buffer}, task::{
        change_program_brk, current_user_token, exit_current_and_run_next, get_current_task_syscall_count, mmap_current, munmap_current, suspend_current_and_run_next
    }, timer::get_time_us
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let sec = us / 1_000_000;
    let usec = us % 1_000_000;

    let buffers = translated_byte_buffer(
        current_user_token(), 
        ts as *const u8, 
        core::mem::size_of::<TimeVal>(),
    );

    let mut kernel_bytes = [0u8; 16];
    let sec_bytes = sec.to_ne_bytes();
    let usec_bytes = usec.to_ne_bytes();
    kernel_bytes[..8].copy_from_slice(&sec_bytes);
    kernel_bytes[8..].copy_from_slice(&usec_bytes);

    let mut kernel_iter = kernel_bytes.iter();
    for buffer in buffers {
        for byte in buffer {
            if let Some(k_byte) = kernel_iter.next() {
                *byte = *k_byte;
            } else {
                break;
            }
        }
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
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        0 => {
            let va = VirtAddr::from(id);
            let vpn = VirtAddr::from(id).floor();
            let pte = PageTable::from_token(current_user_token()).translate(vpn);
            if let Some(pte) = pte {
                if pte.is_valid() && pte.readable() && pte.is_user() {
                    let ptr = (PhysAddr::from(pte.ppn()).0 + va.page_offset()) as *const u8;
                    unsafe { return (*ptr) as isize; }
                }
            }
            -1
        }
        1 => {
            let va = VirtAddr::from(id);
            let vpn = VirtAddr::from(id).floor();
            let pte = PageTable::from_token(current_user_token()).translate(vpn);
            if let Some(pte) = pte {
                if pte.is_valid() && pte.writable() && pte.is_user() {
                    let ptr = (PhysAddr::from(pte.ppn()).0 + va.page_offset()) as *const u8;
                    unsafe { *(ptr as *mut u8) = data as u8; }
                    return 0;
                }
            }
            -1
        }
        2 => {
            let syscall_id = id;
            let count = get_current_task_syscall_count(syscall_id);
            count as isize
        }
        _ => -1,
    }
}

/// map a memory region in user space, and return 0 if success.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap");
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    if port & !0x7 != 0 || port & 0x7 == 0 {
        return -1;
    }
    mmap_current(start, len, port)
}

/// unmap a memory region in user space, and return 0 if success.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if start % PAGE_SIZE != 0 {
        return -1;
    }
    munmap_current(start, len)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
