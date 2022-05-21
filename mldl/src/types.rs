// Includes

use std::fmt::{Display, Formatter, Result};

// Globals

pub(crate) const FLAG_DISPATCHER: u64 = 0x01;
pub(crate) const FLAG_DYNAMIC: u64 = 0x02;
pub(crate) const FLAG_LOCKING: u64 = 0x04;
pub(crate) const FLAG_PRELOAD: u64 = 0x08;
pub(crate) const FLAG_OPTIONAL: u64 = 0x10;
pub(crate) const FLAG_PRIORITY: u64 = 0x20;

// Types

#[derive(PartialEq)]
pub enum ModuleKind {
    Binary,
}

pub struct DynamicEntry {
    pub address: usize,
    pub sym: String,
    pub record: String,
}

pub struct HookEntry {
    pub target: usize,
    pub callback: usize,
    pub dispatcher: bool,
    pub dynamic: bool,
    pub locking: bool,
    pub preload: bool,
    pub optional: bool,
    pub priority: bool,
}

pub type DynamicTable = Vec<DynamicEntry>;

#[derive(Default)]
pub struct HookTable {
    pub dispatchers: Vec<HookEntry>,
    pub hooks: Vec<HookEntry>,
    pub locking_hooks: Vec<HookEntry>,
}

// Sections

#[repr(C)]
pub(crate) struct SecDynamic32 {
    pub addr: u32,
    pub sym: u32,
    pub record: u32,
}

#[repr(C)]
pub(crate) struct SecDynamic64 {
    pub addr: u64,
    pub sym: u64,
    pub record: u64,
}

pub(crate) trait SecDynamic {
    fn address(&self) -> u64;
    fn sym(&self) -> u64;
    fn record(&self) -> u64;
}

#[repr(C)]
pub(crate) struct SecHook32 {
    pub target: u32,
    pub callback: u32,
    pub flags: u64,
}

#[repr(C)]
pub(crate) struct SecHook64 {
    pub target: u64,
    pub callback: u64,
    pub flags: u64,
}

pub(crate) trait SecHook {
    fn target(&self) -> u64;
    fn callback(&self) -> u64;
    fn flags(&self) -> u64;
}

// Impl

impl SecDynamic for SecDynamic32 {
    fn address(&self) -> u64 {
        self.addr as _
    }

    fn sym(&self) -> u64 {
        self.sym as _
    }

    fn record(&self) -> u64 {
        self.record as _
    }
}

impl SecDynamic for SecDynamic64 {
    fn address(&self) -> u64 {
        self.addr
    }

    fn sym(&self) -> u64 {
        self.sym
    }

    fn record(&self) -> u64 {
        self.record
    }
}

impl SecHook for SecHook32 {
    fn target(&self) -> u64 {
        self.target as _
    }

    fn callback(&self) -> u64 {
        self.callback as _
    }

    fn flags(&self) -> u64 {
        self.flags
    }
}

impl SecHook for SecHook64 {
    fn target(&self) -> u64 {
        self.target
    }

    fn callback(&self) -> u64 {
        self.callback
    }

    fn flags(&self) -> u64 {
        self.flags
    }
}

impl Display for DynamicEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "Address: 0x{:X}", self.address)?;
        writeln!(f, "Symbol: {}", self.sym)?;
        writeln!(
            f,
            "Record: {}",
            if self.record.is_empty() {
                "(none)"
            } else {
                &self.record
            }
        )
    }
}

impl Display for HookEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let read_opt = |b: bool| match b {
            true => "Yes",
            false => "No",
        };

        writeln!(f, "Target: 0x{:X}", self.target)?;
        writeln!(f, "Callback: 0x{:X}", self.callback)?;
        writeln!(f, "Is dispatcher? {}", read_opt(self.dispatcher))?;
        writeln!(f, "Is dynamic? {}", read_opt(self.dynamic))?;
        writeln!(f, "Is locking? {}", read_opt(self.locking))?;
        writeln!(f, "Is preload? {}", read_opt(self.preload))?;
        writeln!(f, "Is optional? {}", read_opt(self.optional))?;
        writeln!(f, "Is priority? {}", read_opt(self.priority))
    }
}
