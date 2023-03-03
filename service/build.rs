use copy_to_output::copy_to_output;
use embed_manifest::embed_manifest_file;
use std::env;

fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        embed_manifest_file("./game-time-manager.exe.manifest")
            .expect("unable to embed manifest file");
    }

    let proj_root = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    copy_to_output(
        &format!("{}/src/config.toml", proj_root),
        &env::var("PROFILE").unwrap(),
    )
    .expect("Could not copy");

    copy_to_output(
        &format!("{}/src/install.ps1", proj_root),
        &env::var("PROFILE").unwrap(),
    )
    .expect("Could not copy");

    copy_to_output(
        &format!("{}/src/uninstall.ps1", proj_root),
        &env::var("PROFILE").unwrap(),
    )
    .expect("Could not copy");

    println!("cargo:rerun-if-changed=build.rs");
}
