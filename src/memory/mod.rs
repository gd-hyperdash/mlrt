// Includes

use mlsys::*;

// Platform

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use self::windows::*;

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub use self::linux::*;

// Helpers

unsafe fn make_writable(address: Address, size: usize) -> Result<u32> {
    let p = self::query(address)?.mask;

    if p == MEM_N {
        return Err(Error::InvalidAccess);
    }

    if (p & MEM_W) == 0 {
        self::mask(address, size, p | MEM_W)?;
    }

    Ok(p)
}

// Memory

pub unsafe fn copy_unchecked(address: Address, source: Address, size: usize) {
    std::ptr::copy::<u8>(source as _, address as _, size)
}

pub unsafe fn copy(address: Address, source: Address, size: usize) -> Result<()> {
    let p = make_writable(address, size)?;

    copy_unchecked(address, source, size);

    if (p & MEM_W) == 0 {
        self::mask(address, size, p)?;
    }

    Ok(())
}

pub unsafe fn fill_unchecked(address: Address, size: usize, value: u8) {
    std::ptr::write_bytes::<u8>(address as _, value, size)
}

pub unsafe fn fill(address: Address, size: usize, value: u8) -> Result<()> {
    let p = make_writable(address, size)?;

    fill_unchecked(address, size, value);

    if (p & MEM_W) == 0 {
        self::mask(address, size, p)?;
    }

    Ok(())
}

// Bindings

#[no_mangle]
unsafe extern "C" fn MLMemoryFlush(address: Address, size: usize) -> Error {
    match self::flush(address, size) {
        Ok(()) => Error::Success,
        Err(e) => e,
    }
}

#[no_mangle]
unsafe extern "C" fn MLMemoryAlloc(size: usize, mask: u32, out: *mut Address, hint: Address) -> Error {
    if out.is_null() {
        return Error::InvalidArgument;
    }

    match self::allocate(size, mask, hint) {
        Ok(address) => {
            *out = address;
            Error::Success
        }
        Err(e) => e,
    }
}

#[no_mangle]
unsafe extern "C" fn MLMemoryFree(address: Address) -> Error {
    match self::free(address) {
        Ok(()) => Error::Success,
        Err(e) => e,
    }
}

#[no_mangle]
unsafe extern "C" fn MLMemoryMask(address: Address, size: usize, mask: u32) -> Error {
    match self::mask(address, size, mask) {
        Ok(()) => Error::Success,
        Err(e) => e,
    }
}

#[no_mangle]
unsafe extern "C" fn MLMemoryQuery(address: Address, info: *mut MemInfo) -> Error {
    if info.is_null() {
        return Error::InvalidArgument;
    }

    match self::query(address) {
        Ok(i) => {
            std::ptr::copy_nonoverlapping(&i, info, std::mem::size_of_val(&i));
            Error::Success
        }
        Err(e) => e,
    }
}

#[no_mangle]
unsafe extern "C" fn MLMemoryCopy(address: Address, source: Address, size: usize) -> Error {
    match self::copy(address, source, size) {
        Ok(()) => Error::Success,
        Err(e) => e,
    }
}

#[no_mangle]
unsafe extern "C" fn MLMemoryFill(address: Address, size: usize, value: u8) -> Error {
    match self::fill(address, size, value) {
        Ok(()) => Error::Success,
        Err(e) => e,
    }
}
