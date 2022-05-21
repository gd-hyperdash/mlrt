mod elf;
mod pe;
mod types;

pub use self::types::*;

// Includes

use goblin::*;

use std::fs;
use std::path::Path;

// Types

pub type MLError<T> = error::Result<T>;

// MLDL

pub fn parse_ml_binary(offset: usize, path: &Path) -> MLError<(DynamicTable, HookTable)> {
    let buffer = fs::read(path)?;

    match Object::parse(&buffer)? {
        Object::Elf(elf) => elf::dump_elf(offset, buffer.as_ptr() as _, &elf),
        Object::PE(pe) => pe::dump_pe(offset, buffer.as_ptr() as _, &pe),
        _ => Err(error::Error::Malformed(String::from(
            "Invalid binary file!",
        ))),
    }
}
