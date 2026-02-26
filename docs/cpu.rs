use crate::addressing::AddressingMode;
use crate::bus::Bus;
use crate::instructions::{Opcode, get_opcode};
use crate::status::{Flag, StatusRegister};

pub struct Cpu<B: Bus> {
    // Registers
    pub pc: u16,
    pub sp: u8,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub status: StatusRegister,

    // Internal state
    pub cycles: u8,
    pub halted: bool,

    // Interrupt flags
    nmi_pending: bool,
    irq_pending: bool,
    nmi_edge_detected: bool,

    pub bus: B,
}

impl<B: Bus> Cpu<B> {
    pub fn new(bus: B) -> Self {
        Self {
            pc: 0,
            sp: 0xFD,
            a: 0,
            x: 0,
            y: 0,
            status: StatusRegister::new(),
            cycles: 0,
            halted: false,
            nmi_pending: false,
            irq_pending: false,
            nmi_edge_detected: false,
            bus,
        }
    }

    /// Reads the reset vector from $FFFC-$FFFD and jumps there
    pub fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.sp = 0xFD;
        self.status = StatusRegister::new();
        self.pc = self.read_word(0xFFFC);
        self.cycles = 7; // Reset takes 7 cycles
        self.halted = false;
        self.nmi_pending = false;
        self.irq_pending = false;
        self.nmi_edge_detected = false;
    }

    pub fn step(&mut self) {
        if self.halted {
            return;
        }

        if self.cycles > 0 {
            self.cycles -= 1;
            self.bus.tick();
            return;
        }

        if self.nmi_pending {
            self.handle_nmi();
            return;
        }

        if self.irq_pending && !self.status.get(Flag::InterruptDisable) {
            self.handle_irq();
            return;
        }

        // Fetch and execute instruction
        let opcode_byte = self.fetch_byte();
        let opcode = get_opcode(opcode_byte);

        self.execute_opcode(opcode);
    }

    pub fn execute_instruction(&mut self) {
        while self.cycles > 0 {
            self.step();
        }

        self.step();

        while self.cycles > 0 {
            self.step();
        }
    }

    pub fn request_nmi(&mut self) {
        if !self.nmi_edge_detected {
            self.nmi_pending = true;
            self.nmi_edge_detected = true;
        }
    }

    #[allow(dead_code)]
    pub fn release_nmi(&mut self) {
        self.nmi_edge_detected = false;
    }

    pub fn request_irq(&mut self) {
        self.irq_pending = true;
    }

    #[allow(dead_code)]
    pub fn release_irq(&mut self) {
        self.irq_pending = false;
    }

    #[allow(dead_code)]
    pub fn halt(&mut self) {
        self.halted = true;
    }

    // ========== Memory Access ==========
    #[allow(dead_code)]
    pub fn read_byte(&mut self, address: u16) -> u8 {
        self.bus.read(address)
    }

    #[allow(dead_code)]
    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.bus.write(address, value);
    }

    pub fn read_word(&mut self, address: u16) -> u16 {
        self.bus.read_word(address)
    }

    fn fetch_byte(&mut self) -> u8 {
        let value = self.bus.read(self.pc);
        self.pc = self.pc.wrapping_add(1);
        value
    }

    #[allow(dead_code)]
    fn fetch_word(&mut self) -> u16 {
        let low = self.fetch_byte() as u16;
        let high = self.fetch_byte() as u16;
        (high << 8) | low
    }

    // ========== Stack Operations ==========
    pub fn push_byte(&mut self, value: u8) {
        self.bus.write(0x0100 | (self.sp as u16), value);
        self.sp = self.sp.wrapping_sub(1);
    }

    pub fn pull_byte(&mut self) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.bus.read(0x0100 | (self.sp as u16))
    }

    pub fn push_word(&mut self, value: u16) {
        self.push_byte((value >> 8) as u8);
        self.push_byte(value as u8);
    }

    pub fn pull_word(&mut self) -> u16 {
        let low = self.pull_byte() as u16;
        let high = self.pull_byte() as u16;
        (high << 8) | low
    }

    // ========== Interrupt Handling ==========
    fn handle_nmi(&mut self) {
        self.push_word(self.pc);
        self.push_byte(self.status.to_byte() & !0x10); // Clear B flag

        self.status.set(Flag::InterruptDisable, true);
        self.pc = self.read_word(0xFFFA);

        self.cycles = 7;
        self.nmi_pending = false;
    }

    fn handle_irq(&mut self) {
        self.push_word(self.pc);
        self.push_byte(self.status.to_byte() & !0x10); // Clear B flag

        self.status.set(Flag::InterruptDisable, true);
        self.pc = self.read_word(0xFFFE);

        self.cycles = 7;
    }

    // ========== Addressing Mode Helpers ==========
    fn get_operand_address(&mut self, mode: AddressingMode) -> (u16, bool) {
        match mode {
            AddressingMode::Implied | AddressingMode::Accumulator => (0, false),

            AddressingMode::Immediate => {
                let addr = self.pc;
                self.pc = self.pc.wrapping_add(1);
                (addr, false)
            }

            AddressingMode::ZeroPage => {
                let addr = self.fetch_byte() as u16;
                (addr, false)
            }

            AddressingMode::ZeroPageX => {
                let base = self.fetch_byte();
                let addr = base.wrapping_add(self.x) as u16;
                (addr, false)
            }

            AddressingMode::ZeroPageY => {
                let base = self.fetch_byte();
                let addr = base.wrapping_add(self.y) as u16;
                (addr, false)
            }

            AddressingMode::Absolute => {
                let addr = self.fetch_word();
                (addr, false)
            }

            AddressingMode::AbsoluteX => {
                let base = self.fetch_word();
                let addr = base.wrapping_add(self.x as u16);
                let page_crossed = (base & 0xFF00) != (addr & 0xFF00);
                (addr, page_crossed)
            }

            AddressingMode::AbsoluteY => {
                let base = self.fetch_word();
                let addr = base.wrapping_add(self.y as u16);
                let page_crossed = (base & 0xFF00) != (addr & 0xFF00);
                (addr, page_crossed)
            }

            AddressingMode::IndirectX => {
                let base = self.fetch_byte();
                let ptr = base.wrapping_add(self.x);
                let low = self.bus.read(ptr as u16) as u16;
                let high = self.bus.read(ptr.wrapping_add(1) as u16) as u16;
                ((high << 8) | low, false)
            }

            AddressingMode::IndirectY => {
                let ptr = self.fetch_byte();
                let low = self.bus.read(ptr as u16) as u16;
                let high = self.bus.read(ptr.wrapping_add(1) as u16) as u16;
                let base = (high << 8) | low;
                let addr = base.wrapping_add(self.y as u16);
                let page_crossed = (base & 0xFF00) != (addr & 0xFF00);
                (addr, page_crossed)
            }

            AddressingMode::Indirect => {
                let ptr = self.fetch_word();
                // NMOS 6502 bug: if ptr is $xxFF, high byte wraps within page
                let low = self.bus.read(ptr) as u16;
                let high = if (ptr & 0x00FF) == 0x00FF {
                    self.bus.read(ptr & 0xFF00) as u16
                } else {
                    self.bus.read(ptr.wrapping_add(1)) as u16
                };
                ((high << 8) | low, false)
            }

            AddressingMode::Relative => {
                let offset = self.fetch_byte();
                (offset as u16, false)
            }
        }
    }

    // ========== Instruction Execution ==========
    fn execute_opcode(&mut self, opcode: &'static Opcode) {
        let mnemonic = opcode.mnemonic;
        let mode = opcode.mode;

        let (address, page_crossed) = self.get_operand_address(mode);

        let mut extra_cycles: u8 = 0;
        if opcode.page_boundary_cycle && page_crossed {
            extra_cycles += 1;
        }

        match mnemonic {
            "LDA" => {
                let value = self.bus.read(address);
                self.lda(value);
            }
            "LDX" => {
                let value = self.bus.read(address);
                self.ldx(value);
            }
            "LDY" => {
                let value = self.bus.read(address);
                self.ldy(value);
            }
            "STA" => {
                let value = self.sta();
                self.bus.write(address, value);
            }
            "STX" => {
                let value = self.stx();
                self.bus.write(address, value);
            }
            "STY" => {
                let value = self.sty();
                self.bus.write(address, value);
            }

            // Transfer
            "TAX" => self.tax(),
            "TAY" => self.tay(),
            "TXA" => self.txa(),
            "TYA" => self.tya(),
            "TSX" => self.tsx(),
            "TXS" => self.txs(),

            // Stack
            "PHA" => self.pha(),
            "PLA" => self.pla(),
            "PHP" => self.php(),
            "PLP" => self.plp(),

            // Arithmetic
            "ADC" => {
                let value = self.bus.read(address);
                self.adc(value);
            }
            "SBC" => {
                let value = self.bus.read(address);
                self.sbc(value);
            }
            "CMP" => {
                let value = self.bus.read(address);
                self.cmp(value);
            }
            "CPX" => {
                let value = self.bus.read(address);
                self.cpx(value);
            }
            "CPY" => {
                let value = self.bus.read(address);
                self.cpy(value);
            }

            // Logic
            "AND" => {
                let value = self.bus.read(address);
                self.and(value);
            }
            "ORA" => {
                let value = self.bus.read(address);
                self.ora(value);
            }
            "EOR" => {
                let value = self.bus.read(address);
                self.eor(value);
            }
            "BIT" => {
                let value = self.bus.read(address);
                self.bit(value);
            }

            // Shift/Rotate
            "ASL" => {
                if mode == AddressingMode::Accumulator {
                    self.asl_acc();
                } else {
                    let value = self.bus.read(address);
                    let result = self.asl_mem(value);
                    self.bus.write(address, result);
                }
            }
            "LSR" => {
                if mode == AddressingMode::Accumulator {
                    self.lsr_acc();
                } else {
                    let value = self.bus.read(address);
                    let result = self.lsr_mem(value);
                    self.bus.write(address, result);
                }
            }
            "ROL" => {
                if mode == AddressingMode::Accumulator {
                    self.rol_acc();
                } else {
                    let value = self.bus.read(address);
                    let result = self.rol_mem(value);
                    self.bus.write(address, result);
                }
            }
            "ROR" => {
                if mode == AddressingMode::Accumulator {
                    self.ror_acc();
                } else {
                    let value = self.bus.read(address);
                    let result = self.ror_mem(value);
                    self.bus.write(address, result);
                }
            }

            // Inc/Dec
            "INC" => {
                let value = self.bus.read(address);
                let result = self.inc_mem(value);
                self.bus.write(address, result);
            }
            "DEC" => {
                let value = self.bus.read(address);
                let result = self.dec_mem(value);
                self.bus.write(address, result);
            }
            "INX" => self.inx(),
            "DEX" => self.dex(),
            "INY" => self.iny(),
            "DEY" => self.dey(),

            // Flow Control
            "JMP" => self.jmp(address),
            "JSR" => self.jsr(address),
            "RTS" => self.rts(),
            "BRK" => self.brk(),
            "RTI" => self.rti(),

            // Branches
            "BCC" => extra_cycles += self.bcc(address as u8),
            "BCS" => extra_cycles += self.bcs(address as u8),
            "BEQ" => extra_cycles += self.beq(address as u8),
            "BNE" => extra_cycles += self.bne(address as u8),
            "BMI" => extra_cycles += self.bmi(address as u8),
            "BPL" => extra_cycles += self.bpl(address as u8),
            "BVC" => extra_cycles += self.bvc(address as u8),
            "BVS" => extra_cycles += self.bvs(address as u8),

            // Flags
            "CLC" => self.clc(),
            "SEC" => self.sec(),
            "CLI" => self.cli(),
            "SEI" => self.sei(),
            "CLD" => self.cld(),
            "SED" => self.sed(),
            "CLV" => self.clv(),

            // NOP
            "NOP" => {}

            // Unknown/Illegal
            _ => {
                // For now, treat illegal opcodes as NOP
                // TODO: Implement illegal opcodes
            }
        }

        self.cycles = opcode.cycles + extra_cycles - 1; // -1 because we'll tick at the end
        self.bus.tick();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::SimpleBus;

    fn setup_cpu(program: &[u8]) -> Cpu<SimpleBus> {
        let mut bus = SimpleBus::new();
        bus.load(0x8000, program);
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);
        let mut cpu = Cpu::new(bus);
        cpu.reset();
        cpu
    }

    #[test]
    fn test_reset() {
        let mut bus = SimpleBus::new();
        bus.write(0xFFFC, 0x34);
        bus.write(0xFFFD, 0x12);
        let mut cpu = Cpu::new(bus);
        cpu.reset();

        assert_eq!(cpu.pc, 0x1234);
        assert_eq!(cpu.sp, 0xFD);
        assert_eq!(cpu.a, 0);
        assert_eq!(cpu.x, 0);
        assert_eq!(cpu.y, 0);
    }

    #[test]
    fn test_simple_program() {
        // LDA #$42, STA $10, LDA #$00, LDA $10
        let mut cpu = setup_cpu(&[0xA9, 0x42, 0x85, 0x10, 0xA9, 0x00, 0xA5, 0x10]);

        cpu.execute_instruction(); // LDA #$42
        assert_eq!(cpu.a, 0x42);

        cpu.execute_instruction(); // STA $10
        assert_eq!(cpu.bus.read(0x10), 0x42);

        cpu.execute_instruction(); // LDA #$00
        assert_eq!(cpu.a, 0x00);

        cpu.execute_instruction(); // LDA $10
        assert_eq!(cpu.a, 0x42);
    }

    #[test]
    fn test_indexed_addressing() {
        let mut bus = SimpleBus::new();
        // Store value at $1005
        bus.write(0x1005, 0xAB);
        // LDX #$05, LDA $1000,X
        bus.load(0x8000, &[0xA2, 0x05, 0xBD, 0x00, 0x10]);
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();

        cpu.execute_instruction(); // LDX #$05
        assert_eq!(cpu.x, 0x05);

        cpu.execute_instruction(); // LDA $1000,X
        assert_eq!(cpu.a, 0xAB);
    }

    #[test]
    fn test_indirect_y_addressing() {
        let mut bus = SimpleBus::new();
        // Set up indirect pointer at zero page $20
        bus.write(0x20, 0x00); // low byte
        bus.write(0x21, 0x10); // high byte -> points to $1000
        // Store value at $1005
        bus.write(0x1005, 0xCD);
        // LDY #$05, LDA ($20),Y
        bus.load(0x8000, &[0xA0, 0x05, 0xB1, 0x20]);
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();

        cpu.execute_instruction(); // LDY #$05
        cpu.execute_instruction(); // LDA ($20),Y

        assert_eq!(cpu.a, 0xCD);
    }

    #[test]
    fn test_nmi() {
        let mut bus = SimpleBus::new();

        bus.write(0xFFFA, 0x00);
        bus.write(0xFFFB, 0x90);

        bus.load(0x8000, &[0xEA]); // NOP
        bus.write(0xFFFC, 0x00);
        bus.write(0xFFFD, 0x80);

        let mut cpu = Cpu::new(bus);
        cpu.reset();

        while cpu.cycles > 0 {
            cpu.step();
        }

        cpu.request_nmi();
        cpu.step(); // Should handle NMI

        while cpu.cycles > 0 {
            cpu.step();
        }

        assert_eq!(cpu.pc, 0x9000);
        assert!(cpu.status.get(Flag::InterruptDisable));
    }
}
