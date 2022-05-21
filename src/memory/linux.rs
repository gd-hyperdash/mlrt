// Includes

use mlsys::*;

use lazy_static::*;

use std::os::unix::ffi::OsStrExt;

// Globals

lazy_static! {
    static ref PAGE_SIZE: usize = platform::sysconf(platform::SysconfVar::PAGE_SIZE)
        .unwrap()
        .unwrap() as usize;
}

// Helpers

unsafe fn page_of(address: Address) -> Address {
    address.sub((address as usize) & (*PAGE_SIZE - 1))
}

fn round_size(size: usize) -> usize {
    *PAGE_SIZE + (size & !(*PAGE_SIZE - 1))
}

fn get_os_protection(mask: u32) -> platform::ProtFlags {
    let mut p = platform::ProtFlags::PROT_NONE;

    if mask & MEM_R == MEM_R {
        p |= platform::ProtFlags::PROT_READ;
    }

    if mask & MEM_W == MEM_W {
        p |= platform::ProtFlags::PROT_WRITE;
    }

    if mask & MEM_X == MEM_X {
        p |= platform::ProtFlags::PROT_EXEC;
    }

    p
}

fn wrap_protection(p: platform::ProtFlags) -> u32 {
    let mut prot = MEM_N;

    if p & platform::ProtFlags::PROT_READ == platform::ProtFlags::PROT_READ {
        prot |= MEM_R;
    }

    if p & platform::ProtFlags::PROT_WRITE == platform::ProtFlags::PROT_WRITE {
        prot |= MEM_W;
    }

    if p & platform::ProtFlags::PROT_EXEC == platform::ProtFlags::PROT_EXEC {
        prot |= MEM_X;
    }

    prot
}

// Mem

pub unsafe fn flush(address: Address, size: usize) -> Result<()> {
    let page = page_of(address);
    let p = query(page)?.mask;

    if p == MEM_N {
        return Err(Error::InvalidAccess);
    }

    Ok(platform::cacheflush(page, round_size(size)))
}

pub unsafe fn allocate(size: usize, mask: u32, hint: Address) -> Result<Address> {
    match platform::mmap(
        hint as _,
        size,
        get_os_protection(mask),
        platform::MapFlags::MAP_ANONYMOUS | platform::MapFlags::MAP_PRIVATE,
        -1,
        0,
    ) {
        Ok(address) => Ok(address as Address),
        Err(e) => Err(platform::wrap_system_error(e)),
    }
}

pub unsafe fn free(address: Address) -> Result<()> {
    if let Some(map) = platform::mappings::from_address(address as _) {
        return match platform::munmap(address as _, map.end.offset_from(map.base) as _) {
            Ok(()) => Ok(()),
            Err(e) => Err(platform::wrap_system_error(e)),
        };
    }

    Err(Error::ItemNotFound)
}

pub unsafe fn mask(address: Address, size: usize, mask: u32) -> Result<()> {
    let p = get_os_protection(mask);

    match platform::mprotect(page_of(address) as _, size, p) {
        Ok(()) => Ok(()),
        Err(e) => Err(platform::wrap_system_error(e)),
    }
}

pub unsafe fn query(address: Address) -> Result<MemInfo> {
    if let Some(map) = platform::mappings::from_address(address as _) {
        return Ok(MemInfo {
            base: map.base,
            size: map.end.offset_from(map.base) as _,
            mask: wrap_protection(map.flags),
            path: if let Some(p) = map.path {
                RawString::from_bytes(p.as_os_str().as_bytes())
            } else {
                RawString::default()
            },
        });
    }

    Err(Error::ItemNotFound)
}
