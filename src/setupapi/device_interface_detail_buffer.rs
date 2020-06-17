use std::alloc::{alloc_zeroed, dealloc, Layout};
use std::mem::size_of;
use std::path::{Path, PathBuf};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

use winapi::um::setupapi::SP_DEVICE_INTERFACE_DETAIL_DATA_W;

use crate::util::slice_from_utf16_ptr;

/*
    SP_DEVICE_INTERFACE_DETAIL_DATA_W can not account for the string buffer for DevicePath,
    so we must hackily allocate a buffer on the heap with the required size
*/
pub struct DeviceInterfaceDetailBuffer {
    ptr: *mut u8,
    layout: Layout
}

impl DeviceInterfaceDetailBuffer {
    pub fn new(required_size: usize) -> Self {
        let struct_size = size_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>();

        if required_size < struct_size {
            panic!("required_size is smaller than struct fixed size")
        }

        let layout = Layout::from_size_align(
            required_size,
            std::mem::align_of::<SP_DEVICE_INTERFACE_DETAIL_DATA_W>(),
        ).expect("layout error");
    
        let ptr = unsafe { alloc_zeroed(layout) };

        let mut buffer = Self {
            ptr,
            layout
        };

        unsafe {
            let cb_size_ptr: *mut u32 = &mut buffer.as_mut_ref().cbSize;
            cb_size_ptr.write(struct_size as u32);
        }
    
        buffer
    }

    #[allow(clippy::cast_ptr_alignment)]
    pub fn as_mut_ptr(&mut self) -> *mut SP_DEVICE_INTERFACE_DETAIL_DATA_W {
        self.ptr as *mut SP_DEVICE_INTERFACE_DETAIL_DATA_W
    }

    #[allow(clippy::transmute_ptr_to_ref)]
    unsafe fn as_ref(&self) -> &SP_DEVICE_INTERFACE_DETAIL_DATA_W {
        std::mem::transmute(self.ptr)
    }

    #[allow(clippy::transmute_ptr_to_ref)]
    unsafe fn as_mut_ref(&mut self) -> &mut SP_DEVICE_INTERFACE_DETAIL_DATA_W {
        std::mem::transmute(self.ptr)
    }

    pub fn path(&self) -> PathBuf {
        let device_path_ptr = unsafe { &(self.as_ref().DevicePath) as *const u16 };
        let path_utf16_slice = slice_from_utf16_ptr(device_path_ptr);
        let path_os_string = OsString::from_wide(&path_utf16_slice);
        Path::new(&path_os_string).to_path_buf()
    }
}

impl Drop for DeviceInterfaceDetailBuffer {
    fn drop(&mut self) {
        unsafe {
            dealloc(
                self.ptr,
                self.layout,
            );
        }
    }
}