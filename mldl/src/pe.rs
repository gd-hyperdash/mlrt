// Includes

use goblin::pe::*;
use goblin::*;

use crate::types::*;

use std::ffi::CStr;

// Globals

const DYN_SECTION: &'static str = ".mldyn";
const HOOK_SECTION: &'static str = ".mlhook";

// Helpers

unsafe fn find_raw_addr(pe: &PE, vaddr: u64) -> Option<u64> {
    let base = pe.image_base as u64;
    let sections = &pe.sections;

    for s in sections {
        let raw = s.pointer_to_raw_data as u64;
        let va = base + s.virtual_address as u64;
        let va_end = va + s.size_of_raw_data as u64;

        if vaddr > va && vaddr < va_end {
            return Some(vaddr - va + raw);
        }
    }

    None
}

unsafe fn read_raw_string<'a>(pe: &PE, base: u64, vaddr: u64) -> Option<&'a str> {
    match CStr::from_ptr((base + find_raw_addr(pe, vaddr)?) as _).to_str() {
        Ok(s) => Some(s),
        Err(_) => None,
    }
}

// Dumper

unsafe fn dump_dyn<T>(
    mut lib_offset: usize,
    base: u64,
    pe: &PE,
    dyn_table: &mut DynamicTable,
) -> error::Result<()>
where
    T: SecDynamic,
{
    let mut p: *const T = std::ptr::null_mut();
    let mut size = 0u64;

    if lib_offset != 0 {
        lib_offset -= pe.image_base;
    }

    for sec in &pe.sections {
        if sec.name()? == DYN_SECTION {
            p = (base + sec.pointer_to_raw_data as u64) as _;
            size = sec.size_of_raw_data as _;
            break;
        }
    }

    if let Some(mut entry) = p.as_ref() {
        let mut offset = 0u64;

        while offset < size {
            if entry.address() == 0 {
                break;
            }

            let sym = read_raw_string(pe, base, entry.sym());

            if sym.is_none() {
                return Err(error::Error::Malformed(String::from(
                    "Could not read binary metadata!",
                )));
            }

            let sym = sym.unwrap().to_string();
            let record = match read_raw_string(pe, base, entry.record()) {
                Some(s) => s.to_string(),
                None => String::from(""),
            };

            dyn_table.push(DynamicEntry {
                address: lib_offset + (entry.address() as usize),
                sym,
                record,
            });

            offset += std::mem::size_of::<T>() as u64;
            p = p.offset(1);
            entry = &*p;
        }
    }

    Ok(())
}

unsafe fn dump_hooks<T>(
    mut lib_offset: usize,
    base: u64,
    pe: &PE,
    hook_table: &mut HookTable,
) -> error::Result<()>
where
    T: SecHook,
{
    let mut p: *const T = std::ptr::null_mut();
    let mut size = 0u64;

    if lib_offset != 0 {
        lib_offset -= pe.image_base;
    }

    for sec in &pe.sections {
        if sec.name()? == HOOK_SECTION {
            p = (base + sec.pointer_to_raw_data as u64) as _;
            size = sec.size_of_raw_data as _;
            break;
        }
    }

    if let Some(mut entry) = p.as_ref() {
        let mut offset = 0u64;

        while offset < size {
            if entry.target() == 0 {
                break;
            }

            let flags = entry.flags();
            let is_dynamic = (flags & FLAG_DYNAMIC) != 0;

            if (flags & FLAG_DISPATCHER) != 0 {
                hook_table.dispatchers.push(HookEntry {
                    target: lib_offset + (entry.target() as usize),

                    callback: lib_offset + (entry.callback() as usize),
                    dispatcher: true,
                    dynamic: is_dynamic,
                    locking: false,
                    preload: false,
                    optional: false,
                    priority: false,
                });
            } else if (flags & FLAG_LOCKING) != 0 {
                hook_table.locking_hooks.push(HookEntry {
                    target: lib_offset + (entry.target() as usize),
                    callback: lib_offset + (entry.callback() as usize),
                    dispatcher: false,
                    dynamic: is_dynamic,
                    locking: true,
                    preload: (flags & FLAG_PRELOAD) != 0,
                    optional: (flags & FLAG_OPTIONAL) != 0,
                    priority: (flags & FLAG_PRIORITY) != 0,
                });
            } else {
                hook_table.hooks.push(HookEntry {
                    target: lib_offset + (entry.target() as usize),
                    callback: lib_offset + (entry.callback() as usize),
                    dispatcher: false,
                    dynamic: is_dynamic,
                    locking: false,
                    preload: (flags & FLAG_PRELOAD) != 0,
                    optional: (flags & FLAG_OPTIONAL) != 0,
                    priority: (flags & FLAG_PRIORITY) != 0,
                });
            }

            offset += std::mem::size_of::<T>() as u64;
            p = p.offset(1);
            entry = &*p;
        }
    }

    Ok(())
}

// PE

pub fn dump_pe(offset: usize, base: u64, pe: &PE) -> error::Result<(DynamicTable, HookTable)> {
    let mut dyn_table = DynamicTable::new();
    let mut hook_table = HookTable::default();

    if pe.is_64 {
        unsafe {
            dump_dyn::<SecDynamic64>(offset, base, pe, &mut dyn_table)?;
            dump_hooks::<SecHook64>(offset, base, pe, &mut hook_table)?;
        }
    } else {
        unsafe {
            dump_dyn::<SecDynamic32>(offset, base, pe, &mut dyn_table)?;
            dump_hooks::<SecHook32>(offset, base, pe, &mut hook_table)?;
        }
    }

    Ok((dyn_table, hook_table))
}
