use std::fs;
use std::path::Path;

pub struct RomData {
    pub basic_b000: Vec<u8>,
    pub basic_c000: Vec<u8>,
    pub basic_d000: Vec<u8>,
    pub kernal_f000: Vec<u8>,
    pub editor_e000: Vec<u8>,
    pub char_rom: Vec<u8>,
}

fn load_rom(name: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let path = Path::new("roms").join(name);
    let data = fs::read(&path).map_err(|e| format!("Failed to load ROM '{}': {}", name, e))?;
    Ok(data)
}

pub fn load_roms() -> Result<RomData, Box<dyn std::error::Error>> {
    Ok(RomData {
        basic_b000: load_rom("basic-4-b000.901465-19.bin")?,
        basic_c000: load_rom("basic-4-c000.901465-20.bin")?,
        basic_d000: load_rom("basic-4-d000.901465-21.bin")?,
        kernal_f000: load_rom("kernal-4.901465-22.bin")?,
        editor_e000: load_rom("edit-4-40-n-60Hz.901499-01.bin")?,
        char_rom: load_rom("characters-2.901447-10.bin")?,
    })
}
