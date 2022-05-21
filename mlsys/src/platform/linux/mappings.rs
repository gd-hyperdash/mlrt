// Includes

use crate::core::*;

use nix::sys::mman::ProtFlags;

use lazy_static::*;
use regex::{Captures, Regex};

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::vec::Vec;

use std::path::PathBuf;

// Types

#[derive(Debug)]
pub struct MemoryMap {
    pub base: Address,
    pub end: Address,
    pub flags: ProtFlags,
    pub is_shared: bool,
    pub offset: usize,
    pub major_device: u8,
    pub minor_device: u8,
    pub inode: u32,
    pub path: Option<PathBuf>,
}

// Globals

const REGEX_STR: &str = r#"(?i)([0-9A-F]+)-([0-9A-F]+)\s+([rwxps-]+)\s+([0-9A-F]+)\s+([0-9A-F]+):([0-9A-F]+)\s+([0-9]+)\s*(.*)?"#;

lazy_static! {
    static ref REGEX: Regex = Regex::new(REGEX_STR).unwrap();
}

// Helpers

fn parse_dec_string(s: &str) -> Option<usize> {
    match usize::from_str_radix(s, 10) {
        Ok(v) => Some(v),
        Err(_e) => None,
    }
}

fn parse_hex_string(s: &str) -> Option<Address> {
    match usize::from_str_radix(s, 16) {
        Ok(v) => Some(v as _),
        Err(_e) => None,
    }
}

fn wrap_flags(s: &str) -> Option<ProtFlags> {
    let mut flags = ProtFlags::PROT_NONE;

    if s.chars().nth(0)? == 'r' {
        flags |= ProtFlags::PROT_READ;
    }

    if s.chars().nth(1)? == 'w' {
        flags |= ProtFlags::PROT_WRITE;
    }

    if s.chars().nth(2)? == 'x' {
        flags |= ProtFlags::PROT_EXEC;
    }

    Some(flags)
}

fn is_shared(s: &str) -> Option<bool> {
    Some(s.chars().nth(3)? == 's')
}

fn get_path(groups: &Captures) -> Option<PathBuf> {
    let p = PathBuf::from(groups.get(8)?.as_str());

    match p.as_os_str().is_empty() {
        false => Some(p),
        true => None,
    }
}

fn parse_line(line: &str) -> Option<MemoryMap> {
    let groups = REGEX.captures(line)?;

    Some(MemoryMap {
        base: parse_hex_string(&groups[1])?,
        end: parse_hex_string(&groups[2])?,
        flags: wrap_flags(&groups[3])?,
        is_shared: is_shared(&groups[3])?,
        offset: parse_hex_string(&groups[4])? as _,
        major_device: parse_hex_string(&groups[5])? as _,
        minor_device: parse_hex_string(&groups[6])? as _,
        inode: parse_dec_string(&groups[7])? as _,
        path: get_path(&groups),
    })
}

// Mappings

pub fn get() -> Option<Vec<MemoryMap>> {
    let mut buffer: Vec<MemoryMap> = Vec::new();

    match File::open("/proc/self/maps") {
        Ok(input) => {
            let input = BufReader::new(input);
            for line in input.lines() {
                if let Ok(line) = line {
                    buffer.push(parse_line(&line)?);
                }
            }

            Some(buffer)
        }
        Err(_e) => None,
    }
}

pub fn from_address(address: Address) -> Option<MemoryMap> {
    let maps = get()?;

    for map in maps {
        if map.base <= address && map.end > address {
            return Some(map);
        }
    }

    None
}

pub fn from_path(path: &Path) -> Option<MemoryMap> {
    let maps = get()?;

    for mut map in maps {
        if path == map.path.take()? {
            return Some(map);
        }
    }

    None
}
