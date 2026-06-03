use std::{env::var_os, fs::read_dir, path::PathBuf};

use bindgen::{EnumVariation, builder};

fn main() {
    let base = PathBuf::from("./source");
    builder()
        .headers(read_dir(base).unwrap().filter_map(|f| {
            if let Ok(f) = f
                && let Ok(ft) = f.file_type()
                && ft.is_file()
                && let Ok(fpath) = f.path().as_path().canonicalize()
                && let Some(ext) = fpath.extension()
                && let Some(ext) = ext.to_str()
                && (ext == "c" || ext == "h")
            {
                return Some(fpath.to_string_lossy().to_string());
            }
            None
        }))
        .default_enum_style(EnumVariation::Rust {
            non_exhaustive: false,
        })
        .size_t_is_usize(true)
        .allowlist_type("^_HTS.*")
        .allowlist_function("^HTS.*")
        .layout_tests(false)
        .generate_comments(false)
        .wrap_static_fns(true)
        .generate()
        .unwrap()
        .write_to_file(PathBuf::from(var_os("OUT_DIR").unwrap()).join("hts_engine_bindings.rs"))
        .unwrap();
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-search=/opt/homebrew/opt/open-jtalk/lib");
    #[cfg(target_os = "linux")]
    {
        #[cfg(target_arch = "x86_64")]
        println!("cargo:rustc-link-search=/usr/lib/x86_64-linux-gnu");
        #[cfg(target_arch = "aarch64")]
        println!("cargo:rustc-link-search=/usr/lib/aarch64-linux-gnu");
    }
    println!("cargo:rustc-link-lib=HTSEngine");
}
