use core::fmt;
use std::arch::x86_64::_mm_minpos_epu16;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Write};
use std::num::NonZeroU32;

/// Error struct made to handle error codes returned from the operating system
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct SysError(pub NonZeroU32);

/// Takes a code and returns a SysError
const fn internal_error(n: u16) -> SysError {
    let code = SysError::INTERNAL_START + (n as u32);
    SysError(unsafe {
        NonZeroU32::new_unchecked(code)
    })
}

impl SysError {
    /// This target/platform is not supported
    pub const UNSUPPORTED: SysError = internal_error(0);
    /// The platform's ERRNO returned negative value
    pub const ERRNO_NOT_POSITIVE: SysError = internal_error(1);
    /// Error calling windows [`RtlGenRandom`](https://docs.microsoft.com/en-us/windows/win32/api/ntsecapi/nf-ntsecapi-rtlgenrandom)
    pub const WINDOWS_RTL_GEN_RANDOM: SysError = internal_error(4);
    /// RDRAND failed due to hardware error
    pub const FAILED_RDRAND: SysError = internal_error(5);
    /// RDRAND unsupported on this target
    pub const NO_RDRAND: SysError = internal_error(6);
    /// Codes below this point represent OS errors
    pub const INTERNAL_START: u32 = 0x80000000; // top bit is set to 1
    /// Codes at or below this point can be used to represent custom errors
    pub const CUSTOM_START: u32 = 0xC0000000; // top two bits are set to 1

    /// Extract the raw OS error if the error came from the OS
    #[inline]
    pub const fn raw_os_error(self) -> Option<i32> {
        if self.0.get() < Self::INTERNAL_START {
            #[cfg(target_os = "solid_asp3")]
            {
                Some(-(self.0.get() as i32))
            }
            #[cfg(not(target_os = "solid_asp3"))]
            {
                Some(self.0.get() as i32)
            }
        } else {
            None
        }
    }

    /// Returns the code
    #[inline]
    pub const fn code(self) -> NonZeroU32 {
        self.0
    }
}

impl Debug for SysError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut dbg = f.debug_struct("SysError");
        if let Some(errno) = self.raw_os_error() {
            dbg.field("os_error", &errno);
        } else if let Some(desc) = internal_desc(*self) {
            dbg.field("internal_code", &self.0.get());
            dbg.field("description", &desc);
        } else {
            dbg.field("unknown_code", &self.0.get());
        }
        dbg.finish()
    }
}

impl Display for SysError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(errno) = self.raw_os_error() {
            write!(f, "OS Error: {}", errno)
        } else if let Some(desc) = internal_desc(*self) {
            f.write_str(desc)
        } else {
            write!(f, "Unknown Error: {}", self.0.get())
        }
    }
}

impl Error for SysError {}

impl From<NonZeroU32> for SysError {
    fn from(value: NonZeroU32) -> Self {
        Self(value)
    }
}

/// Gets a description for the error if such is known
fn internal_desc(error: SysError) -> Option<&'static str> {
    match error {
        SysError::UNSUPPORTED => Some("getrandom: this target is not supported"),
        SysError::ERRNO_NOT_POSITIVE => Some("errno: did not return a positive value"),
        SysError::WINDOWS_RTL_GEN_RANDOM => Some("RtlGenRandom: Windows system function failure"),
        SysError::FAILED_RDRAND => Some("RDRAND: failed multiple times: CPU issue likely"),
        SysError::NO_RDRAND => Some("RDRAND: instruction not supported"),
        _ => None,
    }
}