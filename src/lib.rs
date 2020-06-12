mod hidsdi;
mod setupapi;
mod util;

use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::AsRawHandle;
use winapi::um::setupapi::HDEVINFO;

pub fn hid_devices() -> HidDeviceIterator {
    let device_info_set = setupapi::get_device_info_set();

    HidDeviceIterator {
        device_info_set,
        device_index: 0,
    }
}

#[derive(Debug)]
pub struct HidDevice {
    pub path: String,
    pub product_id: u16,
    pub vendor_id: u16,
    pub version_number: u16,
    pub manufacturer: Option<String>,
    pub product: Option<String>,
    pub serial_number: Option<String>,
    pub dev_inst: Option<u32>,
    pub pdo_name: Option<String>,
}

#[derive(Debug)]
pub struct HidDeviceIterator {
    device_info_set: HDEVINFO,
    device_index: u32,
}

impl Iterator for HidDeviceIterator {
    type Item = HidDevice;
    fn next(&mut self) -> Option<HidDevice> {
        let mut device_interface_data =
            match setupapi::get_device_interface(self.device_info_set, self.device_index) {
                Some(device_interface_data) => device_interface_data,
                None => return None,
            };

        let mut device_interface_detail =
            setupapi::get_device_interface_detail(self.device_info_set, &mut device_interface_data);

        let pdo_name = setupapi::get_pdo_name(
            self.device_info_set,
            &mut device_interface_detail.device_info,
        );

        let device_interface_path = String::from_utf16_lossy(&device_interface_detail.path);
        let file = std::fs::OpenOptions::new()
            .access_mode(0)
            .open(&device_interface_path)
            .unwrap_or_else(|_| panic!("unable to open device {}", &device_interface_path));

        let device_handle = file.as_raw_handle();

        let attributes = hidsdi::get_attributes(device_handle);
        let manufacturer = hidsdi::get_manufacturer(device_handle);
        let product = hidsdi::get_product(device_handle);
        let serial_number = hidsdi::get_serial_number(device_handle);

        self.device_index += 1;

        Some(HidDevice {
            path: device_interface_path,
            product_id: attributes.ProductID,
            vendor_id: attributes.VendorID,
            version_number: attributes.VersionNumber,
            manufacturer,
            product,
            serial_number,
            dev_inst: Some(device_interface_detail.device_info.DevInst),
            pdo_name,
        })
    }
}
