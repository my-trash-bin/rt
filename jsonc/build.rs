use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=native/jsonc.c");
    println!("cargo:rerun-if-changed=include/jsonc.h");

    cc::Build::new()
        .include("include")
        .file("native/jsonc.c")
        .flag_if_supported("-std=c99")
        .compile("jsonc");

    let bindings = bindgen::Builder::default()
        .header("include/jsonc.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
