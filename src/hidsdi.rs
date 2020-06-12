use std::mem::{size_of, MaybeUninit};
use std::os::windows::raw::HANDLE as StdHandle;

use winapi::shared::hidsdi::HIDD_ATTRIBUTES;
use winapi::shared::hidsdi::{
    HidD_GetAttributes, HidD_GetManufacturerString, HidD_GetProductString,
    HidD_GetSerialNumberString,
};
use winapi::shared::minwindef::DWORD;
use winapi::shared::ntdef::FALSE;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winnt::{HANDLE, PVOID, WCHAR};

use super::util;

pub fn get_attributes(device_handle: StdHandle) -> HIDD_ATTRIBUTES {
    let mut attributes = MaybeUninit::<HIDD_ATTRIBUTES>::uninit();

    unsafe {
        let has_error =
            HidD_GetAttributes(device_handle as HANDLE, attributes.as_mut_ptr()) == FALSE;

        if has_error {
            let error_code = GetLastError();
            let error_message = util::format_winapi_error(error_code);
            panic!(
                "error calling HidD_GetAttributes: {} ({})",
                error_message, error_code
            );
        };

        attributes.assume_init()
    }
}

type StringBuffer = [WCHAR; 127]; // character limit set at 126 in docs for HidD_GetManufacturerString, HidD_GetProductString, and HidD_GetSerialNumberString
const STRING_BUFFER_SIZE: usize = size_of::<StringBuffer>();

pub fn get_manufacturer(device_handle: StdHandle) -> Option<String> {
    let mut manufacturer_string: MaybeUninit<StringBuffer> = MaybeUninit::uninit();
    unsafe {
        let has_error = HidD_GetManufacturerString(
            device_handle as HANDLE,
            manufacturer_string.as_mut_ptr() as PVOID,
            STRING_BUFFER_SIZE as DWORD,
        ) == FALSE;

        if has_error {
            let error_code = GetLastError();
            let error_message = util::format_winapi_error(error_code);
            eprintln!(
                "error calling HidD_GetManufacturerString: {} ({})",
                error_message, error_code
            );
            None
        } else {
            let manufacturer_string = manufacturer_string.assume_init();
            Some(util::string_from_utf16_ptr(
                &manufacturer_string as *const u16,
            ))
        }
    }
}

// TODO: refactor to remove code duplication

pub fn get_product(device_handle: StdHandle) -> Option<String> {
    let mut product: MaybeUninit<StringBuffer> = MaybeUninit::uninit();
    unsafe {
        let has_error = HidD_GetProductString(
            device_handle as HANDLE,
            product.as_mut_ptr() as PVOID,
            STRING_BUFFER_SIZE as DWORD,
        ) == FALSE;

        if has_error {
            let error_code = GetLastError();
            let error_message = util::format_winapi_error(error_code);
            eprintln!(
                "error calling HidD_GetProductString: {} ({})",
                error_message, error_code
            );
            None
        } else {
            let product = product.assume_init();
            Some(util::string_from_utf16_ptr(&product as *const u16))
        }
    }
}

pub fn get_serial_number(device_handle: StdHandle) -> Option<String> {
    const ERROR_INVALID_PARAMETER: DWORD = 87;

    let mut serial_number: MaybeUninit<StringBuffer> = MaybeUninit::uninit();
    unsafe {
        let has_error = HidD_GetSerialNumberString(
            device_handle as HANDLE,
            serial_number.as_mut_ptr() as PVOID,
            STRING_BUFFER_SIZE as DWORD,
        ) == FALSE;
        if has_error {
            let error_code = GetLastError();
            if error_code != ERROR_INVALID_PARAMETER {
                let error_message = util::format_winapi_error(error_code);
                eprintln!(
                    "error calling HidD_GetSerialNumberString: {} ({})",
                    error_message, error_code
                );
            }
            None
        } else {
            let serial_number = serial_number.assume_init();
            Some(util::string_from_utf16_ptr(&serial_number as *const u16))
        }
    }
}
