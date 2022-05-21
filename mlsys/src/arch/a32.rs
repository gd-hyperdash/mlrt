// Includes

use crate::core::*;

// Arch

pub const fn max_insn_size() -> usize {
    4
}

pub fn get_trap_data() -> Vec<u8> {
    panic!("Unimplemented")
}

pub fn get_jump_data(from: Address, to: Address) -> Vec<u8> {
    panic!("Unimplemented")
}

pub fn get_backjump_data(offset: u8) -> Vec<u8> {
    panic!("Unimplemented")
}

pub fn get_overwrite_size(buffer: &[u8]) -> usize {
    panic!("Unimplemented")
}

pub fn get_padding_size(buffer: &[u8]) -> usize {
    panic!("Unimplemented")
}

pub fn relocate(buffer: &[u8], to: Address) -> Result<Vec<u8>> {
    panic!("Unimplemented")
}
