// Includes

use mlsys::*;

// Types

pub(crate) enum HookType {
    Inline,
    Backjump,
    Trap,
}

#[allow(unused)]
pub(crate) struct HookData {
    hook_type: HookType,
    offset: usize,
    original: Vec<u8>,
}

// Helpers

fn build_hook_data(hook_type: HookType, offset: usize, original: &[u8]) -> HookData {
    HookData {
        hook_type,
        offset,
        original: Vec::from(original),
    }
}

// Hook

#[allow(unused)]
pub(crate) unsafe fn place_internal(from: Address, to: Address) -> Result<HookData> {
    // Get inline hook data.
    let mut inline_data = arch::get_jump_data(to);

    // Read prolog data.
    let mut buffer = vec![0u8; inline_data.len() + arch::max_insn_size()];
    crate::memory::copy(buffer.as_mut_ptr() as _, from, buffer.len())?;

    // Get max bytes we can overwrite.
    let prolog_max = arch::get_overwrite_size(&buffer);

    // Do we have enough space for the inline hook?
    if inline_data.len() <= prolog_max {
        // Save original bytes.
        buffer.resize(inline_data.len(), 0u8);
        crate::memory::copy(buffer.as_mut_ptr() as _, from, buffer.len())?;

        // Overwrite the prolog.
        crate::memory::copy(from, inline_data.as_ptr() as _, inline_data.len())?;
        return Ok(build_hook_data(HookType::Inline, 0, &buffer));
    }

    // Attempt backjumping.
    let mut backjump_data = arch::get_backjump_data(inline_data.len() as u8);

    // Do we have enough space for backjumping?
    if backjump_data.len() <= prolog_max {
        // Read upper paddings.
        buffer.resize(buffer.len(), 0);
        crate::memory::copy(
            buffer.as_mut_ptr() as _,
            from.sub(inline_data.len()),
            buffer.len(),
        )?;

        // Get max upper bytes we can overwrite.
        let padding_max = arch::get_padding_size(&buffer);

        // Can we abuse upper paddings?
        if inline_data.len() <= padding_max {
            // Prepare payload.
            let backsize = inline_data.len();
            inline_data.append(&mut backjump_data);

            // Save original bytes.
            buffer.resize(inline_data.len(), 0u8);
            crate::memory::copy(buffer.as_mut_ptr() as _, from.sub(backsize), buffer.len())?;

            // Overwrite the paddings and the prolog.
            crate::memory::copy(
                from.sub(backsize),
                inline_data.as_ptr() as _,
                inline_data.len(),
            )?;

            return Ok(build_hook_data(HookType::Backjump, backsize, &buffer));
        }
    }

    // We have to rely on a trap.
    let trap_data = arch::get_trap_data();

    if trap_data.len() <= prolog_max {
        // Save original bytes.
        buffer.resize(trap_data.len(), 0u8);
        crate::memory::copy(buffer.as_mut_ptr() as _, from, buffer.len())?;

        // Overwrite the prolog.
        crate::memory::copy(from, trap_data.as_ptr() as _, trap_data.len())?;
        return Ok(build_hook_data(HookType::Trap, 0, &buffer));
    }

    // We cant hook the address.
    Err(Error::NoMemory)
}

pub unsafe fn place(from: Address, to: Address) -> Result<usize> {
    let buffer = arch::get_jump_data(to);
    crate::memory::copy(from, buffer.as_ptr() as _, buffer.len())?;
    Ok(buffer.len())
}

// Bindings

#[no_mangle]
unsafe extern "C" fn MLHookSize(to: Address) -> usize {
    arch::get_jump_data(to).len()
}

#[no_mangle]
unsafe extern "C" fn MLPlaceHook(from: Address, to: Address) -> Error {
    match self::place(from, to) {
        Ok(_) => Error::Success,
        Err(e) => e,
    }
}
