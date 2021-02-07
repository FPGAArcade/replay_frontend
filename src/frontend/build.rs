use std::env;

fn main() {
    let target = env::var("TARGET").expect("TARGET was not set");

    if target.contains("linux") {
        println!("cargo:rustc-link-lib=static=stdc++");
        println!("cargo:rustc-link-lib=dylib=GL");
        println!("cargo:rustc-link-lib=dylib=dl");
        println!("cargo:rustc-link-lib=dylib=X11");
        println!("cargo:rustc-link-lib=dylib=pthread");
    } else if target.contains("apple") {
        println!("cargo:rustc-link-lib=static=c++");
    }
}
