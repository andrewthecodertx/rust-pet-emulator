pub struct Via6522 {
    ora: u8,
    ira: u8,
    ddra: u8,
    orb: u8,
    irb: u8,
    ddrb: u8,
    t1_counter: u16,
    t1_latch: u16,
    t1_running: bool,
    t2_counter: u16,
    t2_latch_low: u8,
    sr: u8,
    acr: u8,
    pcr: u8,
    ifr: u8,
    ier: u8,
    pub irq_out: bool,
}

impl Via6522 {
    pub fn new() -> Self {
        Self {
            ora: 0,
            ira: 0xFF,
            ddra: 0,
            orb: 0,
            irb: 0xFF,
            ddrb: 0,
            t1_counter: 0xFFFF,
            t1_latch: 0,
            t1_running: false,
            t2_counter: 0xFFFF,
            t2_latch_low: 0,
            sr: 0,
            acr: 0,
            pcr: 0,
            ifr: 0,
            ier: 0,
            irq_out: false,
        }
    }

    pub fn write_register(&mut self, reg: u8, val: u8) {
        match reg {
            0x00 => self.orb = val,
            0x01 => self.ora = val,
            0x02 => self.ddrb = val,
            0x03 => self.ddra = val,
            0x04 => self.t1_latch = (self.t1_latch & 0xFF00) | (val as u16),
            0x05 => {
                self.t1_latch = (self.t1_latch & 0x00FF) | ((val as u16) << 8);
                self.t1_counter = self.t1_latch;
                self.t1_running = true;
                self.ifr &= !0x40;
                self.update_irq();
            }
            0x06 => self.t1_latch = (self.t1_latch & 0xFF00) | (val as u16),
            0x07 => {
                self.t1_latch = (self.t1_latch & 0x00FF) | ((val as u16) << 8);
                self.ifr &= !0x40;
                self.update_irq();
            }
            0x08 => self.t2_latch_low = val,
            0x09 => {
                self.t2_counter = ((val as u16) << 8) | (self.t2_latch_low as u16);
                self.ifr &= !0x20;
                self.update_irq();
            }
            0x0A => self.sr = val,
            0x0B => self.acr = val,
            0x0C => self.pcr = val,
            0x0D => {
                self.ifr &= !val;
                self.update_irq();
            }
            0x0E => {
                if val & 0x80 != 0 {
                    self.ier |= val & 0x7F;
                } else {
                    self.ier &= !(val & 0x7F);
                }
                self.update_irq();
            }
            0x0F => self.ora = val,
            _ => {}
        }
    }

    pub fn read_register(&mut self, reg: u8) -> u8 {
        match reg {
            0x00 => (self.irb & !self.ddrb) | (self.orb & self.ddrb),
            0x01 => (self.ira & !self.ddra) | (self.ora & self.ddra),
            0x02 => self.ddrb,
            0x03 => self.ddra,
            0x04 => {
                self.ifr &= !0x40;
                self.update_irq();
                (self.t1_counter & 0xFF) as u8
            }
            0x05 => (self.t1_counter >> 8) as u8,
            0x06 => (self.t1_latch & 0xFF) as u8,
            0x07 => (self.t1_latch >> 8) as u8,
            0x08 => {
                self.ifr &= !0x20;
                self.update_irq();
                (self.t2_counter & 0xFF) as u8
            }
            0x09 => (self.t2_counter >> 8) as u8,
            0x0A => self.sr,
            0x0B => self.acr,
            0x0C => self.pcr,
            0x0D => {
                let status = self.ifr & self.ier & 0x7F;
                if status != 0 {
                    self.ifr | 0x80
                } else {
                    self.ifr & 0x7F
                }
            }
            0x0E => self.ier | 0x80,
            0x0F => (self.ira & !self.ddra) | (self.ora & self.ddra),
            _ => 0xFF,
        }
    }

    pub fn tick(&mut self, cycles: u32) {
        if self.t1_running {
            if self.t1_counter <= cycles as u16 {
                self.ifr |= 0x40;

                if (self.acr & 0x40) != 0 {
                    self.t1_counter = self.t1_latch.wrapping_sub(cycles as u16 - self.t1_counter);
                } else {
                    self.t1_running = false;
                    self.t1_counter = self.t1_counter.wrapping_sub(cycles as u16);
                }
                self.update_irq();
            } else {
                self.t1_counter -= cycles as u16;
            }
        }
    }

    fn update_irq(&mut self) {
        let active = self.ifr & self.ier & 0x7F;
        if active != 0 {
            self.ifr |= 0x80;
            self.irq_out = true;
        } else {
            self.ifr &= 0x7F;
            self.irq_out = false;
        }
    }
}
