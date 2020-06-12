use std::mem::MaybeUninit;
use std::ptr::{null, null_mut};

use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winbase::FormatMessageW;
use winapi::um::winbase::{
    FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS,
};
use winapi::um::winnt::LPWSTR;

pub fn format_winapi_error(error_code: u32) -> String {
    let mut message_buffer = MaybeUninit::<LPWSTR>::uninit();

    unsafe {
        let has_error = FormatMessageW(
            FORMAT_MESSAGE_ALLOCATE_BUFFER
                | FORMAT_MESSAGE_FROM_SYSTEM
                | FORMAT_MESSAGE_IGNORE_INSERTS,
            null(),
            error_code,
            0,
            message_buffer.as_mut_ptr() as *mut u16,
            0,
            null_mut(),
        ) == 0;

        if has_error {
            let new_error_code = GetLastError();
            panic!(
                "error calling FormatMessageW for {}: {}",
                error_code, new_error_code
            );
        }

        string_from_utf16_ptr(message_buffer.assume_init())
    }
}

pub fn vec_from_utf16_ptr(ptr: *const u16) -> Vec<u16> {
    unsafe {
        const NUL: u16 = 0;

        let mut index: usize = 0;
        while *(ptr.add(index)) != NUL {
            index += 1;
        }
        std::slice::from_raw_parts(ptr, index).to_vec()
    }
}

pub fn string_from_utf16_ptr(ptr: *const u16) -> String {
    String::from_utf16_lossy(&vec_from_utf16_ptr(ptr))
}
