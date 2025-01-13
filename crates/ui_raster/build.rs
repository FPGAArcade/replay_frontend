fn main() {
    /*
    use ispc_compile::TargetISA;

    #[cfg(target_arch = "x86_64")]
    let target_isas = vec![TargetISA::SSE4i16x8];

    #[cfg(target_arch = "aarch64")]
    let target_isas = vec![TargetISA::Neoni32x4];

    let bindgen_builder = ispc_compile::bindgen::builder()
        .allowlist_function("ispc_raster_rectangle_solid_lerp_color")
        .allowlist_function("ispc_raster_texture_aligned");

    ispc_compile::Config::new()
        .file("src/ispc_raster.ispc")
        .target_isas(target_isas)
        .enable_llvm_intrinsics()
        .bindgen_builder(bindgen_builder)
        .compile("ui_raster");
    */
}
