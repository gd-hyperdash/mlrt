// Types

pub type Address = *mut libc::c_void;
pub type Handle = *const libc::c_void;
pub type RawString = *const libc::c_char;

#[repr(C)]
#[derive(Debug)]
pub enum Error {
    Success,
    ItemNotFound,
    InvalidAccess,
    InvalidArgument,
    InvalidData,
    NoMemory,
}

#[repr(C)]
#[derive(Debug)]
pub struct MemInfo {
    pub base: Address,
    pub size: usize,
    pub mask: u32,
    pub path: RawString,
}

pub type Result<T> = std::result::Result<T, Error>;

// Globals

pub const NULLPTR: Address = std::ptr::null_mut();

pub const ALLOC_NO_HINT: Address = NULLPTR;

pub const MEM_N: u32 = 0x00; // No access
pub const MEM_R: u32 = 0x01; // Read flag
pub const MEM_W: u32 = 0x02; // Write flag
pub const MEM_X: u32 = 0x04; // Execute flag
pub const MEM_RW: u32 = MEM_R | MEM_W;
pub const MEM_XR: u32 = MEM_X | MEM_R;
pub const MEM_XRW: u32 = MEM_X | MEM_R | MEM_W;

// Traits

pub trait RS {
    fn default() -> Self;
    fn from_bytes(buffer: &[u8]) -> Self;
    fn to_bytes<'a>(&self) -> Option<&'a [u8]>;
    fn free(&self);
}

impl RS for RawString {
    fn default() -> Self {
        NULLPTR as _
    }

    fn from_bytes(buffer: &[u8]) -> Self {
        unsafe {
            let p = libc::malloc(buffer.len());

            if !p.is_null() {
                libc::memcpy(p, buffer.as_ptr() as _, buffer.len());
            }

            p as _
        }
    }

    fn to_bytes<'a>(&self) -> Option<&'a [u8]> {
        match !self.is_null() {
            true => unsafe {
                Some(std::slice::from_raw_parts::<u8>(
                    *self as _,
                    libc::strlen(*self as _),
                ))
            },
            false => None,
        }
    }

    fn free(&self) {
        unsafe { libc::free(*self as _) }
    }
}
