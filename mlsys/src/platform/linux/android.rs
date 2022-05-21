// Includes

use crate::core::*;

use goblin::elf::*;
use lazy_static::*;
use nix::libc::*;

use std::ffi::OsString;
use std::fs::File;
use std::io::Read;
use std::os::unix::ffi::OsStringExt;
use std::path::{Path, PathBuf};

// Types

// int property_get(const char *key, char *value, const char *default_value);
type PropertyGetT = fn(*const c_char, *mut c_char, *const c_char) -> c_int;
// int __system_property_get(const char *name, char *value);
type SystemPropertyGetT = fn(*const c_char, *mut c_char) -> c_int;
// char const* get_soname(void* h);
type GetSoNameT = fn(*mut c_void) -> *const c_char;
//void __loader_android_get_LD_LIBRARY_PATH(char* buffer, size_t buffer_size);
type GetLDPATHT = fn(*mut c_char, size_t);
// void __loader_android_update_LD_LIBRARY_PATH(const char* ld_library_path);
type UpdateLDPATHT = fn(*const c_char);

#[repr(C)]
struct soinfo_t {
    name: [u8; 128],
    phdr: Address,
    phnum: usize,
    entry: Address,
    base: Address,
}

#[repr(C)]
struct sohandle_t {
    phdr: Address,
    phnum: usize,
    base: Address,
}

struct LinkerData {
    map_ptr: Option<Address>,
    soname_ptr: Option<Address>,
}

struct LDPATHData {
    get_ld_library_path: Option<GetLDPATHT>,
    update_ld_library_path: Option<UpdateLDPATHT>,
}

// Globals

const PROP_VALUE_MAX: usize = 92;
const PROP_KEY: &[u8] = b"ro.build.version.sdk\0";
const LD_PATHS_SIZE: usize = (PATH_MAX as usize) * 16;

const ANDROID_API_5: usize = 21;
const ANDROID_API_7: usize = 24;
const ANDROID_API_7_1: usize = 25;
const ANDROID_API_9: usize = 28;
const ANDROID_API_10: usize = 29;

lazy_static! {
    static ref API_LEVEL: usize = unsafe { init_api_level() };
    static ref LINKER_PATH_CSTR: &'static [u8] = init_linker_path();
    static ref LINKER_PATH: PathBuf = PathBuf::from(OsString::from_vec(LINKER_PATH_CSTR.to_vec()));
    static ref LINKER_DATA: LinkerData = init_linker_data();
    static ref LD_PATH_DATA: LDPATHData = unsafe { init_ldpath_data() };
}

// Helpers

extern "C" {
    fn cxx_get_android_module_handle(map: *const c_void, h: *const c_void) -> *mut c_void;
}

fn get_sym_offset(buffer: &[u8], name: &str) -> Option<usize> {
    let binary = Elf::parse(&buffer).unwrap();
    let syms = binary.syms;
    let strtab = binary.strtab;

    if !syms.is_empty() {
        for sym in syms.iter() {
            let sym_name = strtab.get_at(sym.st_name).unwrap();

            if let Some(_x) = sym_name.find(name) {
                return Some(sym.st_value as _);
            }
        }
    }

    None
}

unsafe fn get_linker_handle() -> Result<*mut c_void> {
    let linker = dlopen(LINKER_PATH_CSTR.as_ptr() as _, RTLD_LAZY | RTLD_NOLOAD);

    if !linker.is_null() {
        return Ok(linker);
    }

    Err(Error::ItemNotFound)
}

/// Get the API level of the device.
unsafe fn init_api_level() -> usize {
    let buffer = [0u8; PROP_VALUE_MAX];

    let libutils = dlopen(b"libcutils.so\0".as_ptr() as _, RTLD_LAZY | RTLD_NOLOAD);

    if !libutils.is_null() {
        let property_get_ptr = dlsym(libutils, b"property_get\0".as_ptr() as _);
        if !property_get_ptr.is_null() {
            let property_get = std::mem::transmute::<*mut c_void, PropertyGetT>(property_get_ptr);
            property_get(
                PROP_KEY.as_ptr() as _,
                buffer.as_ptr() as _,
                b"0\0".as_ptr() as _,
            );
        }
    }

    if buffer[0] == 0 {
        let libc = dlopen(b"libc.so\0".as_ptr() as _, RTLD_LAZY | RTLD_NOLOAD);

        if !libc.is_null() {
            let __system_property_get_ptr = dlsym(libc, b"__system_property_get\0".as_ptr() as _);
            if !__system_property_get_ptr.is_null() {
                let __system_property_get = std::mem::transmute::<*mut c_void, SystemPropertyGetT>(
                    __system_property_get_ptr,
                );
                __system_property_get(PROP_KEY.as_ptr() as _, buffer.as_ptr() as _);
            }
        }
    }

    if let Ok(s) = std::str::from_utf8(&buffer) {
        if let Ok(version) = usize::from_str_radix(s, 10) {
            return version;
        };
    }

    panic!("OS version parsing failed.")
}

#[cfg(target_pointer_width = "64")]
/// Find the correct linker path (64 bit).
fn init_linker_path() -> &'static [u8] {
    if *API_LEVEL >= ANDROID_API_10 {
        return b"/apex/com.android.runtime/bin/linker64\0";
    }

    b"/system/bin/linker64\0"
}

#[cfg(target_pointer_width = "32")]
/// Find the correct linker path (32 bit).
fn init_linker_path() -> &'static [u8] {
    if *API_LEVEL >= ANDROID_API_10 {
        return b"/apex/com.android.runtime/bin/linker\0";
    }

    b"/system/bin/linker\0"
}

/// Init linker data.
fn init_linker_data() -> LinkerData {
    let base = super::mappings::from_path(&*LINKER_PATH).unwrap().base;
    let mut file = File::open(&*LINKER_PATH).unwrap();
    let len = file.metadata().unwrap().len();
    let mut buffer = vec![0; len as usize];
    file.read_exact(&mut buffer[..len as usize]).unwrap();

    LinkerData {
        map_ptr: match get_sym_offset(&buffer, "g_soinfo_handles_map") {
            Some(x) => Some(base + x),
            None => None,
        },
        soname_ptr: match get_sym_offset(&buffer, "_ZNK6soinfo10get_sonameEv") {
            Some(x) => Some(base + x),
            None => None,
        },
    }
}

/// Init ldpath data.
unsafe fn init_ldpath_data() -> LDPATHData {
    let mut data = LDPATHData {
        get_ld_library_path: None,
        update_ld_library_path: None,
    };

    if *API_LEVEL >= ANDROID_API_9 {
        let linker = get_linker_handle().unwrap();
        let get_ptr = dlsym(
            linker,
            b"__loader_android_get_LD_LIBRARY_PATH\0".as_ptr() as _,
        );

        let update_ptr = dlsym(
            linker,
            b"__loader_android_update_LD_LIBRARY_PATH\0".as_ptr() as _,
        );

        if get_ptr.is_null() || update_ptr.is_null() {
            panic!("Linker metadata not found.");
        }

        data.get_ld_library_path = Some(std::mem::transmute::<*mut c_void, GetLDPATHT>(get_ptr));
        data.update_ld_library_path = Some(std::mem::transmute::<*mut c_void, UpdateLDPATHT>(
            update_ptr,
        ));
    } else if *API_LEVEL >= ANDROID_API_5 {
        let base = super::mappings::from_path(&*LINKER_PATH).unwrap().base;
        let mut file = File::open(&*LINKER_PATH).unwrap();
        let len = file.metadata().unwrap().len();
        let mut buffer = vec![0; len as usize];
        file.read_exact(&mut buffer[..len as usize]).unwrap();

        data.get_ld_library_path = Some(std::mem::transmute::<Address, GetLDPATHT>(
            base + get_sym_offset(&buffer, "android_get_LD_LIBRARY_PATHPcj").unwrap(),
        ));

        data.update_ld_library_path = Some(std::mem::transmute::<Address, UpdateLDPATHT>(
            base + get_sym_offset(&buffer, "android_update_LD_LIBRARY_PATHPKc").unwrap(),
        ));
    } else {
        panic!("This version of android lacks the required metadata for handling LD_LIBRARY_PATH.");
    }

    data
}

/// Android 7 lacks the symbol for "get_soname".
/// we need a hack.
unsafe fn soname_hack(h: *const sohandle_t) -> Vec<u8> {
    let p = match super::mappings::from_address((*h).base) {
        Some(_) => ((h as Address) + 0x178) as *const u8,
        None => h as *const u8,
    };

    std::slice::from_raw_parts(p, strlen(p as _)).to_vec()
}

/// Get the path from a handle.
/// Android 7 and above.
unsafe fn get_module_path_new(h: Handle) -> Result<Vec<u8>> {
    if h & 1 == 0 {
        return get_module_path_old(h);
    }

    let handle = cxx_get_android_module_handle(LINKER_DATA.map_ptr.unwrap() as _, h as _);

    if !handle.is_null() {
        if *API_LEVEL > ANDROID_API_7_1 {
            let get_soname =
                std::mem::transmute::<Address, GetSoNameT>(LINKER_DATA.soname_ptr.unwrap());
            let path = get_soname(handle);
            let size = strlen(path);
            return Ok(std::slice::from_raw_parts(path as *const u8, size).to_vec());
        } else {
            return Ok(soname_hack(handle as _));
        }
    }

    Err(Error::ItemNotFound)
}

/// Get the base from a handle.
/// Android 7 and above.
unsafe fn get_module_base_new(h: Handle) -> Result<Address> {
    if h & 1 == 0 {
        return get_module_base_old(h);
    }

    let mut h = h as *const sohandle_t;

    if !h.is_null() {
        if *API_LEVEL > ANDROID_API_7_1 {
            // Handle above android 7 (2 workaround fields).
            if let Some(m) = super::mappings::from_address((*h).base) {
                if m.base == (*h).base {
                    return Ok(m.base);
                }
            }

            let work_around_b_24465209 = std::mem::size_of::<Address>() * 2;
            h = (h as Address + work_around_b_24465209) as *const sohandle_t;

            if let Some(m) = super::mappings::from_address((*h).base) {
                if m.base == (*h).base {
                    return Ok(m.base);
                }
            }
        } else {
            // Handle android 7 (1 workaround field).

            let work_around_b_24465209 = std::mem::size_of::<Address>();
            h = (h as Address + work_around_b_24465209) as *const sohandle_t;

            if let Some(m) = super::mappings::from_address((*h).base) {
                if m.base == (*h).base {
                    return Ok(m.base);
                }
            }

            h = (h as Address + work_around_b_24465209) as *const sohandle_t;

            if let Some(m) = super::mappings::from_address((*h).base) {
                if m.base == (*h).base {
                    return Ok(m.base);
                }
            }
        }
    }

    Err(Error::ItemNotFound)
}

/// Get the path from a handle.
/// Below android 7.
unsafe fn get_module_path_old(h: Handle) -> Result<Vec<u8>> {
    let info = h as *const soinfo_t;

    if !info.is_null() {
        let path = (*info).name;
        if path[0] != '\0' as _ {
            return Ok(path.to_vec());
        }
    }

    Err(Error::ItemNotFound)
}

/// Get the base from a handle.
/// Below android 7.
unsafe fn get_module_base_old(h: Handle) -> Result<Address> {
    let info = h as *const soinfo_t;

    if !info.is_null() {
        return Ok((*info).base);
    }

    Err(Error::ItemNotFound)
}

// Android

pub unsafe fn add_android_linker_path(p: &Path) -> Result<()> {
    if let Some(path) = p.as_os_str().to_str() {
        let ld_data = &LD_PATH_DATA;
        let buffer = calloc(1, LD_PATHS_SIZE);

        if !buffer.is_null() {
            (ld_data.get_ld_library_path.unwrap())(buffer as _, LD_PATHS_SIZE);
            let paths = String::from_utf8(
                std::slice::from_raw_parts(buffer as *const u8, LD_PATHS_SIZE).to_vec(),
            );

            free(buffer);

            return match paths {
                Ok(paths) => {
                    let mut tokens: Vec<&str> = paths.split(':').collect();
                    tokens.push(path);
                    Ok((ld_data.update_ld_library_path.unwrap())(
                        tokens.join(":").as_ptr() as _,
                    ))
                }
                Err(_) => Err(Error::InvalidData),
            };
        }

        return Err(super::get_system_error_wrapped());
    }

    Err(Error::InvalidParameter)
}

pub unsafe fn remove_android_linker_path(p: &Path) -> Result<()> {
    if let Some(path) = p.as_os_str().to_str() {
        let ld_data = &LD_PATH_DATA;
        let buffer = calloc(1, LD_PATHS_SIZE);

        if !buffer.is_null() {
            (ld_data.get_ld_library_path.unwrap())(buffer as _, LD_PATHS_SIZE);
            let paths = String::from_utf8(
                std::slice::from_raw_parts(buffer as *const u8, LD_PATHS_SIZE).to_vec(),
            );

            free(buffer);

            if let Ok(paths) = paths {
                let mut tokens: Vec<&str> = paths.split(':').collect();
                return match tokens.iter().position(|&s| s == path) {
                    Some(index) => {
                        tokens.remove(index);
                        Ok((ld_data.update_ld_library_path.unwrap())(
                            tokens.join(":").as_ptr() as _,
                        ))
                    }
                    None => Err(Error::ItemNotFound),
                };
            }

            return Err(Error::InvalidData);
        }

        return Err(super::get_system_error_wrapped());
    }

    Err(Error::InvalidParameter)
}

pub unsafe fn get_android_other_module_path(h: Handle) -> Result<PathBuf> {
    let buffer = if *API_LEVEL >= ANDROID_API_7 {
        get_module_path_new(h)?
    } else {
        get_module_path_old(h)?
    };

    Ok(PathBuf::from(OsString::from_vec(buffer)))
}

pub unsafe fn get_android_module_base(h: Handle) -> Result<Address> {
    if *API_LEVEL >= ANDROID_API_7 {
        get_module_base_new(h)
    } else {
        get_module_base_old(h)
    }
}
