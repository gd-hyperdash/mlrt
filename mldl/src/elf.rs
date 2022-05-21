#![allow(unused)]

// Includes

use goblin::elf::*;
use goblin::*;

use crate::types::*;

// Helpers

unsafe fn dump_dyn<T>(
    mut lib_offset: usize,
    base: u64,
    elf: &Elf,
    dyn_table: &mut DynamicTable,
) -> error::Result<()>
where
    T: SecDynamic,
{
    todo!("Unimplemented!")
}

unsafe fn dump_hooks<T>(
    mut lib_offset: usize,
    base: u64,
    elf: &Elf,
    hook_table: &mut HookTable,
) -> error::Result<()>
where
    T: SecHook,
{
    todo!("Unimplemented!")
}

// ELF

pub fn dump_elf(offset: usize, base: u64, elf: &Elf) -> error::Result<(DynamicTable, HookTable)> {
    let mut dyn_table = DynamicTable::new();
    let mut hook_table = HookTable::default();

    if elf.is_64 {
        unsafe {
            dump_dyn::<SecDynamic64>(offset, base, elf, &mut dyn_table)?;
            dump_hooks::<SecHook64>(offset, base, elf, &mut hook_table)?;
        }
    } else {
        unsafe {
            dump_dyn::<SecDynamic32>(offset, base, elf, &mut dyn_table)?;
            dump_hooks::<SecHook32>(offset, base, elf, &mut hook_table)?;
        }
    }

    Ok((dyn_table, hook_table))
}
