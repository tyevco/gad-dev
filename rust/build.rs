use std::process::Command;
use std::env;
use std::path::Path;

fn main() {
    // Run the default build
    Command::new("cargo")
        .args(&["build", "--release"])
        .status()
        .expect("Failed to build project");

    // Get the output directory
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);
    let out_dir = out_dir.ancestors().nth(3).unwrap();

    // Construct the source path
    let src_path = out_dir.join("libgodot_asset_browser.so");

    // Construct the destination path
    // Update this path to match your Godot project structure
    let dest_path = Path::new("../godot/addons/godot_asset_browser/libgodot_asset_browser.so");

    // Copy the file
    std::fs::copy(&src_path, &dest_path)
        .expect("Failed to copy .so file");

    println!("cargo:rerun-if-changed=src");
}