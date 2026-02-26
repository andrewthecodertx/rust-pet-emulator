use crate::crtc6845::Crtc6845;
use crate::pia6821::Pia6821;
use crate::rom_loader::RomData;
use crate::via6522::Via6522;
use mos6502::bus::Bus as CpuBus;

pub struct PetBus {
    pub ram: [u8; 0x8800],
    pub roms: RomData,
    pub via: Via6522,
    pub pia: Pia6821,
    pub crtc: Crtc6845,
    pub irq_asserted: bool,
    pub total_cycles: u64,
}

impl PetBus {
    pub fn new(roms: RomData) -> Self {
        let mut crtc = Crtc6845::new();
        crtc.init_pet4032_screen();
        Self {
            ram: [0; 0x8800],
            roms,
            via: Via6522::new(),
            pia: Pia6821::new(),
            crtc,
            irq_asserted: false,
            total_cycles: 0,
        }
    }

    pub fn tick(&mut self) {
        self.total_cycles += 1;
        self.via.tick(1);
        self.pia.tick(1);

        let via_irq = self.via.irq_out;
        let pia_irq = self.pia.irq_out();
        self.irq_asserted = via_irq || pia_irq;
    }
}

impl CpuBus for PetBus {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => self.ram[addr as usize],
            0x8000..=0x87FF => self.ram[addr as usize],
            0xB000..=0xBFFF => self.roms.basic_b000[(addr & 0x0FFF) as usize],
            0xC000..=0xCFFF => self.roms.basic_c000[(addr & 0x0FFF) as usize],
            0xD000..=0xDFFF => self.roms.basic_d000[(addr & 0x0FFF) as usize],
            0xE000..=0xE7FF => self.roms.editor_e000[(addr & 0x07FF) as usize],
            0xE810..=0xE813 => {
                let reg = (addr & 0x03) as u8;
                self.pia.read_register(reg)
            }
            0xE840..=0xE84F => {
                let reg = (addr & 0x0F) as u8;
                self.via.read_register(reg)
            }
            0xE880..=0xE881 => {
                let reg = (addr & 0x01) as usize;
                self.crtc.read_register(reg)
            }
            0xF000..=0xFFFF => self.roms.kernal_f000[(addr - 0xF000) as usize],
            _ => 0xFF,
        }
    }

    fn write(&mut self, addr: u16, val: u8) {
        match addr {
            0x0000..=0x7FFF => self.ram[addr as usize] = val,
            0x8000..=0x87FF => self.ram[addr as usize] = val,
            0xE810..=0xE813 => {
                let reg = (addr & 0x03) as u8;
                self.pia.write_register(reg, val);
            }
            0xE840..=0xE84F => {
                let reg = (addr & 0x0F) as u8;
                self.via.write_register(reg, val);
            }
            0xE880..=0xE881 => {
                let reg = (addr & 0x01) as usize;
                self.crtc.write_register(reg, val);
            }
            _ => {}
        }
    }

    fn tick(&mut self) {}
}
