fn main() {
    for device in win_hid_info::hid_devices() {
        println!("{:#?}", device)
    }
}
