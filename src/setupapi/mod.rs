mod device_interface_detail_buffer;

use std::mem::{size_of, MaybeUninit};
use std::ptr::{null, null_mut};

use winapi::shared::hidclass::GUID_DEVINTERFACE_HID;
use winapi::shared::minwindef::{DWORD, FALSE};

use winapi::um::errhandlingapi::GetLastError;
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::setupapi::{
    SetupDiEnumDeviceInterfaces, SetupDiGetClassDevsW, SetupDiGetDeviceInterfaceDetailW,
    SetupDiGetDeviceRegistryPropertyW,
};
use winapi::um::setupapi::{
    DIGCF_DEVICEINTERFACE, DIGCF_PRESENT, HDEVINFO, SPDRP_PHYSICAL_DEVICE_OBJECT_NAME,
    SP_DEVICE_INTERFACE_DATA, SP_DEVINFO_DATA,
};
use winapi::um::winnt::WCHAR;

use super::util;
use device_interface_detail_buffer::DeviceInterfaceDetailBuffer;

pub fn get_device_info_set() -> HDEVINFO {
    unsafe {
        let handle = SetupDiGetClassDevsW(
            &GUID_DEVINTERFACE_HID,
            null(),
            null_mut(),
            DIGCF_PRESENT | DIGCF_DEVICEINTERFACE,
        );

        if handle == INVALID_HANDLE_VALUE {
            let error_code = GetLastError();
            let error_message = util::format_winapi_error(error_code);
            panic!(
                "error calling SetupDiGetClassDevsW: {} ({})",
                error_message, error_code
            );
        }

        handle
    }
}

pub fn get_device_interface(
    device_info_set: HDEVINFO,
    device_index: DWORD,
) -> Option<SP_DEVICE_INTERFACE_DATA> {
    const ERROR_NO_MORE_ITEMS: DWORD = 259;

    let mut device_interface_data = MaybeUninit::<SP_DEVICE_INTERFACE_DATA>::zeroed();

    unsafe {
        (*device_interface_data.as_mut_ptr()).cbSize =
            size_of::<SP_DEVICE_INTERFACE_DATA>() as DWORD;

        let has_error = SetupDiEnumDeviceInterfaces(
            device_info_set,
            null_mut(),
            &GUID_DEVINTERFACE_HID,
            device_index,
            device_interface_data.as_mut_ptr(),
        ) == FALSE;

        if has_error {
            let error_code = GetLastError();
            if error_code == ERROR_NO_MORE_ITEMS {
                return None;
            } else {
                let error_message = util::format_winapi_error(error_code);
                panic!(
                    "error calling SetupDiEnumDeviceInterfaces: {} ({})",
                    error_message, error_code
                );
            }
        };
        Some(device_interface_data.assume_init())
    }
}

pub fn get_pdo_name(
    device_info_set: HDEVINFO,
    device_info_data: &mut SP_DEVINFO_DATA,
) -> Option<String> {
    // TODO: get required size by doing two calls
    type StringBuffer = [WCHAR; 1024]; // guess buffer size
    const STRING_BUFFER_SIZE: usize = size_of::<StringBuffer>();

    unsafe {
        let mut reg_data_type: DWORD = 0;

        let mut pdo_name: MaybeUninit<StringBuffer> = MaybeUninit::uninit();

        let has_error = SetupDiGetDeviceRegistryPropertyW(
            device_info_set,
            device_info_data,
            SPDRP_PHYSICAL_DEVICE_OBJECT_NAME,
            &mut reg_data_type,
            pdo_name.as_mut_ptr() as *mut u8,
            STRING_BUFFER_SIZE as DWORD,
            null_mut(),
        ) == FALSE;

        let pdo_name = pdo_name.assume_init();

        if has_error {
            let error_code = GetLastError();
            let error_message = util::format_winapi_error(error_code);
            eprintln!(
                "error calling SetupDiEnumDeviceInterfaces: {} ({})",
                error_message, error_code
            );
            None
        } else {
            Some(util::string_from_utf16_ptr(&pdo_name as *const u16))
        }
    }
}

pub struct DeviceInterfaceDetail {
    pub path: std::path::PathBuf,
    pub device_info: SP_DEVINFO_DATA,
}

pub fn get_device_interface_detail(
    device_info_set: HDEVINFO,
    device_interface_data: &mut SP_DEVICE_INTERFACE_DATA,
) -> DeviceInterfaceDetail {
    const ERROR_INSUFFICIENT_BUFFER: DWORD = 122;

    let mut required_size: DWORD = 0;

    // Get required_size so that we know how much buffer space to allocate
    unsafe {
        let has_error = SetupDiGetDeviceInterfaceDetailW(
            device_info_set,
            device_interface_data,
            null_mut(),
            0,
            &mut required_size,
            null_mut(),
        ) == FALSE;

        if !has_error {
            panic!("something went wrong, this should fail due to not providing buffer");
        } else {
            let error_code = GetLastError();
            let error_message = util::format_winapi_error(error_code);
            if error_code != ERROR_INSUFFICIENT_BUFFER {
                eprintln!(
                    "error calling SetupDiGetDeviceInterfaceDetailW: {} ({})",
                    error_message, error_code
                )
            }
        }
    }

    let mut device_interface_detail_buffer = DeviceInterfaceDetailBuffer::new(required_size as usize);

    let mut device_info_data = MaybeUninit::<SP_DEVINFO_DATA>::zeroed();

    unsafe {
        (*device_info_data.as_mut_ptr()).cbSize = size_of::<SP_DEVINFO_DATA>() as DWORD;

        let has_error = SetupDiGetDeviceInterfaceDetailW(
            device_info_set,
            device_interface_data,
            device_interface_detail_buffer.as_mut_ptr(),
            required_size,
            null_mut(),
            device_info_data.as_mut_ptr(),
        ) == FALSE;

        if has_error {
            let error_code = GetLastError();
            let error_message = util::format_winapi_error(error_code);
            panic!(
                "error calling SetupDiGetDeviceInterfaceDetailW: {} ({})",
                error_message, error_code
            );
        };

        DeviceInterfaceDetail {
            path: device_interface_detail_buffer.path(),
            device_info: device_info_data.assume_init(),
        }
    }
}
