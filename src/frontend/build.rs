use std::env;

fn main() {
    let tundra_dir = env::var("TUNDRA_OBJECTDIR").unwrap_or("".to_string());
    let libs = env::var("TUNDRA_STATIC_LIBS").unwrap_or("".to_string());
    let target = env::var("TARGET").expect("TARGET was not set");

    let native_libs = libs.split(" ");

    println!("cargo:rustc-link-search=native={}", tundra_dir);

    for lib in native_libs {
        println!("cargo:rustc-link-lib=static={}", lib);
        println!("cargo:rerun-if-changed={}", lib);
    }

    if target.contains("linux") {
        println!("cargo:rustc-link-lib=static=stdc++");
    } else if target.contains("apple") {
        println!("cargo:rustc-link-lib=static=c++");
    }
}
