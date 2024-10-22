#[cfg(feature = "ispc")]
fn compile_kernel() {
        use ispc_compile::{bindgen::builder, Config, TargetISA};

    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_isas = match target_arch.as_str() {
        "x86" | "x86_64" => vec![
            TargetISA::SSE2i32x4,
            TargetISA::SSE4i32x4,
            TargetISA::AVX1i32x8,
            TargetISA::AVX2i32x8,
        ],
        "arm" | "aarch64" => vec![
            // TargetISA::Neoni32x4,
            TargetISA::Neoni32x8,
        ],
        x => panic!("Unsupported target architecture {}", x),
    };

    Config::new()
        .opt_level(2)
        .woff()
        .target_isas(target_isas.clone())
        .out_dir("src/ispc")
        .file("src/ispc/x64_kernel.ispc")
        .bindgen_builder(builder())
        .compile("kernel");
}

#[cfg(not(feature = "ispc"))]
fn compile_kernel() {

}

fn main() {
    compile_kernel();
}
