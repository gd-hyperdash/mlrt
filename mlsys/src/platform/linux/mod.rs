#[cfg(target_os = "android")]
pub mod android;
pub mod mappings;

// Includes

use crate::core::*;

pub use nix::errno::*;
pub use nix::sys::mman::*;
pub use nix::sys::signal::*;
pub use nix::unistd::*;

pub use nix::libc::{
    c_int, c_void, dladdr, dlclose, dlerror, dlopen, dlsym, getpid, readlink, siginfo_t,
    ucontext_t, Dl_info, PATH_MAX, RTLD_GLOBAL, RTLD_LAZY, RTLD_NOLOAD,
};

#[cfg(not(target_os = "android"))]
pub use nix::libc::{dlinfo, RTLD_DI_LINKMAP};

#[cfg(target_arch = "x86_64")]
pub use nix::libc::REG_RIP;

#[cfg(target_arch = "x86")]
pub use nix::libc::REG_EIP;

// Types

#[repr(C)]
pub struct link_map {
    pub l_addr: *const nix::libc::c_void,
    pub l_name: *const u8,
}

pub struct LinuxSigHandler {
    pub old_handler: SigAction,
}

// Helpers

extern "C" {
    fn cxx_flush_cache(beg: *mut nix::libc::c_void, end: *mut nix::libc::c_void);
}

// LinuxSigHandler

impl LinuxSigHandler {
    pub fn new(handler: &SigAction) -> Self {
        unsafe {
            match sigaction(Signal::SIGILL, handler) {
                Ok(h) => LinuxSigHandler { old_handler: h },
                Err(_) => panic!("Could not initialize signal handler!"),
            }
        }
    }
}

impl Drop for LinuxSigHandler {
    fn drop(&mut self) {
        unsafe {
            if sigaction(Signal::SIGILL, &self.old_handler).is_err() {
                panic!("Could not cleanup signal handler!");
            }
        }
    }
}

// Linux

pub fn get_empty_dlinfo() -> Dl_info {
    Dl_info {
        dli_fname: std::ptr::null(),
        dli_fbase: std::ptr::null_mut(),
        dli_sname: std::ptr::null(),
        dli_saddr: std::ptr::null_mut(),
    }
}

// TODO: complete.
pub unsafe fn wrap_system_error(error: Errno) -> Error {
    match error {
        _ => panic!("System call failed with error: {:?}", error),
    }
}

pub unsafe fn get_system_error() -> Errno {
    nix::errno::from_i32(errno())
}

pub unsafe fn get_system_error_casted() -> u32 {
    errno() as u32
}

pub unsafe fn get_system_error_wrapped() -> Error {
    wrap_system_error(get_system_error())
}

pub unsafe fn cacheflush(address: Address, size: usize) {
    cxx_flush_cache(address as _, address.add(size) as _)
}
