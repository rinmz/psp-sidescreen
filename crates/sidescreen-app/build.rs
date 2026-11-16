//! Lets Tauri generate what the program needs at build time: the window manifest, the icon and
//! the permission files.

fn main() {
    tauri_build::build();
}
