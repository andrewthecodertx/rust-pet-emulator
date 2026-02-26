# Agent Instructions for rust-pet-emulator

This is a Commodore PET 4032 emulator written in Rust.

## Build Commands

```bash
# Build the project
cargo build

# Build in release mode
cargo build --release

# Run the emulator
cargo run --release

# Check for compilation errors without building
cargo check
```

## Test Commands

```bash
# Run all tests
cargo test

# Run a specific test
cargo test <test_name>

# Run tests with output
cargo test -- --nocapture

# Run tests with verbose output
cargo test -- --nocapture --test-threads=1
```

## Lint Commands

```bash
# Run clippy
cargo clippy

# Run clippy with all features
cargo clippy --all-features

# Check formatting
cargo fmt -- --check

# Format code
cargo fmt
```

## Code Style Guidelines

### No Comments
- Do not add comments in the code
- Code should be self-documenting through clear naming

### Markdown Formatting
- Max line length: 80 characters
- Add blank lines where appropriate for readability
- Follow standard markdown linting rules

### README Updates
- Update README.md alongside any code changes
- Keep documentation synchronized with implementation

### Imports
- Group imports: std, external crates, then internal modules
- Separate import groups with blank lines
- Use crate-level imports: `use crate::bus::PetBus`
- External crate imports first: `use mos6502::cpu::Cpu`

Example:
```rust
use sdl2::event::Event;
use std::time::{Duration, Instant};

mod bus;
mod crtc6845;
use crate::bus::PetBus;
use mos6502::cpu::Cpu;
```

### Rust Formatting
- Use `cargo fmt` for formatting
- Use 4 spaces for indentation
- No trailing whitespace

### Naming Conventions
- Types: PascalCase (e.g., `PetBus`, `Crtc6845`, `Pia6821`)
- Functions/Methods: snake_case (e.g., `read_register`, `set_key`)
- Constants: UPPER_SNAKE_CASE
- Modules: snake_case
- Files match module names (e.g., `bus.rs`, `pia6821.rs`)

### Types and Error Handling
- Prefer `Result<T, E>` for fallible operations
- Use `?` operator for error propagation
- Error type: `Box<dyn std::error::Error>`
- Avoid `unwrap()` in production code
- Use `u8`, `u16`, `u32`, `u64` for specific bit widths

### Struct Definitions
- Group related fields
- Use public fields with `pub` when external access needed
- Document struct purpose

Example:
```rust
pub struct PetBus {
    pub ram: [u8; 0x8800],
    pub roms: RomData,
    pub via: Via6522,
    pub pia: Pia6821,
    pub crtc: Crtc6845,
    pub irq_asserted: bool,
    pub total_cycles: u64,
}
```

### Trait Implementations
- Keep trait impls close to struct definition
- Prefer `impl TraitName for TypeName` pattern

Example:
```rust
impl CpuBus for PetBus {
    fn read(&mut self, addr: u16) -> u8 { ... }
    fn write(&mut self, addr: u16, val: u8) { ... }
}
```

### Architecture
This is an emulator project with hardware components:
- `main.rs`: Entry point, SDL2 rendering loop
- `bus.rs`: Memory bus and address mapping
- `pia6821.rs`: Peripheral Interface Adapter (keyboard)
- `crtc6845.rs`: CRT Controller
- `via6522.rs`: Versatile Interface Adapter
- `renderer.rs`: Screen rendering logic
- `rom_loader.rs`: ROM file loading
- `file_dialog.rs`: File browser for loading .prg files

### Key Patterns
- Hardware components are structs with tick() methods
- Memory-mapped I/O via Bus trait
- SDL2 for graphics and input
- External dependency: mos6502 CPU from `../rust-6502-emulator`

### Dependencies
- `bitflags = "2.4"` - Flag/bitfield handling
- `sdl2 = { version = "0.38", features = ["ttf"] }` - Graphics library

### ROM Files
- Located in `roms/` directory
- Binary files for system ROMs
- Do not modify ROM files

## Testing

The project currently has minimal tests. When adding tests:
- Place unit tests in `#[cfg(test)]` modules at file end
- Use descriptive test names
- Test edge cases for hardware emulation

## Project Structure

```
rust-pet-emulator/
├── Cargo.toml          # Package manifest
├── Cargo.lock          # Dependency lock file
├── src/
│   ├── main.rs         # Application entry
│   ├── bus.rs          # Memory bus
│   ├── crtc6845.rs     # CRT controller
│   ├── pia6821.rs      # PIA (keyboard)
│   ├── via6522.rs      # VIA
│   ├── renderer.rs     # Screen drawing
│   └── rom_loader.rs   # ROM loading
├── roms/               # System ROM files
└── target/             # Build output
```
