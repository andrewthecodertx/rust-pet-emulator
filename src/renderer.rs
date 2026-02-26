use crate::bus::PetBus;
use crate::file_dialog::FileDialog;
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

fn get_font_path() -> Option<&'static str> {
    let paths = [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
    ];
    for path in &paths {
        if std::path::Path::new(path).exists() {
            return Some(path);
        }
    }
    None
}

pub fn draw_file_dialog(
    canvas: &mut Canvas<Window>,
    file_dialog: &FileDialog,
    ttf_context: &sdl2::ttf::Sdl2TtfContext,
) {
    canvas.set_draw_color(Color::RGB(32, 32, 32));
    canvas.fill_rect(Rect::new(40, 40, 560, 320)).unwrap();
    canvas.set_draw_color(Color::RGB(200, 200, 200));
    canvas.draw_rect(Rect::new(40, 40, 560, 320)).unwrap();

    let font_path = match get_font_path() {
        Some(path) => path,
        None => return,
    };

    let font = ttf_context.load_font(font_path, 14).unwrap();
    let white = Color::RGB(255, 255, 255);
    let yellow = Color::RGB(255, 255, 0);
    let texture_creator = canvas.texture_creator();

    let dir_surface = font
        .render(file_dialog.current_dir())
        .blended(white)
        .unwrap();
    let dir_texture = texture_creator
        .create_texture_from_surface(&dir_surface)
        .unwrap();
    let dir_query = dir_texture.query();
    canvas
        .copy(
            &dir_texture,
            None,
            Rect::new(50, 50, dir_query.width, dir_query.height),
        )
        .unwrap();

    for (i, entry) in file_dialog.entries().iter().enumerate() {
        let color = if i == file_dialog.selected_index() {
            yellow
        } else {
            white
        };
        let surface = font.render(entry).blended(color).unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();
        let query = texture.query();
        canvas
            .copy(
                &texture,
                None,
                Rect::new(50, 70 + (i as i32 * 20), query.width, query.height),
            )
            .unwrap();
    }

    let help_text = "F2: Close | Up/Down: Navigate | Enter: Select | Backspace: Up";
    let help_surface = font.render(help_text).blended(white).unwrap();
    let help_texture = texture_creator
        .create_texture_from_surface(&help_surface)
        .unwrap();
    let help_query = help_texture.query();
    canvas
        .copy(
            &help_texture,
            None,
            Rect::new(50, 340, help_query.width, help_query.height),
        )
        .unwrap();

    canvas.present();
}
