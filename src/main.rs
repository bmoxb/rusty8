mod chip8;

use macroquad::prelude as quad;

#[macroquad::main("Rusty8")]
async fn main() {
    let mut c8 = chip8::Chip8::new();

    if let Some(rom_path) = std::env::args().next() {
        let rom = std::fs::read(rom_path).expect("Could not read input ROM file: {rom_path}");
        c8.load(&rom);
    }

    loop {
        quad::clear_background(quad::RED);
        quad::next_frame().await
    }
}
