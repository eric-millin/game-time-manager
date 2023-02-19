use embed_manifest::{
    embed_manifest, embed_manifest_file, manifest::AssemblyIdentity, new_manifest,
};

fn main() {
    let assembly = AssemblyIdentity::new(
        "Microsoft.Windows.Common-Controls",
        [6, 0, 0, 0],
        0x6595b64144ccf1df,
    );
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        embed_manifest_file("./gamer-gauge.exe.manifest").expect("unable to embed manifest file");
        // embed_manifest(new_manifest("PastTime").dependency(assembly))
        //     .expect("unable to embed manifest file");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
