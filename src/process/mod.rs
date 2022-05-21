// Platform

#[cfg(target_os = "windows")]
mod win;

#[cfg(target_os = "windows")]
pub use self::win::*;

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub use self::linux::*;

// Includes

use std::ffi::CStr;
use std::path::{Path, PathBuf};
use std::os::raw::c_char;

// Proc

pub unsafe fn load_module(path: &Path, mid: &str) -> mlsys::Result<mlsys::Handle> {
    match self::load_module_internal(path) {
        Ok(h) => {
            ldr::initialize_ml_binary(self::get_module_base(h)?, path, h, mid)?;
            Ok(h)
        }
        Err(e) => Err(e),
    }
}

pub unsafe fn free_module(h: mlsys::Handle) -> mlsys::Result<()> {
    let base = self::get_module_base(h)?;
    let path = self::get_module_path(h)?;
    self::free_module_internal(h)?;

    if self::get_module_base(h).unwrap_or(0) != base {
        return ldr::cleanup_ml_binary(base, &path, h);
    }

    Ok(())
}

// Bindings

#[no_mangle]
unsafe extern "C" fn MLProcId() -> u32 {
    self::get_id()
}

#[no_mangle]
unsafe extern "C" fn MLProcHandle() -> mlsys::Handle {
    self::get_handle()
}

#[no_mangle]
unsafe extern "C" fn MLProcAddLinkerPath(p: *const mlsys::RawString) -> mlsys::Error {
    if let Some(&path) = p.as_ref() {
        return match self::add_linker_path(&PathBuf::from(path)) {
            Ok(()) => mlsys::Error::Success,
            Err(e) => e,
        };
    }

    mlsys::Error::InvalidArgument
}

#[no_mangle]
unsafe extern "C" fn MLProcRemoveLinkerPath(p: *const mlsys::RawString) -> mlsys::Error {
    if let Some(&path) = p.as_ref() {
        return match self::remove_linker_path(&PathBuf::from(path)) {
            Ok(()) => mlsys::Error::Success,
            Err(e) => e,
        };
    }

    mlsys::Error::InvalidArgument
}

#[no_mangle]
unsafe extern "C" fn MLProcGetModule(
    s: *const mlsys::RawString,
    out: *mut mlsys::Handle,
) -> mlsys::Error {
    if let Some(&name) = s.as_ref() {
        if !out.is_null() {
            return match self::get_module(&String::from(name)) {
                Ok(h) => {
                    *out = h;
                    mlsys::Error::Success
                }
                Err(e) => e,
            };
        }
    }

    mlsys::Error::InvalidArgument
}

#[no_mangle]
unsafe extern "C" fn MLProcGetModuleFromAddress(
    p: mlsys::Address,
    out: *mut mlsys::Handle,
) -> mlsys::Error {
    if !out.is_null() {
        return match self::get_module_from_address(p) {
            Ok(h) => {
                *out = h;
                mlsys::Error::Success
            }
            Err(e) => e,
        };
    }

    mlsys::Error::InvalidArgument
}

#[no_mangle]
unsafe extern "C" fn MLProcLoadModule(
    p: *const mlsys::RawString,
    mid: *const c_char,
    out: *mut mlsys::Handle,
) -> mlsys::Error {
    let mid = if mid.is_null() {
        String::new()
    } else {
        match CStr::from_ptr(mid).to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return mlsys::Error::InvalidArgument,
        }
    };

    if let Some(&path) = p.as_ref() {
        if !out.is_null() {
            return match self::load_module(&PathBuf::from(path), &mid) {
                Ok(h) => {
                    *out = h;
                    mlsys::Error::Success
                }
                Err(e) => e,
            };
        }
    }

    mlsys::Error::InvalidArgument
}

#[no_mangle]
unsafe extern "C" fn MLProcFreeModule(h: mlsys::Handle) -> mlsys::Error {
    match self::free_module(h) {
        Ok(()) => mlsys::Error::Success,
        Err(e) => e,
    }
}

#[no_mangle]
unsafe extern "C" fn MLProcGetModulePath(
    h: mlsys::Handle,
    out: *mut mlsys::RawString,
) -> mlsys::Error {
    if !out.is_null() {
        return match self::get_module_path(h) {
            Ok(p) => {
                *out = mlsys::RawString::from(p.as_os_str());
                mlsys::Error::Success
            }
            Err(e) => e,
        };
    }

    mlsys::Error::InvalidArgument
}

#[no_mangle]
unsafe extern "C" fn MLProcGetModuleBase(
    h: mlsys::Handle,
    out: *mut mlsys::Address,
) -> mlsys::Error {
    if !out.is_null() {
        return match self::get_module_base(h) {
            Ok(base) => {
                *out = base;
                mlsys::Error::Success
            }
            Err(e) => e,
        };
    }

    mlsys::Error::InvalidArgument
}

#[no_mangle]
unsafe extern "C" fn MLProcGetModuleSymbolAddress(
    h: mlsys::Handle,
    s: *const c_char,
    out: *mut mlsys::Address,
) -> mlsys::Error {
    if !s.is_null() {
        if let Ok(sym) = CStr::from_ptr(s).to_str() {
            if !out.is_null() {
                return match self::get_module_symbol_address(h, sym) {
                    Ok(address) => {
                        *out = address;
                        mlsys::Error::Success
                    }
                    Err(e) => e,
                };
            }
        }
    }

    mlsys::Error::InvalidArgument
}
