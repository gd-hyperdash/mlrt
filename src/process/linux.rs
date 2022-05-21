// Includes

use mlsys::*;

use std::ffi::OsString;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};

// Helpers

unsafe fn get_main_module_path(h: Handle) -> Result<PathBuf> {
    let mut buffer = [0u8; platform::PATH_MAX as usize];

    // https://stackoverflow.com/a/8876887
    let size = platform::readlink(
        "/proc/self/exe".as_ptr() as _,
        buffer.as_mut_ptr() as _,
        platform::PATH_MAX as _,
    );

    if size != -1 {
        return Ok(PathBuf::from(OsString::from_vec(buffer.to_vec())));
    }

    // Fallback.
    if let Some(m) = platform::mappings::from_address(get_module_base(h)?) {
        if let Some(path) = m.path {
            return Ok(path);
        }
    }

    Err(platform::get_system_error_wrapped())
}

#[cfg(target_os = "android")]
unsafe fn get_other_module_path(h: Handle) -> Result<PathBuf> {
    platform::android::get_android_other_module_path(h)
}

#[cfg(not(target_os = "android"))]
unsafe fn get_other_module_path(h: Handle) -> Result<PathBuf> {
    let mut info: *mut platform::link_map = std::ptr::null_mut();

    platform::dlinfo(
        h as _,
        platform::RTLD_DI_LINKMAP,
        &mut info as *mut _ as *mut _,
    );

    if !info.is_null() {
        let path = (*info).l_name;
        if !path.is_null() && (*path != '\0' as _) {
            let size = {
                let mut c = 0usize;

                while *(path.offset(c as isize)) != '\0' as _ {
                    c += 1;
                }

                c
            };
            return Ok(PathBuf::from(OsString::from_vec(
                std::slice::from_raw_parts(path, size).to_vec(),
            )));
        }
    }

    Err(platform::get_system_error_wrapped())
}

// Proc

pub unsafe fn get_id() -> u32 {
    platform::getpid() as u32
}

pub unsafe fn get_handle() -> Handle {
    platform::dlopen(
        std::ptr::null_mut(),
        platform::RTLD_LAZY | platform::RTLD_NOLOAD,
    ) as Handle
}

#[cfg(target_os = "android")]
pub unsafe fn add_linker_path(p: &Path) -> Result<()> {
    platform::android::add_android_linker_path(p)
}

#[cfg(target_os = "android")]
pub unsafe fn remove_linker_path(p: &Path) -> Result<()> {
    platform::android::remove_android_linker_path(p)
}

#[cfg(not(target_os = "android"))]
pub unsafe fn add_linker_path(_p: &Path) -> Result<()> {
    panic!("Unimplemented!") // TODO
}

#[cfg(not(target_os = "android"))]
pub unsafe fn remove_linker_path(_p: &Path) -> Result<()> {
    panic!("Unimplemented!") // TODO
}

pub unsafe fn get_module(s: &str) -> Result<Handle> {
    let m = platform::dlopen(
        s.as_bytes().as_ptr() as _,
        platform::RTLD_LAZY | platform::RTLD_NOLOAD,
    );

    if !m.is_null() {
        return Ok(m as Handle);
    }

    Err(platform::get_system_error_wrapped())
}

pub unsafe fn get_module_from_address(address: Address) -> Result<Handle> {
    let mut info = platform::get_empty_dlinfo();

    if platform::dladdr(address as _, &mut info) != 0 && !info.dli_fname.is_null() {
        let m = platform::dlopen(info.dli_fname, platform::RTLD_LAZY | platform::RTLD_NOLOAD);

        if !m.is_null() {
            return Ok(m as Handle);
        }
    }

    Err(platform::get_system_error_wrapped())
}

pub unsafe fn load_module_internal(p: &Path) -> Result<Handle> {
    let m = platform::dlopen(
        p.as_os_str().as_bytes().as_ptr() as _,
        platform::RTLD_LAZY | platform::RTLD_GLOBAL,
    );

    if !m.is_null() {
        return Ok(m as Handle);
    }

    Err(platform::get_system_error_wrapped())
}

pub unsafe fn free_module_internal(h: Handle) -> Result<()> {
    if h == get_handle() {
        return Err(mlsys::Error::InvalidArgument);
    }

    if platform::dlclose(h as _) == 0 {
        return Ok(());
    }

    Err(platform::get_system_error_wrapped())
}

pub unsafe fn get_module_path(h: Handle) -> Result<PathBuf> {
    match h == get_handle() {
        true => get_main_module_path(h),
        false => get_other_module_path(h),
    }
}

#[cfg(target_os = "android")]
pub unsafe fn get_module_base(h: Handle) -> Result<Address> {
    platform::android::get_android_module_base(h)
}

#[cfg(not(target_os = "android"))]
pub unsafe fn get_module_base(h: Handle) -> Result<Address> {
    let mut info: *mut platform::link_map = std::ptr::null_mut();

    platform::dlinfo(
        h as _,
        platform::RTLD_DI_LINKMAP,
        &mut info as *mut _ as *mut _,
    );

    if !info.is_null() {
        return Ok((*info).l_addr as _);
    }

    Err(platform::get_system_error_wrapped())
}

pub unsafe fn get_module_symbol_address(h: mlsys::Handle, sym: &str) -> Result<Address> {
    platform::dlerror();
    let address = platform::dlsym(h as _, sym.as_bytes().as_ptr() as _);

    if platform::dlerror().is_null() {
        return Ok(address as Address);
    }

    Err(platform::get_system_error_wrapped())
}
