// Arch

#[cfg(target_arch = "aarch64")]
mod a64;

#[cfg(target_arch = "aarch64")]
pub use self::a64::*;

#[cfg(target_arch = "arm")]
mod a32;

#[cfg(target_arch = "arm")]
pub use self::a32::*;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod x32_64;

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub use self::x32_64::*;
