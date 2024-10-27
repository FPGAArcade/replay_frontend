#[cfg(feature = "ispc")]
fn link_or_compile_kernel() {
    use ispc_compile::{bindgen::builder, Config, TargetISA};

    #[cfg(target_arch = "x86_64")]
    let target_isas = vec![
        TargetISA::SSE2i32x4,
        TargetISA::SSE4i32x4,
    ];

    #[cfg(target_arch = "aarch64")]
    let target_isas = vec![TargetISA::Neoni32x4];

    let bindgen_builder = ispc_compile::bindgen::builder()
        .allowlist_function("add_values")
        .allowlist_function("draw_rects");

    ispc_compile::Config::new()
        .file("src/ispc/x64_kernel.ispc")
        .target_isas(target_isas)
        .bindgen_builder(bindgen_builder)
        .out_dir("src/ispc")
        .compile("kernel");
}

#[cfg(not(feature = "ispc"))]
fn link_or_compile_kernel() {
    ispc_rt::PackagedModule::new("kernel")
        .lib_path("src/ispc")
        .link();
}

fn main() {
    link_or_compile_kernel();
}
