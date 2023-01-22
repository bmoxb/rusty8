#![allow(clippy::needless_range_loop)]

mod chip8;

use macroquad::audio;
use macroquad::prelude as quad;

const CYCLE_HZ: usize = 1000;
const TIMER_REG_HZ: usize = 60;

const INPUT_KEYS: [quad::KeyCode; 16] = [
    quad::KeyCode::Key1,
    quad::KeyCode::Key2,
    quad::KeyCode::Key3,
    quad::KeyCode::Key4,
    quad::KeyCode::Q,
    quad::KeyCode::W,
    quad::KeyCode::E,
    quad::KeyCode::R,
    quad::KeyCode::A,
    quad::KeyCode::S,
    quad::KeyCode::D,
    quad::KeyCode::F,
    quad::KeyCode::Z,
    quad::KeyCode::X,
    quad::KeyCode::C,
    quad::KeyCode::V,
];

#[macroquad::main("Rusty8")]
async fn main() {
    let buzz = audio::load_sound("buzz.wav")
        .await
        .expect("could not load buzz sound effect");

    let mut c8 = chip8::Chip8::new();

    if let Some(rom_path) = std::env::args().collect::<Vec<String>>().get(1) {
        let rom = std::fs::read(rom_path).expect("could not read input ROM file");
        c8.load(&rom);
    }

    let mut input = [false; 16];
    let mut output = [[false; chip8::DISPLAY_HEIGHT]; chip8::DISPLAY_WIDTH];

    loop {
        quad::clear_background(quad::BLACK);

        for key in 0..16 {
            input[key] = quad::is_key_down(INPUT_KEYS[key]);
        }

        let mut play_buzz = false;
        for _ in 0..(quad::get_frame_time() * TIMER_REG_HZ as f32).round() as usize {
            play_buzz = play_buzz || c8.step_timers();
        }

        if play_buzz {
            let params = audio::PlaySoundParams {
                looped: true,
                volume: 1.0,
            };
            audio::play_sound(buzz, params);
        } else {
            audio::stop_sound(buzz);
        }

        for _ in 0..(quad::get_frame_time() * CYCLE_HZ as f32).round() as usize {
            c8.step(&input, &mut output);
        }

        draw_output(&output);

        quad::next_frame().await
    }
}

fn draw_output(output: &[[bool; chip8::DISPLAY_HEIGHT]; chip8::DISPLAY_WIDTH]) {
    let pixel_width = quad::screen_width() / chip8::DISPLAY_WIDTH as f32;
    let pixel_height = quad::screen_height() / chip8::DISPLAY_HEIGHT as f32;

    for x in 0..chip8::DISPLAY_WIDTH {
        for y in 0..chip8::DISPLAY_HEIGHT {
            if output[x][y] {
                let draw_x = x as f32 * pixel_width;
                let draw_y = y as f32 * pixel_height;
                quad::draw_rectangle(draw_x, draw_y, pixel_width, pixel_height, quad::RED);
            }
        }
    }
}
