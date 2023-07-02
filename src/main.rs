#![allow(clippy::needless_range_loop)]

mod chip8;

use chip8::Chip8;

use std::time::Instant;

use kira::manager::backend::DefaultBackend;
use kira::manager::AudioManager;
use kira::sound::static_sound::StaticSoundData;
use pixels::{Pixels, SurfaceTexture};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

const CYCLE_HZ: usize = 1000;
const TIMER_REG_HZ: usize = 60;

const INPUT_KEYS: [VirtualKeyCode; chip8::INPUT_COUNT] = [
    VirtualKeyCode::Key1,
    VirtualKeyCode::Key2,
    VirtualKeyCode::Key3,
    VirtualKeyCode::Key4,
    VirtualKeyCode::Q,
    VirtualKeyCode::W,
    VirtualKeyCode::E,
    VirtualKeyCode::R,
    VirtualKeyCode::A,
    VirtualKeyCode::S,
    VirtualKeyCode::D,
    VirtualKeyCode::F,
    VirtualKeyCode::Z,
    VirtualKeyCode::X,
    VirtualKeyCode::C,
    VirtualKeyCode::V,
];

const FOREGROUND_COLOR: [u8; 4] = [230, 40, 55, 255];
const BACKGROUND_COLOR: [u8; 4] = [0, 0, 0, 255];

fn main() {
    let mut c8 = Chip8::new();

    if let Some(rom_path) = std::env::args().collect::<Vec<String>>().get(1) {
        let rom = std::fs::read(rom_path).expect("could not read input ROM file");
        c8.load(&rom);
    }

    let mut input = [false; chip8::INPUT_COUNT];
    let mut output = [[false; chip8::DISPLAY_HEIGHT]; chip8::DISPLAY_WIDTH];

    let event_loop = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("rusty8")
        .build(&event_loop)
        .expect("failed to create window");

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(
            chip8::DISPLAY_WIDTH as u32,
            chip8::DISPLAY_HEIGHT as u32,
            surface_texture,
        )
        .expect("failed to initialise pixels")
    };

    let mut audio_manager = AudioManager::<DefaultBackend>::new(Default::default())
        .expect("failed to initialise the audio manager");
    let buzz_sound = StaticSoundData::from_file("buzz.wav", Default::default())
        .expect("failed to load buzz sound");

    let mut last_instant = Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::MainEventsCleared => {
            let now = Instant::now();
            let delta = (now - last_instant).as_secs_f32();
            last_instant = now;

            update(
                delta,
                &mut c8,
                &mut audio_manager,
                buzz_sound.clone(),
                &input,
                &mut output,
            );

            window.request_redraw();
        }

        Event::RedrawRequested(window_id) if window_id == window.id() => {
            draw(&mut pixels, &output);
        }

        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => {
                *control_flow = ControlFlow::Exit;
            }

            WindowEvent::Resized(size) => {
                pixels.resize_surface(size.width, size.height).unwrap();
            }

            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    },
                ..
            } => {
                handle_input(key, state, &mut input);
            }

            _ => {}
        },

        _ => {}
    });
}

fn update(
    delta: f32,
    c8: &mut Chip8,
    audio_manager: &mut AudioManager<DefaultBackend>,
    buzz_sound: StaticSoundData,
    input: &[bool; chip8::INPUT_COUNT],
    output: &mut [[bool; chip8::DISPLAY_HEIGHT]; chip8::DISPLAY_WIDTH],
) {
    let mut play_sound = false;
    for _ in 0..(delta * TIMER_REG_HZ as f32).round() as usize {
        play_sound = c8.step_timers() || play_sound;
    }

    if play_sound {
        audio_manager.play(buzz_sound).unwrap();
    }

    for _ in 0..(delta * CYCLE_HZ as f32).round() as usize {
        c8.step(input, output);
    }
}

fn draw(pixels: &mut Pixels, output: &[[bool; chip8::DISPLAY_HEIGHT]; chip8::DISPLAY_WIDTH]) {
    for (i, pixel) in pixels.frame_mut().chunks_exact_mut(4).enumerate() {
        let x = i % chip8::DISPLAY_WIDTH;
        let y = i / chip8::DISPLAY_WIDTH;

        let rgba = if output[x][y] {
            FOREGROUND_COLOR
        } else {
            BACKGROUND_COLOR
        };

        pixel.copy_from_slice(&rgba);
    }

    pixels.render().unwrap();
}

fn handle_input(
    key: &VirtualKeyCode,
    state: &ElementState,
    input: &mut [bool; chip8::INPUT_COUNT],
) {
    if let Some(key_index) = INPUT_KEYS.iter().position(|x| x == key) {
        let down = matches!(state, ElementState::Pressed);
        input[key_index] = down;
    }
}
