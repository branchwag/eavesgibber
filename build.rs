use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let ggwave_dir = out_dir.join("ggwave");

    // Clone ggwave if not present
    if !ggwave_dir.exists() {
        println!("cargo:warning=Cloning ggwave repository...");
        let status = Command::new("git")
            .args(["clone", "--depth", "1", "https://github.com/ggerganov/ggwave.git"])
            .arg(&ggwave_dir)
            .status()
            .expect("Failed to clone ggwave");

        if !status.success() {
            panic!("Failed to clone ggwave repository");
        }
    }

    // Build ggwave
    println!("cargo:warning=Building ggwave...");
    cc::Build::new()
        .cpp(true)
        .file(ggwave_dir.join("src/ggwave.cpp"))
        .include(ggwave_dir.join("include"))
        .flag("-std=c++11")
        .flag("-O3")
        .compile("ggwave");

    // Generate bindings
    println!("cargo:warning=Generating bindings...");
    let bindings = bindgen::Builder::default()
        .header(ggwave_dir.join("include/ggwave/ggwave.h").to_str().unwrap())
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=build.rs");
}
