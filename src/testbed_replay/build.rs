fn main() {
    println!("cargo:rustc-link-search=../../external/libinput");
    println!("cargo:rustc-link-search=../../external/libudev");
    println!("cargo:rustc-link-search=../../external/mali");
    println!("cargo:rustc-link-lib=udev");
}
