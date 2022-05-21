// Includes

use mlsys::*;

// Globals

const EXEC_FLAGS: [platform::PAGE_PROTECTION_FLAGS; 4] = [
    platform::PAGE_EXECUTE,
    platform::PAGE_EXECUTE_READ,
    platform::PAGE_EXECUTE_READWRITE,
    platform::PAGE_EXECUTE_WRITECOPY,
];

const READ_FLAGS: [platform::PAGE_PROTECTION_FLAGS; 5] = [
    platform::PAGE_READONLY,
    platform::PAGE_READWRITE,
    platform::PAGE_WRITECOPY,
    platform::PAGE_EXECUTE_READ,
    platform::PAGE_EXECUTE_READWRITE,
];

const WRITE_FLAGS: [platform::PAGE_PROTECTION_FLAGS; 4] = [
    platform::PAGE_READWRITE,
    platform::PAGE_WRITECOPY,
    platform::PAGE_EXECUTE_READWRITE,
    platform::PAGE_EXECUTE_WRITECOPY,
];

const OTHER_FLAGS: [platform::PAGE_PROTECTION_FLAGS; 4] = [
    platform::PAGE_TARGETS_NO_UPDATE,
    platform::PAGE_GUARD,
    platform::PAGE_NOCACHE,
    platform::PAGE_WRITECOMBINE,
];

// Helpers

fn get_os_protection(mask: u32) -> Option<platform::PAGE_PROTECTION_FLAGS> {
    match mask {
        MEM_N => Some(platform::PAGE_NOACCESS),
        MEM_R => Some(platform::PAGE_READONLY),
        MEM_RW => Some(platform::PAGE_READWRITE),
        MEM_X => Some(platform::PAGE_EXECUTE),
        MEM_XR => Some(platform::PAGE_EXECUTE_READ),
        MEM_XRW => Some(platform::PAGE_EXECUTE_READWRITE),
        _ => None,
    }
}

fn wrap_protection(mut p: platform::PAGE_PROTECTION_FLAGS) -> u32 {
    let mut flags = MEM_N;

    for flag in OTHER_FLAGS {
        p &= !(flag);
    }

    for flag in READ_FLAGS {
        if p == flag {
            flags |= MEM_R;
        }
    }

    for flag in WRITE_FLAGS {
        if p == flag {
            flags |= MEM_W;
        }
    }

    for flag in EXEC_FLAGS {
        if p == flag {
            flags |= MEM_X;
        }
    }

    flags
}

unsafe fn get_path_of_address(address: Address) -> RawString {
    match platform::get_winapi_module_path(platform::get_winapi_hinstance(address as _)) {
        Ok(v) => RawString::from_buffer(&v),
        Err(_e) => RawString::default(),
    }
}

// Mem

pub unsafe fn flush(address: Address, size: usize) -> Result<()> {
    if !platform::FlushInstructionCache(platform::GetCurrentProcess(), address as _, size).as_bool()
    {
        return Err(platform::get_system_error_wrapped());
    }

    Ok(())
}

pub unsafe fn allocate(size: usize, mask: u32, hint: Address) -> Result<Address> {
    if let Some(p) = get_os_protection(mask) {
        let addr = platform::VirtualAlloc(
            hint as _,
            size,
            platform::MEM_COMMIT | platform::MEM_RESERVE,
            p,
        );

        if addr.is_null() {
            return Err(platform::get_system_error_wrapped());
        }

        return Ok(addr as Address);
    }

    Err(Error::InvalidParameter)
}

pub unsafe fn free(address: Address) -> Result<()> {
    if !platform::VirtualFree(address as _, 0, platform::MEM_RELEASE).as_bool() {
        return Err(platform::get_system_error_wrapped());
    }

    Ok(())
}

pub unsafe fn mask(address: Address, size: usize, mask: u32) -> Result<()> {
    let mut p_out = platform::PAGE_NOACCESS;

    if let Some(p) = get_os_protection(mask) {
        if !platform::VirtualProtect(address as _, size, p, &mut p_out).as_bool() {
            return Err(platform::get_system_error_wrapped());
        }

        return Ok(());
    }

    Err(mlsys::Error::InvalidParameter)
}

pub unsafe fn query(address: Address) -> Result<MemInfo> {
    let mut buf = platform::MEMORY_BASIC_INFORMATION::default();
    let size = std::mem::size_of_val(&buf);

    if platform::VirtualQuery(address as _, &mut buf, size) != size {
        return Err(platform::get_system_error_wrapped());
    }

    let flags = wrap_protection(buf.Protect);

    Ok(MemInfo {
        base: buf.BaseAddress as Address,
        size: buf.RegionSize,
        mask: flags,
        path: get_path_of_address(buf.AllocationBase as _),
    })
}

