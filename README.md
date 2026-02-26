# Commodore PET 4032 Emulator

A Rust-based emulator for the Commodore PET 4032 computer.

## Requirements

- Rust toolchain (1.80+)
- SDL2 development libraries
- MOS 6502 emulator crate (sibling project)

### System Dependencies

**Ubuntu/Debian:**

```bash
sudo apt install libsdl2-dev libsdl2-ttf-dev
```

**Arch Linux:**

```bash
sudo pacman -S sdl2 sdl2_ttf
```

**macOS:**

```bash
brew install sdl2 sdl2_ttf
```

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run --release
```

Or run the built binary directly:

```bash
./target/release/pet4032
```

## Controls

- Use your keyboard to type on the virtual PET keyboard
- Press `Escape` to exit the emulator
- Press `F2` to open the file dialog for loading .prg files

### Loading .prg Files

The emulator can load Commodore PET .prg files (BASIC programs and machine
code). Place your .prg files in the `./software` directory, then:

1. Press `F2` to open the file browser
2. Use `Up/Down` arrow keys to navigate
3. Press `Enter` to select and load a .prg file
4. Press `Backspace` to navigate up to parent directory
5. Press `F2` to close without selecting

When you select a file, it will be loaded into memory and "RUN" will be
typed automatically to start the program.

Software for the PET is available at:
https://zimmers.net/anonftp/pub/cbm/pet/

## ROMs

The emulator requires ROM files which should be placed in the
`roms/` directory:

- `basic-4-b000.901465-19.bin`
- `basic-4-c000.901465-20.bin`
- `basic-4-d000.901465-21.bin`
- `kernal-4.901465-22.bin`
- `edit-4-40-n-60Hz.901499-01.bin`
- `characters-2.901447-10.bin`

These ROM files are bundled with the project and should already be
present in the `roms/` directory.
