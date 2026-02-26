pub struct Pia6821 {
    ddra: u8,
    ora: u8,
    cra: u8,
    ddrb: u8,
    orb: u8,
    crb: u8,
    keyboard_matrix: [[bool; 8]; 10],
    irqa1: bool,
    irqa2: bool,
    irqb1: bool,
    irqb2: bool,
    cycle_count: u32,
    auto_type_queue: Vec<(usize, usize)>,
    auto_type_delay: u32,
    auto_type_timer: u32,
    auto_type_state: AutoTypeState,
}

#[derive(Clone, Copy, PartialEq)]
enum AutoTypeState {
    Idle,
    Pressing,
    Releasing,
}

impl Pia6821 {
    pub fn new() -> Self {
        Self {
            ddra: 0,
            ora: 0,
            cra: 0,
            ddrb: 0,
            orb: 0,
            crb: 0,
            keyboard_matrix: [[false; 8]; 10],
            irqa1: false,
            irqa2: false,
            irqb1: false,
            irqb2: false,
            cycle_count: 0,
            auto_type_queue: Vec::new(),
            auto_type_delay: 50000,
            auto_type_timer: 0,
            auto_type_state: AutoTypeState::Idle,
        }
    }

    pub fn tick(&mut self, cycles: u32) {
        self.cycle_count += cycles;

        if self.cycle_count >= 16666 {
            self.cycle_count -= 16666;
            self.irqb1 = true;
        }

        if !self.auto_type_queue.is_empty() {
            self.auto_type_timer += cycles;
            if self.auto_type_timer >= self.auto_type_delay {
                self.auto_type_timer = 0;
                match self.auto_type_state {
                    AutoTypeState::Pressing => {
                        if let Some(&(row, col)) = self.auto_type_queue.last() {
                            self.set_key(row, col, false);
                        }
                        self.auto_type_state = AutoTypeState::Releasing;
                    }
                    AutoTypeState::Releasing => {
                        self.auto_type_queue.pop();
                        if let Some(&(row, col)) = self.auto_type_queue.last() {
                            self.set_key(row, col, true);
                            self.auto_type_state = AutoTypeState::Pressing;
                        } else {
                            self.auto_type_state = AutoTypeState::Idle;
                        }
                    }
                    AutoTypeState::Idle => {}
                }
            }
        }
    }

    pub fn auto_type(&mut self, keys: &[(usize, usize)]) {
        self.auto_type_queue.clear();
        for &(row, col) in keys {
            self.auto_type_queue.push((row, col));
        }
        if let Some(&(row, col)) = self.auto_type_queue.last() {
            self.set_key(row, col, true);
            self.auto_type_state = AutoTypeState::Pressing;
        }
        self.auto_type_timer = 0;
    }

    pub fn read_register(&mut self, reg: u8) -> u8 {
        match reg {
            0 => {
                if (self.cra & 0x04) != 0 {
                    let external_input: u8 = 0xF0;
                    (self.ora & self.ddra) | (external_input & !self.ddra)
                } else {
                    self.ddra
                }
            }
            1 => {
                let mut val = self.cra & 0x3F;
                if self.irqa1 {
                    val |= 0x80;
                }
                if self.irqa2 {
                    val |= 0x40;
                }
                self.irqa1 = false;
                self.irqa2 = false;
                val
            }
            2 => {
                if (self.crb & 0x04) != 0 {
                    let mut result = self.orb & self.ddrb;
                    let keyboard_columns = self.scan_keyboard();
                    result |= keyboard_columns & !self.ddrb;
                    self.irqb1 = false;
                    result
                } else {
                    self.ddrb
                }
            }
            3 => {
                let mut val = self.crb & 0x3F;
                if self.irqb1 {
                    val |= 0x80;
                }
                if self.irqb2 {
                    val |= 0x40;
                }
                val
            }
            _ => 0xFF,
        }
    }

    pub fn write_register(&mut self, reg: u8, val: u8) {
        match reg {
            0 => {
                if (self.cra & 0x04) != 0 {
                    self.ora = val;
                } else {
                    self.ddra = val;
                }
            }
            1 => {
                self.cra = val;
            }
            2 => {
                if (self.crb & 0x04) != 0 {
                    self.orb = val;
                } else {
                    self.ddrb = val;
                }
            }
            3 => {
                self.crb = val;
            }
            _ => {}
        }
    }

    pub fn irq_out(&self) -> bool {
        let ca1_irq = (self.cra & 0x01) != 0 && self.irqa1;
        let cb1_irq = (self.crb & 0x01) != 0 && self.irqb1;
        ca1_irq || cb1_irq
    }

    fn scan_keyboard(&self) -> u8 {
        let mut columns = 0xFF_u8;
        let row = (self.ora & 0x0F) as usize;

        if row < 10 {
            for col in 0..8 {
                if self.keyboard_matrix[row][col] {
                    columns &= !(1 << col);
                }
            }
        }

        columns
    }

    pub fn set_key(&mut self, row: usize, col: usize, pressed: bool) {
        if row < 10 && col < 8 {
            self.keyboard_matrix[row][col] = pressed;
        }
    }
}
