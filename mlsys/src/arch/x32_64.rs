// Includes

use crate::core::*;

use iced_x86::*;
use lazy_static::*;

// Globals

#[cfg(target_pointer_width = "64")]
const BITNESS: u32 = 64;

#[cfg(target_pointer_width = "32")]
const BITNESS: u32 = 32;

const REDIRECT_FLOW: [Code; 30] = [
    Code::Jmp_m1616,
    Code::Jmp_m1632,
    Code::Jmp_m1664,
    Code::Jmp_ptr1616,
    Code::Jmp_ptr1632,
    Code::Jmp_rel16,
    Code::Jmp_rel32_32,
    Code::Jmp_rel32_64,
    Code::Jmp_rel8_16,
    Code::Jmp_rel8_32,
    Code::Jmp_rel8_64,
    Code::Jmp_rm16,
    Code::Jmp_rm32,
    Code::Jmp_rm64,
    Code::Jmpe_disp16,
    Code::Jmpe_disp32,
    Code::Jmpe_rm16,
    Code::Jmpe_rm32,
    Code::Retnw,
    Code::Retnd,
    Code::Retnq,
    Code::Retfw,
    Code::Retfd,
    Code::Retfq,
    Code::Retnw_imm16,
    Code::Retnd_imm16,
    Code::Retnq_imm16,
    Code::Retfw_imm16,
    Code::Retfd_imm16,
    Code::Retfq_imm16,
];

const PADDINGS: [Code; 7] = [
    Code::Nop_rm16,
    Code::Nop_rm32,
    Code::Nop_rm64,
    Code::Nopw,
    Code::Nopd,
    Code::Nopq,
    Code::Int3,
];

lazy_static! {
    static ref TRAP_DATA: Vec<u8> = single_encoder(&Instruction::with(Code::Ud2));
    static ref MAX_JUMP_SIZE: usize = get_jump_data(NULLPTR).len();
}

// Helpers

fn redirects_flow(insn: &Instruction) -> bool {
    REDIRECT_FLOW
        .iter()
        .position(|&code| code == insn.code())
        .is_some()
}

fn is_padding(insn: &Instruction) -> bool {
    PADDINGS
        .iter()
        .position(|&code| code == insn.code())
        .is_some()
}

fn single_encoder(insn: &Instruction) -> Vec<u8> {
    let mut encoder = Encoder::new(BITNESS);
    encoder.encode(insn, 0).unwrap();
    encoder.take_buffer()
}

#[cfg(target_pointer_width = "64")]
fn add_jump(buffer: &mut Vec<Instruction>, address: u64) {
    buffer.push(Instruction::with1(Code::Pushq_imm32, address as u32).unwrap());
    buffer.push(
        Instruction::with2(
            Code::Mov_rm32_imm32,
            MemoryOperand::with_base_displ(Register::RSP, 0x04),
            (address >> 32) as u32,
        )
        .unwrap(),
    );
    buffer.push(Instruction::with(Code::Retnq));
}

#[cfg(target_pointer_width = "32")]
fn add_jump(buffer: &mut Vec<Instruction>, target: u64) {
    buffer.push(Instruction::with1(Code::Pushd_imm32, target as u32).unwrap());
    buffer.push(Instruction::with(Code::Retnd));
}

// Arch

pub const fn max_insn_size() -> usize {
    15
}

pub fn max_jump_size() -> usize {
    *MAX_JUMP_SIZE
}

pub fn get_trap_data() -> Vec<u8> {
    TRAP_DATA.to_vec()
}

pub fn get_jump_data(target: Address) -> Vec<u8> {
    let mut buffer = Vec::new();
    let mut encoder = Encoder::new(BITNESS);
    add_jump(&mut buffer, target as _);

    for insn in buffer {
        encoder.encode(&insn, 0).unwrap();
    }

    encoder.take_buffer()
}

pub fn get_backjump_data(offset: u8) -> Vec<u8> {
    let offset = -(offset as i8);
    single_encoder(&Instruction::with1(Code::Jmp_rel8_16, offset as i32).unwrap())
}

pub fn get_overwrite_size(buffer: &[u8]) -> usize {
    let mut size = 0usize;
    let mut insn = Instruction::new();
    let mut decoder = Decoder::new(BITNESS, buffer, DecoderOptions::NONE);
    let mut flow_redirected = false;

    while decoder.can_decode() {
        decoder.decode_out(&mut insn);

        if insn.is_invalid() || (flow_redirected && !is_padding(&insn)) {
            break;
        }

        if redirects_flow(&insn) {
            flow_redirected = true;
        }

        size += insn.len();
    }

    size
}

pub fn get_padding_size(buffer: &[u8]) -> usize {
    let mut size = 0usize;
    let mut nop_state = 0usize;

    for &b in buffer.iter().rev() {
        if b == 0x90 || b == 0xCC {
            size += 1;
            continue;
        }

        if b == 0x0F && nop_state == 2 {
            size += 3;
            nop_state = 0;
            continue;
        }

        if b == 0x1F && nop_state == 1 {
            nop_state = 2;
            continue;
        }

        if nop_state == 0 {
            nop_state = 1;
            continue;
        }

        break;
    }

    size
}

pub fn relocate(buffer: &[u8], to: Address) -> Result<Vec<u8>> {
    let mut decoder = Decoder::new(BITNESS, buffer, DecoderOptions::NONE);
    let mut buffer = Vec::new();

    while decoder.can_decode() {
        let insn = decoder.decode();

        if insn.is_invalid() {
            return Err(Error::InvalidData);
        }

        buffer.push(insn);
    }

    let block = InstructionBlock::new(&buffer, to as _);
    Ok(
        BlockEncoder::encode(BITNESS, block, BlockEncoderOptions::NONE)
            .unwrap()
            .code_buffer,
    )
}
