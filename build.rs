use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    slint_build::compile("ui/main_window.slint").expect("Failed to compile Slint UI");

    // Pre-decode PNG icon to raw ARGB32 bytes at build time so the runtime binary doesn't need the image crate.
    // Resize to 64x64 (standard high-DPI tray icon resolution) to save ~245 KB in binary size.
    let img = image::open("assets/icon.png").expect("Failed to open assets/icon.png");
    let img = img.resize_exact(64, 64, image::imageops::FilterType::Lanczos3);
    let (width, height) = (img.width(), img.height());
    let mut data = img.into_rgba8().into_vec();
    for pixel in data.as_chunks_mut::<4>().0 {
        pixel.rotate_right(1); // [R, G, B, A] -> [A, R, G, B] network byte order for StatusNotifierItem
    }
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out_dir.join("tray_icon.bin"), &data).expect("Failed to write tray_icon.bin");
    fs::write(
        out_dir.join("tray_icon_meta.rs"),
        format!("pub const ICON_WIDTH: i32 = {width};\npub const ICON_HEIGHT: i32 = {height};\n"),
    )
    .expect("Failed to write tray_icon_meta.rs");
}
