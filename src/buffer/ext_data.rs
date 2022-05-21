// Includes

use crate::types::*;
use lazy_static::*;
use mlsys::*;

// Globals

lazy_static! {
    static ref EXTENSIONS: Mutex<NoHashMap<FNV, SyncAddress>> = Mutex::new(NoHashMap::default());
}

// Bindings

#[no_mangle]
unsafe extern "C" fn MLInsertExt(ext: SyncAddress, id: FNV) -> Bool {
    match EXTENSIONS.lock().insert(id, ext).is_none() {
        true => Bool::True,
        false => Bool::False,
    }
}

#[no_mangle]
unsafe extern "C" fn MLRemoveExt(id: FNV) -> Bool {
    match EXTENSIONS.lock().remove(&id).is_some() {
        true => Bool::True,
        false => Bool::False,
    }
}

#[no_mangle]
unsafe extern "C" fn MLExtFromBase(id: FNV) -> Address {
    match EXTENSIONS.lock().get(&id) {
        Some(&ext) => return ext.extract(),
        None => NULLPTR,
    }
}
