use std::path::Path;

fn add_sources(build: &mut cc::Build, root: &str, files: &[&str]) {
    let root = Path::new(root);
    build.files(files.iter().map(|src| root.join(src)));
}

fn add_includes(build: &mut cc::Build, root: &str, files: &[&str]) {
    build.includes(files.iter().map(|src| format!("{}/{}", root, src)));
}

fn build_cc(target_os: &str) {
    let mut build = cc::Build::new();

    println!("cargo:rerun-if-changed=external/freetype2");

    build
        .warnings(false)
        .include(".")
        .include("external/freetype2/include")
        .define("FT2_BUILD_LIBRARY", None);

    match target_os {
        "linux" => {
            build.flag("-Wno-enum-compare");
        }

        "macos" => {
            build.flag("-Wno-enum-compare");
        }

        _ => {}
    }

    add_sources(
        &mut build,
        "external/freetype2/src",
        &[
            "autofit/autofit.c",
            "base/ftbase.c",
            "base/ftbbox.c",
            "base/ftbdf.c",
            "base/ftbitmap.c",
            "base/ftcid.c",
            "base/ftdebug.c",
            "base/ftfstype.c",
            "base/ftgasp.c",
            "base/ftglyph.c",
            "base/ftgxval.c",
            "base/ftinit.c",
            "base/ftmm.c",
            "base/ftotval.c",
            "base/ftpatent.c",
            "base/ftpfr.c",
            "base/ftstroke.c",
            "base/ftsynth.c",
            "base/ftsystem.c",
            "base/fttype1.c",
            "base/ftwinfnt.c",
            "bdf/bdf.c",
            "bzip2/ftbzip2.c",
            "cache/ftcache.c",
            "cff/cff.c",
            "cid/type1cid.c",
            "gzip/ftgzip.c",
            "lzw/ftlzw.c",
            "pcf/pcf.c",
            "pfr/pfr.c",
            "psaux/psaux.c",
            "pshinter/pshinter.c",
            "psnames/psnames.c",
            "raster/raster.c",
            "sdf/sdf.c",
            "svg/svg.c",
            "sfnt/sfnt.c",
            "smooth/smooth.c",
            "truetype/truetype.c",
            "type1/type1.c",
            "type42/type42.c",
            "winfonts/winfnt.c",
        ],
    );

    build.compile("freetype2");

    let mut build = cc::Build::new();

    build.cpp(true);

    println!("cargo:rerun-if-changed=external/dear-imgui");

    match target_os {
        "linux" => {
            build.flag("-std=c++11");
        }

        "macos" => {
            build.flag("-std=c++11");
        }

        _ => (),
    }

    add_includes(&mut build, "external", &["dear-imgui", "freetype2/include"]);

    add_sources(
        &mut build,
        "external/dear-imgui",
        &[
            "imgui.cpp",
            "imgui_draw.cpp",
            "imgui_tables.cpp",
            "imgui_widgets.cpp",
            "misc/freetype/imgui_freetype.cpp",
        ],
    );

    build.compile("dear-imgui");

    // Build cpp code

    let mut build = cc::Build::new();
    build.cpp(true);

    println!("cargo:rerun-if-changed=src/c_cpp");

    match target_os {
        "linux" => {
            build.flag("-std=c++11");
        }

        "macos" => {
            build.flag("-std=c++11");
        }

        _ => (),
    }

    add_includes(
        &mut build,
        ".",
        &[
            "langs/c_cpp/include",
            "external",
            "external/nanosvg",
            "external/dear-imgui",
            "external/stb",
            "external/freetype2/include",
        ],
    );

    add_sources(&mut build, "src/c_cpp", 
        &[
            "imgui_wrap.cpp",
            "linear_allocator.c",
            "string_allocator.c",
        ],
    );

    build.compile("flowi_cpp");
}

fn main() {
    let os = std::env::var("CARGO_CFG_TARGET_OS").expect("TARGET_OS not specified");
    let target_os = os.as_str();

    println!("cargo:rerun-if-changed=external");
    println!("cargo:rerun-if-changed=c_cpp");

    build_cc(target_os);
}
