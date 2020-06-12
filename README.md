Lists HID devices on Windows OSes

### Usage

``` shell
$ cargo run --bin list_hid_devices

HidDevice {
    path: "\\\\?\\hid#vid_046d&pid_c52b&mi_01&col01#8&34e82c22&0&0000#{4d1e55b2-f16f-11cf-88cb-001111000030}",
    product_id: 50475,
    vendor_id: 1133,
    version_number: 4609,
    manufacturer: Some(
        "Logitech",
    ),
    product: Some(
        "USB Receiver",
    ),
    serial_number: None,
    dev_inst: Some(
        1,
    ),
    pdo_name: Some(
        "\\Device\\0000002f",
    ),
}
```