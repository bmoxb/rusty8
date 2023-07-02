# rusty8

`rusty8` is a simple CHIP-8 emulator implemented in Rust with [pixels](https://github.com/parasyte/pixels) used for rendering and [kira](https://github.com/tesselode/kira) for sound.

The code is designed to be simple and succinct. The full source code is across just 2 files with <500 lines of code total excluding comments and whitespace.

Source file `chip8.rs` contains the emulator implementation while `main.rs` handles rendering, sound, and input/output.

## Usage

The path of the ROM to execute should be passed as the only command-line argument to the emulator.

The following keyboard keys are passed as input to running games: 1, 2, 3, 4, Q, W, E, R, A, S, D, F, Z, X, C, V.

The emulator is exited by simply closing the game window.
