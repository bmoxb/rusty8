mod chip8;

use macroquad::prelude as quad;

#[macroquad::main("Rusty8")]
async fn main() {
    let mut c8 = chip8::Chip8::new();

    loop {
        quad::clear_background(quad::RED);
        quad::next_frame().await
    }
}
