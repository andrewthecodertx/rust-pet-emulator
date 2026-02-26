use crate::bus::PetBus;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

pub fn draw_pet_screen(canvas: &mut Canvas<Window>, bus: &PetBus) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let green = Color::RGB(50, 255, 50);
    canvas.set_draw_color(green);

    let mut video_ram_start = bus.crtc.screen_start_address() as usize;

    if video_ram_start >= 0x800 {
        video_ram_start = 0_usize + (video_ram_start - 0_usize) % (0x7FF - 0x000 + 1);
    }

    let video_ram = &bus.ram[0x8000 + video_ram_start..0x8000 + video_ram_start + 1000];
    let char_rom = &bus.roms.char_rom;

    for row in 0..25 {
        for col in 0..40 {
            let char_code = video_ram[row * 40 + col];
            let is_inverted = (char_code & 0x80) != 0;
            let glyph_offset = ((char_code & 0x7F) as usize) * 8;

            for y in 0..8 {
                if glyph_offset + y >= char_rom.len() {
                    continue;
                }

                let mut byte = char_rom[glyph_offset + y];

                if is_inverted {
                    byte = !byte;
                }

                for x in 0..8 {
                    if (byte & (0x80 >> x)) != 0 {
                        let _ = canvas.fill_rect(Rect::new(
                            (col * 8 + x) as i32 * 2,
                            (row * 8 + y) as i32 * 2,
                            2,
                            2,
                        ));
                    }
                }
            }
        }
    }
    canvas.present();
}
