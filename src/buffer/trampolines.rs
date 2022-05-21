// Includes

use crate::types::*;
use lazy_static::*;
use mlsys::*;

// Types

struct Buffer {
    base: SyncAddress,
    size: usize,
}

// Globals

const BUFFER_SIZE: usize = 0x40000; // ~256kb

lazy_static! {
    static ref BUFFER: Mutex<Buffer> = Mutex::new(Buffer {
        base: unsafe {
            SyncAddress::from(crate::memory::allocate(BUFFER_SIZE, MEM_XRW, ALLOC_NO_HINT).unwrap())
        },
        size: 0
    });
}

// Trampoline

#[allow(unused)]
pub(crate) unsafe fn get_pointer() -> Result<Address> {
    let mut buffer = BUFFER.lock();
    Ok(buffer.base.extract().add(buffer.size))
}

#[allow(unused)]
pub(crate) unsafe fn insert_data(data: &[u8]) -> Result<Address> {
    let mut buffer = BUFFER.lock();
    let addr = buffer.base.extract().add(buffer.size);

    if buffer.size >= data.len() {
        crate::memory::copy_unchecked(Address::from(addr), data.as_ptr() as _, data.len());
        buffer.size += data.len();
        return Ok(Address::from(addr));
    }

    Err(Error::NoMemory)
}
