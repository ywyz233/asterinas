//! Read the Cpu context content then dispatch syscall to corrsponding handler
//! The each sub module contains functions that handle real syscall logic.
use crate::prelude::*;
use crate::syscall::access::sys_access;
use crate::syscall::arch_prctl::sys_arch_prctl;
use crate::syscall::brk::sys_brk;
use crate::syscall::clone::sys_clone;
use crate::syscall::close::sys_close;
use crate::syscall::execve::sys_execve;
use crate::syscall::exit::sys_exit;
use crate::syscall::exit_group::sys_exit_group;
use crate::syscall::fcntl::sys_fcntl;
use crate::syscall::fork::sys_fork;
use crate::syscall::fstat::sys_fstat;
use crate::syscall::futex::sys_futex;
use crate::syscall::getcwd::sys_getcwd;
use crate::syscall::getegid::sys_getegid;
use crate::syscall::geteuid::sys_geteuid;
use crate::syscall::getgid::sys_getgid;
use crate::syscall::getpgrp::sys_getpgrp;
use crate::syscall::getpid::sys_getpid;
use crate::syscall::getppid::sys_getppid;
use crate::syscall::gettid::sys_gettid;
use crate::syscall::getuid::sys_getuid;
use crate::syscall::ioctl::sys_ioctl;
use crate::syscall::kill::sys_kill;
use crate::syscall::lseek::sys_lseek;
use crate::syscall::lstat::sys_lstat;
use crate::syscall::mmap::sys_mmap;
use crate::syscall::mprotect::sys_mprotect;
use crate::syscall::munmap::sys_munmap;
use crate::syscall::openat::sys_openat;
use crate::syscall::poll::sys_poll;
use crate::syscall::prctl::sys_prctl;
use crate::syscall::read::sys_read;
use crate::syscall::readlink::sys_readlink;
use crate::syscall::rt_sigaction::sys_rt_sigaction;
use crate::syscall::rt_sigprocmask::sys_rt_sigprocmask;
use crate::syscall::rt_sigreturn::sys_rt_sigreturn;
use crate::syscall::sched_yield::sys_sched_yield;
use crate::syscall::setpgid::sys_setpgid;
use crate::syscall::tgkill::sys_tgkill;
use crate::syscall::uname::sys_uname;
use crate::syscall::wait4::sys_wait4;
use crate::syscall::waitid::sys_waitid;
use crate::syscall::write::sys_write;
use crate::syscall::writev::sys_writev;
use jinux_frame::cpu::CpuContext;

mod access;
mod arch_prctl;
mod brk;
mod clone;
mod close;
mod constants;
mod execve;
mod exit;
mod exit_group;
mod fcntl;
mod fork;
mod fstat;
mod futex;
mod getcwd;
mod getegid;
mod geteuid;
mod getgid;
mod getpgrp;
mod getpid;
mod getppid;
mod gettid;
mod getuid;
mod ioctl;
mod kill;
mod lseek;
mod lstat;
mod mmap;
mod mprotect;
mod munmap;
mod openat;
mod poll;
mod prctl;
mod read;
mod readlink;
mod rt_sigaction;
mod rt_sigprocmask;
mod rt_sigreturn;
mod sched_yield;
mod setpgid;
mod tgkill;
mod uname;
mod wait4;
mod waitid;
mod write;
mod writev;

macro_rules! define_syscall_nums {
    ( $( $name: ident = $num: expr ),+ ) => {
        $(
            const $name: u64  = $num;
        )*
    }
}

/// This macro is used to define syscall handler.
/// The first param is ths number of parameters,
/// The second param is the function name of syscall handler,
/// The third is optional, means the args(if parameter number > 0),
/// The third is optional, means if cpu context is required.
macro_rules! syscall_handler {
    (0, $fn_name: ident) => { $fn_name() };
    (0, $fn_name: ident, $context: expr) => { $fn_name($context) };
    (1, $fn_name: ident, $args: ident) => { $fn_name($args[0] as _) };
    (1, $fn_name: ident, $args: ident, $context: expr) => { $fn_name($args[0] as _, $context) };
    (2, $fn_name: ident, $args: ident) => { $fn_name($args[0] as _, $args[1] as _)};
    (2, $fn_name: ident, $args: ident, $context: expr) => { $fn_name($args[0] as _, $args[1] as _, $context)};
    (3, $fn_name: ident, $args: ident) => { $fn_name($args[0] as _, $args[1] as _, $args[2] as _)};
    (3, $fn_name: ident, $args: ident, $context: expr) => { $fn_name($args[0] as _, $args[1] as _, $args[2] as _, $context)};
    (4, $fn_name: ident, $args: ident) => { $fn_name($args[0] as _, $args[1] as _, $args[2] as _, $args[3] as _)};
    (4, $fn_name: ident, $args: ident, $context: expr) => { $fn_name($args[0] as _, $args[1] as _, $args[2] as _, $args[3] as _), $context};
    (5, $fn_name: ident, $args: ident) => { $fn_name($args[0] as _, $args[1] as _, $args[2] as _, $args[3] as _, $args[4] as _)};
    (5, $fn_name: ident, $args: ident, $context: expr) => { $fn_name($args[0] as _, $args[1] as _, $args[2] as _, $args[3] as _, $args[4] as _, $context)};
    (6, $fn_name: ident, $args: ident) => { $fn_name($args[0] as _, $args[1] as _, $args[2] as _, $args[3] as _, $args[4] as _, $args[5] as _)};
    (6, $fn_name: ident, $args: ident, $context: expr) => { $fn_name($args[0] as _, $args[1] as _, $args[2] as _, $args[3] as _, $args[4] as _, $args[5] as _, $context)};
}

define_syscall_nums!(
    SYS_READ = 0,
    SYS_WRITE = 1,
    SYS_CLOSE = 3,
    SYS_FSTAT = 5,
    SYS_LSTAT = 6,
    SYS_POLL = 7,
    SYS_LSEEK = 8,
    SYS_MMAP = 9,
    SYS_MPROTECT = 10,
    SYS_MUNMAP = 11,
    SYS_BRK = 12,
    SYS_RT_SIGACTION = 13,
    SYS_RT_SIGPROCMASK = 14,
    SYS_RT_SIGRETRUN = 15,
    SYS_IOCTL = 16,
    SYS_WRITEV = 20,
    SYS_ACCESS = 21,
    SYS_SCHED_YIELD = 24,
    SYS_GETPID = 39,
    SYS_CLONE = 56,
    SYS_FORK = 57,
    SYS_EXECVE = 59,
    SYS_EXIT = 60,
    SYS_WAIT4 = 61,
    SYS_KILL = 62,
    SYS_UNAME = 63,
    SYS_FCNTL = 72,
    SYS_GETCWD = 79,
    SYS_READLINK = 89,
    SYS_GETUID = 102,
    SYS_GETGID = 104,
    SYS_GETEUID = 107,
    SYS_GETEGID = 108,
    SYS_SETPGID = 109,
    SYS_GETPPID = 110,
    SYS_GETPGRP = 111,
    SYS_PRCTL = 157,
    SYS_ARCH_PRCTL = 158,
    SYS_GETTID = 186,
    SYS_FUTEX = 202,
    SYS_EXIT_GROUP = 231,
    SYS_TGKILL = 234,
    SYS_WAITID = 247,
    SYS_OPENAT = 257
);

pub struct SyscallArgument {
    syscall_number: u64,
    args: [u64; 6],
}

/// Syscall return
#[derive(Debug, Clone, Copy)]
pub enum SyscallReturn {
    /// return isize, this value will be used to set rax
    Return(isize),
    /// does not need to set rax
    NoReturn,
}

impl SyscallArgument {
    fn new_from_context(context: &CpuContext) -> Self {
        let syscall_number = context.gp_regs.rax;
        let mut args = [0u64; 6];
        args[0] = context.gp_regs.rdi;
        args[1] = context.gp_regs.rsi;
        args[2] = context.gp_regs.rdx;
        args[3] = context.gp_regs.r10;
        args[4] = context.gp_regs.r8;
        args[5] = context.gp_regs.r9;
        Self {
            syscall_number,
            args,
        }
    }
}

pub fn handle_syscall(context: &mut CpuContext) {
    let syscall_frame = SyscallArgument::new_from_context(context);
    let syscall_return =
        syscall_dispatch(syscall_frame.syscall_number, syscall_frame.args, context);

    match syscall_return {
        Ok(return_value) => {
            debug!("syscall return: {:?}", return_value);
            if let SyscallReturn::Return(return_value) = return_value {
                context.gp_regs.rax = return_value as u64;
            }
        }
        Err(err) => {
            debug!("syscall return error: {:?}", err);
            let errno = err.error() as i32;
            context.gp_regs.rax = (-errno) as u64
        }
    }
}

pub fn syscall_dispatch(
    syscall_number: u64,
    args: [u64; 6],
    context: &mut CpuContext,
) -> Result<SyscallReturn> {
    match syscall_number {
        SYS_READ => syscall_handler!(3, sys_read, args),
        SYS_WRITE => syscall_handler!(3, sys_write, args),
        SYS_CLOSE => syscall_handler!(1, sys_close, args),
        SYS_FSTAT => syscall_handler!(2, sys_fstat, args),
        SYS_LSTAT => syscall_handler!(2, sys_lstat, args),
        SYS_POLL => syscall_handler!(3, sys_poll, args),
        SYS_LSEEK => syscall_handler!(3, sys_lseek, args),
        SYS_MMAP => syscall_handler!(6, sys_mmap, args),
        SYS_MPROTECT => syscall_handler!(3, sys_mprotect, args),
        SYS_MUNMAP => syscall_handler!(2, sys_munmap, args),
        SYS_BRK => syscall_handler!(1, sys_brk, args),
        SYS_RT_SIGACTION => syscall_handler!(4, sys_rt_sigaction, args),
        SYS_RT_SIGPROCMASK => syscall_handler!(4, sys_rt_sigprocmask, args),
        SYS_RT_SIGRETRUN => syscall_handler!(0, sys_rt_sigreturn, context),
        SYS_IOCTL => syscall_handler!(3, sys_ioctl, args),
        SYS_WRITEV => syscall_handler!(3, sys_writev, args),
        SYS_ACCESS => syscall_handler!(2, sys_access, args),
        SYS_SCHED_YIELD => syscall_handler!(0, sys_sched_yield),
        SYS_GETPID => syscall_handler!(0, sys_getpid),
        SYS_CLONE => syscall_handler!(5, sys_clone, args, context.clone()),
        SYS_FORK => syscall_handler!(0, sys_fork, context.clone()),
        SYS_EXECVE => syscall_handler!(3, sys_execve, args, context),
        SYS_EXIT => syscall_handler!(1, sys_exit, args),
        SYS_WAIT4 => syscall_handler!(3, sys_wait4, args),
        SYS_KILL => syscall_handler!(2, sys_kill, args),
        SYS_UNAME => syscall_handler!(1, sys_uname, args),
        SYS_FCNTL => syscall_handler!(3, sys_fcntl, args),
        SYS_GETCWD => syscall_handler!(2, sys_getcwd, args),
        SYS_READLINK => syscall_handler!(3, sys_readlink, args),
        SYS_GETUID => syscall_handler!(0, sys_getuid),
        SYS_GETGID => syscall_handler!(0, sys_getgid),
        SYS_GETEUID => syscall_handler!(0, sys_geteuid),
        SYS_GETEGID => syscall_handler!(0, sys_getegid),
        SYS_SETPGID => syscall_handler!(2, sys_setpgid, args),
        SYS_GETPPID => syscall_handler!(0, sys_getppid),
        SYS_GETPGRP => syscall_handler!(0, sys_getpgrp),
        SYS_PRCTL => syscall_handler!(5, sys_prctl, args),
        SYS_ARCH_PRCTL => syscall_handler!(2, sys_arch_prctl, args, context),
        SYS_GETTID => syscall_handler!(0, sys_gettid),
        SYS_FUTEX => syscall_handler!(6, sys_futex, args),
        SYS_EXIT_GROUP => syscall_handler!(1, sys_exit_group, args),
        SYS_TGKILL => syscall_handler!(3, sys_tgkill, args),
        SYS_WAITID => syscall_handler!(5, sys_waitid, args),
        SYS_OPENAT => syscall_handler!(4, sys_openat, args),
        _ => panic!("Unsupported syscall number: {}", syscall_number),
    }
}

#[macro_export]
macro_rules! log_syscall_entry {
    ($syscall_name: tt) => {
        let syscall_name_str = stringify!($syscall_name);
        info!("[SYSCALL][id={}][{}]", $syscall_name, syscall_name_str);
    };
}