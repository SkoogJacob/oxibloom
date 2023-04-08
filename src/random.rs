const BUFFER_LENGTH: usize = 1024;
#[cfg(target_os = "windows")]
static mut BUFFER: [MaybeUninit<u8>; BUFFER_LENGTH] = [MaybeUninit::uninit(); BUFFER_LENGTH];
#[cfg(target_os = "linux")]
static mut BUFFER: [u8; BUFFER_LENGTH] = [0; BUFFER_LENGTH];
static mut BUFFER_INDEX: usize = 0;
static mut INITIALIZED: bool = false;

macro_rules! read_bytes {
    ($t:ty, $byte_count:literal) => {
        {
            let bytes = <[u8; $byte_count]>::try_from(get_buffer_slice($byte_count)).expect(&format!("Unable to convert from slice of length {}", $byte_count));
            #[cfg(target_endian = "big")]
            {
                <$t>::from_be_bytes(bytes)
            }
            #[cfg(target_endian = "little")]
            {
                <$t>::from_le_bytes(bytes)
            }
        }
    };
}

pub fn get_random_u8() -> u8 {
    let slice = get_buffer_slice(1);
    slice[0]
}

pub fn get_random_i8() -> i8 {
    let slice = get_buffer_slice(1);
    slice[0] as i8
}

pub fn get_random_u16() -> u16 {
    read_bytes!(u16, 2)
}

pub fn get_random_i16() -> i16 {
    read_bytes!(i16, 2)
}

pub fn get_random_u32() -> u32 {
    read_bytes!(u32, 4)
}

pub fn get_random_i32() -> i32 {
    read_bytes!(i32, 4)
}

pub fn get_random_u64() -> u64 {
    read_bytes!(u64, 8)
}

pub fn get_random_i64() -> i64 {
    read_bytes!(i64, 8)
}

pub fn get_random_u128() -> u128 {
    read_bytes!(u128, 16)
}

pub fn get_random_i128() -> i128 {
    read_bytes!(i128, 16)
}

fn get_buffer_slice(slice_size: usize) -> &'static [u8] {
    if !unsafe {INITIALIZED.clone()} {
        initialize_buffer_slice();
    }
    let mut start_index = unsafe { BUFFER_INDEX.clone() };
    if !check_slice_bounds(start_index, slice_size) {
        unsafe { INITIALIZED = false; }
        initialize_buffer_slice();
        unsafe { BUFFER_INDEX = 0 }
        start_index = 0;
    }

    unsafe {
        let slice = &BUFFER[start_index..(start_index + slice_size)];
        BUFFER_INDEX = start_index + slice_size;
        #[cfg(target_os = "windows")]
        {
            & *(slice as *const [MaybeUninit<u8>] as *const [u8])
        }
        #[cfg(target_os = "linux")]
        {
            slice
        }
    }
}

fn initialize_buffer_slice() {
    #[cfg(target_os = "windows")]
    unsafe {
        match windows::getrandom_inner(&mut BUFFER) {
            Ok(_) => {
                // SAFETY: should be safe as this is only set here?
                INITIALIZED = true;
            }
            Err(e) => {
                let err = format!("Unable to fill buffer, got error: {e}");
                panic!("{}", &err)
            }
        }
    }
    #[cfg(target_os = "linux")]
    unsafe {
        linux::getrandom_inner(&mut BUFFER);
    }
}

#[inline]
fn check_slice_bounds(start_index: usize, slice_len: usize) -> bool {
    start_index + slice_len < BUFFER_LENGTH
}

#[cfg(target_os = "windows")]
mod windows {
    use std::error::Error;
    use std::ffi::c_void;
    use std::mem::MaybeUninit;
    use std::num::NonZeroU32;
    use crate::error::SysError;

    const BCRYPT_USE_SYSTEM_PREFERRED_RNG: u32 = 0x00000002;

    #[allow(non_snake_case)]
    #[link(name = "bcrypt")]
    extern "system" {
        fn BCryptGenRandom(
            hAlgorithm: *mut c_void,
            pBuffer: *mut u8,
            cbBuffer: u32,
            dwFlags: u32,
        ) -> u32;
    }

    pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), impl Error> {
        for chunk in dest.chunks_mut(u32::MAX as usize) {
            let ret = unsafe {
                BCryptGenRandom(
                    core::ptr::null_mut(),
                    chunk.as_mut_ptr() as *mut u8,
                    chunk.len() as u32,
                    BCRYPT_USE_SYSTEM_PREFERRED_RNG,
                )
            };
            if !extern_error(ret) {
                continue
            }

            let code = ret ^ (1 << 31);
            let code = unsafe {
                NonZeroU32::new_unchecked(code)
            };
            return Err(SysError(code))
        }

        Ok(())
    }

    #[inline]
    fn extern_error(extern_return: u32) -> bool {
        extern_return >> 30 == 0b11
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use std::io::Read;

    pub fn getrandom_inner(dest: &mut [u8]) {
        let mut f = std::fs::File::open("/dev/urandom").expect("Unable to open /dev/urandom");
        f.read(dest).expect("Error reading from /dev/urandom into buffer");
    }
}
