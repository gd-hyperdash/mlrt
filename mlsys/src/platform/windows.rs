// Includes

use crate::core::*;

pub use windows::Win32::Foundation::*;
pub use windows::Win32::System::Diagnostics::Debug::*;
pub use windows::Win32::System::Diagnostics::ToolHelp::*;
pub use windows::Win32::System::LibraryLoader::*;
pub use windows::Win32::System::Memory::*;
pub use windows::Win32::System::Threading::*;
pub use windows::Win32::UI::WindowsAndMessaging::*;

// Types

pub type PathCookie = usize;

#[derive(Default)]
pub struct WinapiSnapshot {
    pub handle: HANDLE,
}

#[derive(Default)]
pub struct WinapiVehHandle {
    pub handle: usize,
}

// Globals

pub const NULL_HANDLE_VALUE: HINSTANCE = HINSTANCE(0isize);

// WinapiSnapshot

impl WinapiSnapshot {
    pub fn new() -> Self {
        let mut ss = WinapiSnapshot::default();

        unsafe {
            ss.handle = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE | TH32CS_SNAPMODULE32, 0);
        }

        if ss.handle != INVALID_HANDLE_VALUE {
            return ss;
        }

        panic!("Could not initialize snapshot!")
    }
}

impl Drop for WinapiSnapshot {
    fn drop(&mut self) {
        unsafe {
            if self.handle != INVALID_HANDLE_VALUE {
                if !CloseHandle(self.handle).as_bool() {
                    panic!("Could not free snapshot handle!");
                }
            }
        }
    }
}

// WinapiVehHandle

impl WinapiVehHandle {
    pub fn new(handler: PVECTORED_EXCEPTION_HANDLER) -> Self {
        let mut h = WinapiVehHandle::default();

        unsafe {
            h.handle = AddVectoredExceptionHandler(1, Some(handler)) as _;
        }

        if h.handle != 0 {
            return h;
        }

        panic!("Could not initialize VEH!")
    }
}

impl Drop for WinapiVehHandle {
    fn drop(&mut self) {
        unsafe {
            if self.handle != 0 {
                if RemoveVectoredExceptionHandler(self.handle as _) == 0 {
                    panic!("Could not cleanup VEH!");
                }
            }
        }
    }
}

// Windows

// TODO: complete.
pub unsafe fn wrap_system_error(error: WIN32_ERROR) -> Error {
    match error {
        _ => panic!("System call failed with error: {} ({:?})", error.0, error),
    }
}

pub unsafe fn get_system_error() -> WIN32_ERROR {
    GetLastError()
}

pub unsafe fn get_system_error_casted() -> u32 {
    get_system_error().0 as u32
}

pub unsafe fn get_system_error_wrapped() -> Error {
    wrap_system_error(get_system_error())
}

pub unsafe fn get_winapi_hinstance(val: isize) -> HINSTANCE {
    let mut data = HINSTANCE::default();
    data.0 = val;
    data
}

pub unsafe fn get_winapi_module_path(handle: HINSTANCE) -> Result<Vec<u16>> {
    let mut buffer = PWSTR::default();
    let mut size = 0u32;

    loop {
        size += MAX_PATH;
        buffer.0 = libc::realloc(buffer.0 as _, size as _) as _;
        size = GetModuleFileNameW(handle, buffer, size as _);
        let error = GetLastError();

        if error != ERROR_INSUFFICIENT_BUFFER {
            if error != ERROR_SUCCESS {
                return Err(wrap_system_error(error));
            }
        }

        break;
    }

    Ok(std::slice::from_raw_parts(buffer.0, size as _).to_vec())
}
