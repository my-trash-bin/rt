fn main() {
    println!("cargo:rerun-if-changed=native/jsonc.c");
    println!("cargo:rerun-if-changed=include/jsonc.h");

    cc::Build::new()
        .include("include")
        .file("native/jsonc.c")
        .flag_if_supported("-std=c99")
        .compile("jsonc");
}
