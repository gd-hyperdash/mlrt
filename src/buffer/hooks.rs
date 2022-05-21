// Includes

use mlsys::*;

use crate::types::*;

// Types

struct ChainData {
    trampoline: Address,
    dispatchers: Vec<Address>,
    hooks: Vec<Address>,
    lock: bool,
}

// Helpers

// Bindings

#[no_mangle]
unsafe extern "C" fn MLEnableHook(target: Address, hook: Address) -> Bool {
    todo!("Unimplemented!")
}

#[no_mangle]
unsafe extern "C" fn MLDisableHook(hook: Address) -> Bool {
    todo!("Unimplemented!")
}

#[no_mangle]
unsafe extern "C" fn MLEnumerateHooks(owner: Handle) -> Address {
    todo!("UN")
}

#[no_mangle]
unsafe extern "C" fn MLInitRecord(handle: Handle, id: RawString) -> Bool {
    todo!("Unimplemented!")
}

#[no_mangle]
unsafe extern "C" fn MLCleanupRecord(handle: Handle, id: RawString) -> Bool {
    todo!("Unimplemented!")
}

#[no_mangle]
unsafe extern "C" fn MLGetHookSize(target: Address) -> usize {
    todo!("Unimplemented!")
}

#[no_mangle]
unsafe extern "C" fn MLGetFirstChainHook(_base: Address) -> Address {
    todo!("Unimplemented!")
}

#[no_mangle]
unsafe extern "C" fn MLGetNextChainHook(_base: Address, _current: Address) -> Address {
    todo!("Unimplemented!")
}
