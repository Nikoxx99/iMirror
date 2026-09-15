fn main() {
    let phone_assets = std::path::Path::new("phone-assets");
    if !phone_assets.exists() {
        std::fs::create_dir_all(phone_assets).expect("create embedded phone asset directory");
        std::fs::write(
            phone_assets.join("index.html"),
            "<!doctype html><title>iMirror</title><p>Build the phone app before packaging iMirror.</p>",
        )
        .expect("create embedded phone placeholder");
    }
    println!("cargo:rerun-if-changed=phone-assets");
    tauri_build::build();
}
