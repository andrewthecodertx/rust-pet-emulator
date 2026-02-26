pub struct Crtc6845 {
    pub registers: [u8; 18],
    pub selected_register: usize,
    pub screen_start_address: u16,
    pub cursor_address: u16,
    pub cursor_start_reg: u8,
    pub cursor_end_reg: u8,
}

impl Crtc6845 {
    pub fn new() -> Self {
        Self {
            registers: [0; 18],
            selected_register: 0,
            screen_start_address: 0,
            cursor_address: 0,
            cursor_start_reg: 0,
            cursor_end_reg: 0,
        }
    }

    pub fn write_register(&mut self, register_select: usize, data: u8) {
        match register_select {
            0 => {
                self.selected_register = (data & 0x1F) as usize;
            }
            1 => {
                self.registers[self.selected_register] = data;
                match self.selected_register {
                    12 => {
                        self.screen_start_address =
                            (self.registers[12] as u16) << 8 | (self.registers[13] as u16)
                    }
                    13 => {
                        self.screen_start_address =
                            (self.registers[12] as u16) << 8 | (self.registers[13] as u16)
                    }
                    14 => {
                        self.cursor_address =
                            (self.registers[14] as u16) << 8 | (self.registers[15] as u16)
                    }
                    15 => {
                        self.cursor_address =
                            (self.registers[14] as u16) << 8 | (self.registers[15] as u16)
                    }
                    10 => self.cursor_start_reg = data,
                    11 => self.cursor_end_reg = data,
                    _ => (),
                }
            }
            _ => (),
        }
    }

    pub fn screen_start_address(&self) -> u16 {
        0x8000 + self.screen_start_address
    }

    pub fn init_pet4032_screen(&mut self) {
        self.registers[0] = 0x31;
        self.registers[1] = 0x27;
        self.registers[2] = 0x29;
        self.registers[3] = 0x0A;
        self.registers[4] = 0x1F;
        self.registers[5] = 0x00;
        self.registers[6] = 0x18;
        self.registers[7] = 0x1E;
        self.registers[8] = 0x00;
        self.registers[9] = 0x07;
        self.registers[10] = 0x00;
        self.registers[11] = 0x00;
        self.registers[12] = 0x00;
        self.registers[13] = 0x00;
        self.registers[14] = 0x00;
        self.registers[15] = 0x00;

        self.screen_start_address = (self.registers[12] as u16) << 8 | (self.registers[13] as u16);
        self.cursor_address = (self.registers[14] as u16) << 8 | (self.registers[15] as u16);
        self.cursor_start_reg = self.registers[10];
        self.cursor_end_reg = self.registers[11];
    }

    pub fn read_register(&self, register_select: usize) -> u8 {
        match register_select {
            0 => 0x00,
            1 => self.registers[self.selected_register],
            _ => 0,
        }
    }
}
