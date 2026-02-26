use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::{Duration, Instant};

mod bus;
mod crtc6845;
mod file_dialog;
mod pia6821;
mod renderer;
mod rom_loader;
mod via6522;

use crate::bus::PetBus;
use crate::file_dialog::{load_prg_file, FileDialog};
use mos6502::bus::Bus;
use mos6502::cpu::Cpu;
use renderer::{draw_file_dialog, draw_pet_screen};
use rom_loader::load_roms;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let ttf_context = sdl2::ttf::init()?;
    let window = video_subsystem
        .window("Commodore PET 4032", 640, 400)
        .position_centered()
        .build()?;

    let mut canvas = window.into_canvas().build()?;
    let mut event_pump = sdl_context.event_pump()?;

    let roms = load_roms()?;
    let bus_instance = PetBus::new(roms);
    let mut cpu = Cpu::new(bus_instance);

    cpu.reset();

    let mut last_frame = Instant::now();
    let cycles_per_frame = 16666;
    let mut file_dialog = FileDialog::new("./software");

    'running: loop {
        for event in event_pump.poll_iter() {
            if file_dialog.is_visible() {
                match event {
                    Event::Quit { .. } => break 'running,
                    Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    Event::KeyDown {
                        keycode: Some(Keycode::F2),
                        ..
                    } => {
                        file_dialog.hide();
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Up),
                        ..
                    } => {
                        file_dialog.move_selection_up();
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Down),
                        ..
                    } => {
                        file_dialog.move_selection_down();
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Return),
                        ..
                    } => {
                        if let Some(path) = file_dialog.select_current() {
                            if let Ok((load_addr, data)) = load_prg_file(&path) {
                                for (i, byte) in data.iter().enumerate() {
                                    let addr = load_addr.wrapping_add(i as u16);
                                    cpu.bus.write(addr, *byte);
                                }
                                let end_addr = load_addr.wrapping_add(data.len() as u16);
                                cpu.bus.write(0x0028, (load_addr & 0xFF) as u8);
                                cpu.bus.write(0x0029, (load_addr >> 8) as u8);
                                cpu.bus.write(0x002A, (end_addr & 0xFF) as u8);
                                cpu.bus.write(0x002B, (end_addr >> 8) as u8);
                                let run_keys = [(7, 2), (2, 3), (3, 1), (6, 5)];
                                cpu.bus.pia.auto_type(&run_keys);
                            }
                        }
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Backspace),
                        ..
                    } => {
                        file_dialog.go_up();
                    }
                    Event::KeyDown {
                        keycode: Some(key), ..
                    } => {
                        if let Some((row, col)) = keycode_to_pet_matrix(key) {
                            cpu.bus.pia.set_key(row, col, true);
                        }
                    }
                    Event::KeyUp {
                        keycode: Some(key), ..
                    } => {
                        if let Some((row, col)) = keycode_to_pet_matrix(key) {
                            cpu.bus.pia.set_key(row, col, false);
                        }
                    }
                    _ => {}
                }
            } else {
                match event {
                    Event::Quit { .. } => break 'running,
                    Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    Event::KeyDown {
                        keycode: Some(Keycode::F2),
                        ..
                    } => {
                        file_dialog.show();
                    }
                    Event::KeyDown {
                        keycode: Some(key), ..
                    } => {
                        if let Some((row, col)) = keycode_to_pet_matrix(key) {
                            cpu.bus.pia.set_key(row, col, true);
                        }
                    }
                    Event::KeyUp {
                        keycode: Some(key), ..
                    } => {
                        if let Some((row, col)) = keycode_to_pet_matrix(key) {
                            cpu.bus.pia.set_key(row, col, false);
                        }
                    }
                    _ => {}
                }
            }
        }

        for _ in 0..cycles_per_frame {
            cpu.step();
            cpu.bus.tick();

            if cpu.bus.irq_asserted {
                cpu.request_irq();
            } else {
                cpu.release_irq();
            }
        }

        if file_dialog.is_visible() {
            draw_file_dialog(&mut canvas, &file_dialog, &ttf_context);
        } else {
            draw_pet_screen(&mut canvas, &cpu.bus);
        }

        let elapsed = last_frame.elapsed();
        if elapsed < Duration::from_millis(16) {
            std::thread::sleep(Duration::from_millis(16) - elapsed);
        }
        last_frame = Instant::now();
    }

    Ok(())
}

fn keycode_to_pet_matrix(key: Keycode) -> Option<(usize, usize)> {
    match key {
        Keycode::Q => Some((2, 0)),
        Keycode::E => Some((2, 1)),
        Keycode::T => Some((2, 2)),
        Keycode::U => Some((2, 3)),
        Keycode::O => Some((2, 4)),
        Keycode::Num7 => Some((2, 6)),
        Keycode::Num9 => Some((2, 7)),

        Keycode::W => Some((3, 0)),
        Keycode::R => Some((3, 1)),
        Keycode::Y => Some((3, 2)),
        Keycode::I => Some((3, 3)),
        Keycode::P => Some((3, 4)),
        Keycode::Num8 => Some((3, 6)),
        Keycode::Slash => Some((3, 7)),

        Keycode::A => Some((4, 0)),
        Keycode::D => Some((4, 1)),
        Keycode::G => Some((4, 2)),
        Keycode::J => Some((4, 3)),
        Keycode::L => Some((4, 4)),
        Keycode::Num4 => Some((4, 6)),
        Keycode::Num6 => Some((4, 7)),

        Keycode::S => Some((5, 0)),
        Keycode::F => Some((5, 1)),
        Keycode::H => Some((5, 2)),
        Keycode::K => Some((5, 3)),
        Keycode::Semicolon => Some((5, 4)),
        Keycode::Num5 => Some((5, 6)),
        Keycode::KpMultiply => Some((5, 7)),

        Keycode::Z => Some((6, 0)),
        Keycode::C => Some((6, 1)),
        Keycode::B => Some((6, 2)),
        Keycode::M => Some((6, 3)),
        Keycode::Return => Some((6, 5)),
        Keycode::Num1 => Some((6, 6)),
        Keycode::Num3 => Some((6, 7)),

        Keycode::X => Some((7, 0)),
        Keycode::V => Some((7, 1)),
        Keycode::N => Some((7, 2)),
        Keycode::Comma => Some((7, 3)),
        Keycode::Num2 => Some((7, 6)),
        Keycode::Equals => Some((7, 7)),

        Keycode::Minus => Some((8, 7)),
        Keycode::Num0 => Some((8, 6)),
        Keycode::RShift => Some((8, 5)),
        Keycode::RightBracket => Some((8, 2)),
        Keycode::LShift => Some((8, 0)),

        Keycode::Period => Some((9, 6)),
        Keycode::Space => Some((9, 2)),
        Keycode::LeftBracket => Some((9, 1)),

        Keycode::Backspace => Some((1, 7)),
        Keycode::Down => Some((1, 6)),
        Keycode::Backslash => Some((1, 3)),
        Keycode::Quote => Some((1, 2)),
        Keycode::Backquote => Some((1, 0)),

        Keycode::Right => Some((0, 7)),
        Keycode::Home => Some((0, 6)),

        _ => None,
    }
}
