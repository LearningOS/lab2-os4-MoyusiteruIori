//! Process management syscalls

use crate::config::{MAX_SYSCALL_NUM, PAGE_SIZE};
use crate::mm::{VirtAddr, get_allocatable_ppn, MapPermission};
use crate::task::{exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, TASK_MANAGER};
use crate::timer::get_time_us;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

#[derive(Clone, Copy)]
pub struct TaskInfo {
    pub status: TaskStatus,
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    pub time: usize,
}

pub fn sys_exit(exit_code: i32) -> ! {
    info!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

// YOUR JOB: 引入虚地址后重写 sys_get_time
// _ts 传进来的地址是基于发起 syscall 的进程的地址空间的
// 所以需要进行从内核空间到那个进程空间的转换，找到真正的 ts
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    let _us = get_time_us();
    let ts = TASK_MANAGER.translate_in_current_task(VirtAddr::from(_ts as usize));
    unsafe {
        *(ts as *mut TimeVal) = TimeVal {
            sec: _us / 1_000_000,
            usec: _us % 1_000_000,
        };
    }
    0
}

// CLUE: 从 ch4 开始不再对调度算法进行测试~
pub fn sys_set_priority(_prio: isize) -> isize {
    -1
}

// YOUR JOB: 扩展内核以实现 sys_mmap 和 sys_munmap
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    let start = VirtAddr::from(_start);
    let end = VirtAddr::from(_start + _len);
    let perm = MapPermission::from_bits((_port << 1) as u8 | 0x10).unwrap();
    if !start.aligned() {
        return -1;
    }
    if (_port & !0x7 != 0) || (_port & 0x7 == 0) {
        return -1;
    }
    let mut frame_num = _len / PAGE_SIZE;
    if _len % PAGE_SIZE != 0 {
        frame_num += 1;
    }
    if frame_num > get_allocatable_ppn() {
        println!("can't alloc!");
        return -1;
    }
    TASK_MANAGER.current_task_mmap(start, end, perm)
}

pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    if !VirtAddr::from(_start).aligned() {
        -1
    }
    else {
        TASK_MANAGER.current_task_munmap(
            VirtAddr::from(_start),
            VirtAddr::from(_start + _len)
        )
    }
}

// YOUR JOB: 引入虚地址后重写 sys_task_info
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    let ti = TASK_MANAGER.translate_in_current_task(VirtAddr::from(ti as usize)) as *mut TaskInfo;
    unsafe {
        *ti = TaskInfo {
            status: TaskStatus::Running,
            syscall_times: TASK_MANAGER.get_current_task_syscall_times(),
            time: (get_time_us() - TASK_MANAGER.get_current_task_start_time()) / 1000
        }
    }
    0
}
