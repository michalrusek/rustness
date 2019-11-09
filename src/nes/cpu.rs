use std::fmt;
use crate::nes::Mem;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Cpu {
    pub pc: u16,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub s: u8,
    pub p: u8,
    pub cycles: u64,
    pub mem: Rc<RefCell<Mem>>,
}

impl Cpu {
    pub fn new(mem: &Rc<RefCell<Mem>>) -> Cpu {
        let pc = mem.borrow_mut().read_u16(0xFFFC);
        Cpu { pc, a: 0, x: 0, y: 0, s: 0xFD, p: 0x24, mem: Rc::clone(mem), cycles: 7 }
    }

    pub fn log_me(&self, opcode: u8) {
        self.mem.borrow_mut().log_string = format!(
            "{:04X} | {:02X} | A:{:02X} | X:{:02X} | Y:{:02X} | P:{:02X} | SP:{:02X} | CYC:{:?}",
            self.pc, opcode, self.a, self.x, self.y, self.p, self.s, self.cycles
        );
    }

    pub fn emulate(&mut self) -> u8 {
        let cycles = self.run_next_opcode();
        self.cycles += cycles as u64;
        cycles
    }

    pub fn set_negative(&mut self, set: bool) {
        if set { self.p = self.p | 0b10000000; } else { self.p = self.p & 0b01111111; }
    }
    pub fn set_overflow(&mut self, set: bool) {
        if set { self.p = self.p | 0b01000000; } else { self.p = self.p & 0b10111111; }
    }
    pub fn set_decimal(&mut self, set: bool) {
        if set { self.p = self.p | 0b00001000; } else { self.p = self.p & 0b11110111; }
    }
    pub fn set_interrupt_disable(&mut self, set: bool) {
        if set { self.p = self.p | 0b00000100; } else { self.p = self.p & 0b11111011; }
    }
    pub fn set_zero(&mut self, set: bool) {
        if set { self.p = self.p | 0b00000010; } else { self.p = self.p & 0b11111101; }
    }
    pub fn set_carry(&mut self, set: bool) {
        if set { self.p = self.p | 0b00000001; } else { self.p = self.p & 0b11111110; }
    }
    pub fn get_negative(&mut self) -> bool {
        (self.p & 0b10000000) > 0
    }
    pub fn get_overflow(&mut self) -> bool {
        (self.p & 0b01000000) > 0
    }
    pub fn get_decimal(&mut self) -> bool {
        (self.p & 0b00001000) > 0
    }
    pub fn get_interrupt_disable(&mut self) -> bool {
        (self.p & 0b00000100) > 0
    }
    pub fn get_zero(&mut self) -> bool {
        (self.p & 0b00000010) > 0
    }
    pub fn get_carry(&mut self) -> bool {
        (self.p & 0b00000001) > 0
    }
    pub fn stack_push_u8(&mut self, n: u8) {
        self.mem.borrow_mut().write_u8(self.s as u16 | 0x100, n);
        self.s = self.s.wrapping_sub(1);
    }
    pub fn stack_push_u16(&mut self, n: u16) {
        self.stack_push_u8((n >> 8) as u8);
        self.stack_push_u8((n & 0xFF) as u8);
    }
    pub fn stack_pop_u8(&mut self) -> u8 {
        self.s = self.s.wrapping_add(1);
        self.mem.borrow_mut().read_u8(self.s as u16 | 0x100)
    }
    pub fn stack_pop_u16(&mut self) -> u16 {
        let lower = self.stack_pop_u8() as u16;
        let upper = self.stack_pop_u8() as u16;
        lower | (upper << 8)
    }
    pub fn branch_if(&mut self, branch: bool) -> u8 {
        if branch {
            let offset = self.mem.borrow_mut().read_signed(self.pc);
            self.pc = self.pc.wrapping_add(1);
            let old_page = self.pc >> 8;
            self.pc = self.pc.wrapping_add(offset as u16);
            let new_page = self.pc >> 8;
            if old_page != new_page {
                4
            } else {
                3
            }
        } else {
            self.pc = self.pc.wrapping_add(1);
            2
        }
    }

    pub fn get_indirect_x_addr(&mut self) -> u16 {
        let adr_of_adr_base = self.mem.borrow_mut().read_u8(self.pc);
        self.pc = self.pc.wrapping_add(1);
        let adr_of_adr_full = adr_of_adr_base.wrapping_add(self.x);
        let adr_low_byte = self.mem.borrow_mut().read_u8(adr_of_adr_full as u16);
        let adr_high_byte = self.mem.borrow_mut().read_u8(
            adr_of_adr_full.wrapping_add(1) as u16);
        (adr_low_byte as u16) + ((adr_high_byte as u16) << 8)
    }

    pub fn get_indirect_y_addr(&mut self) -> (u16, u8) {
        let adr_of_adr_base = self.mem.borrow_mut().read_u8(self.pc);
        self.pc = self.pc.wrapping_add(1);
        let low_byte = self.mem.borrow_mut().read_u8(adr_of_adr_base as u16);
        let high_byte = self.mem.borrow_mut()
            .read_u8(adr_of_adr_base.wrapping_add(1) as u16);
        let mut adr_of_adr_full = ((high_byte as u16) << 8) + low_byte as u16;
        adr_of_adr_full = adr_of_adr_full.wrapping_add(self.y as u16);

        let additional_cycle_required = (low_byte as u16 + self.y as u16) > 0xFF;
        let mut additional_cycle = 0;
        if additional_cycle_required {
            additional_cycle = 1;
        }
        (adr_of_adr_full, additional_cycle)
    }

    pub fn get_absolute_x_addr(&mut self) -> (u16, u8) {
        let low_byte = self.mem.borrow_mut().read_u8(self.pc);
        self.pc = self.pc.wrapping_add(1);
        let high_byte = self.mem.borrow_mut().read_u8(self.pc);
        self.pc = self.pc.wrapping_add(1);
        let addr_full = (((high_byte as u16) << 8) + (low_byte as u16))
            .wrapping_add(self.x as u16);
        let additional_cycle_required = (low_byte as u16 + self.x as u16) > 0xFF;
        let mut additional_cycle = 0;
        if additional_cycle_required {
            additional_cycle = 1;
        }
        (addr_full, additional_cycle)
    }
    pub fn get_absolute_y_addr(&mut self) -> (u16, u8) {
        let low_byte = self.mem.borrow_mut().read_u8(self.pc);
        self.pc = self.pc.wrapping_add(1);
        let high_byte = self.mem.borrow_mut().read_u8(self.pc);
        self.pc = self.pc.wrapping_add(1);
        let addr_full = (((high_byte as u16) << 8) + (low_byte as u16))
            .wrapping_add(self.y as u16);
        let additional_cycle_required = (low_byte as u16 + self.y as u16) > 0xFF;
        let mut additional_cycle = 0;
        if additional_cycle_required {
            additional_cycle = 1;
        }
        (addr_full, additional_cycle)
    }

    pub fn adc(&mut self, n: u8) {
        let mut dirty = (self.a as u16).wrapping_add(n as u16);
        let mut dirty_signed = ((self.a as i8) as i16).wrapping_add((n as i8) as i16);
        if self.get_carry() {
            dirty = dirty.wrapping_add(1);
            dirty_signed = dirty_signed.wrapping_add(1);
        }
        self.a = dirty as u8;
        self.set_carry(dirty > 0xFF);
        self.set_zero(self.a == 0);
        self.set_negative(self.a >= 128);
        let a_is_signed = self.a >= 128;
        let dirty_res_is_signed = dirty_signed < 0;
        self.set_overflow(a_is_signed != dirty_res_is_signed);
    }

    pub fn sbc(&mut self, num_orig: u8) {
        let n = !num_orig;
        let mut dirty = (self.a as u16).wrapping_add(n as u16);
        let mut dirty_signed = ((self.a as i8) as i16).wrapping_add((n as i8) as i16);
        if self.get_carry() {
            dirty = dirty.wrapping_add(1);
            dirty_signed = dirty_signed.wrapping_add(1);
        }
        self.a = dirty as u8;
        self.set_carry(dirty > 0xFF);
        self.set_zero(self.a == 0);
        self.set_negative(self.a >= 128);
        let a_is_signed = self.a >= 128;
        let dirty_res_is_signed = dirty_signed < 0;
        self.set_overflow(a_is_signed != dirty_res_is_signed);
    }

    pub fn run_next_opcode(&mut self) -> u8 {
        //Serve interrupts first
        let interrupt_disable = self.get_interrupt_disable();
        if self.mem.borrow_mut().irq == 0 && !interrupt_disable {
            self.set_interrupt_disable(true);
            self.mem.borrow_mut().irq = 1;
            self.stack_push_u16(self.pc);
            self.stack_push_u8(self.p | 0b10000);
            self.pc = self.mem.borrow_mut().read_u16(0xFFFE);
            self.p = self.p & 0b11001111;
            self.p = self.p | 0b100000;
            return 7;
        }

        if self.mem.borrow_mut().trigger_nmi {
            self.mem.borrow_mut().trigger_nmi = false;
            self.set_interrupt_disable(true);
            self.mem.borrow_mut().irq = 1;
            self.stack_push_u16(self.pc);
            self.stack_push_u8(self.p | 0b10000);
            self.pc = self.mem.borrow_mut().read_u16(0xFFFA);
            self.p = self.p & 0b11001111;
            self.p = self.p | 0b100000;
            return 7;
        }

        //Emulates one opcode and returns the amount of cycles one opcode took
        let opcode = self.mem.borrow_mut().read_u8(self.pc);
        #[cfg(debug_assertions)]
            self.log_me(opcode);
        self.pc = self.pc.wrapping_add(1);
        match opcode {
            0x0 => {
                self.stack_push_u16(self.pc);
                self.stack_push_u8(self.p | 0b10000);
                self.pc = self.mem.borrow_mut().read_u16(0xFFFE);
                self.p = self.p & 0b11001111;
                self.p = self.p | 0b10000;
                7
            }
            0x1 => {
                let adr = self.get_indirect_x_addr();
                self.a = self.a | self.mem.borrow_mut().read_u8(adr);
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                6
            }
//            0x2 => { 2 }
            0x3 => {
                let adr = self.get_indirect_x_addr();
                let mut n = self.mem.borrow_mut().read_u8(adr);
                self.set_carry(n >= 128);
                n = n << 1;
                self.mem.borrow_mut().write_u8(adr, n);
                self.a = self.a | n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                8
            }
            0x4 => {
                self.pc = self.pc.wrapping_add(1);
                3
            }
            0x5 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let n = self.mem.borrow_mut().read_u8(adr as u16);
                self.a = self.a | n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                3
            }
            0x6 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let mut n = self.mem.borrow_mut().read_u8(adr as u16);
                self.set_carry(n >= 128);
                n = n << 1;
                self.set_zero(n == 0);
                self.set_negative(n >= 128);
                self.mem.borrow_mut().write_u8(adr as u16, n);
                5
            }
            0x7 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc) as u16;
                self.pc = self.pc.wrapping_add(1);
                let mut n = self.mem.borrow_mut().read_u8(adr);
                self.set_carry(n >= 128);
                n = n << 1;
                self.mem.borrow_mut().write_u8(adr, n);
                self.a = self.a | n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                5
            }
            0x8 => {
                self.stack_push_u8(self.p | 0b10000);
                3
            }
            0x9 => {
                let n = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.a = self.a | n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                2
            }
            0xa => {
                self.set_carry(self.a >= 128);
                self.a = self.a << 1;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                2
            }
//            0xb => { 11 }
            0xc => {
                self.pc = self.pc.wrapping_add(2);
                4
            }
            0xd => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let n = self.mem.borrow_mut().read_u8(adr as u16);
                self.a = self.a | n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                4
            }
            0xe => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let mut n = self.mem.borrow_mut().read_u8(adr);
                self.set_carry(n >= 128);
                n = n << 1;
                self.set_zero(n == 0);
                self.set_negative(n >= 128);
                self.mem.borrow_mut().write_u8(adr as u16, n);
                6
            }
            0xf => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let mut n = self.mem.borrow_mut().read_u8(adr);
                self.set_carry(n >= 128);
                n = n << 1;
                self.mem.borrow_mut().write_u8(adr, n);
                self.a = self.a | n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                6
            }
            0x10 => {
                let branch = self.get_negative() == false;
                self.branch_if(branch)
            }
            0x11 => {
                let (adr, additional_cycles) = self.get_indirect_y_addr();
                let n = self.mem.borrow_mut().read_u8(adr);
                self.a = self.a | n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                5 + additional_cycles
            }
//            0x12 => { 18 }
            0x13 => {
                let (adr, additional_cycles) = self.get_indirect_y_addr();
                let mut n = self.mem.borrow_mut().read_u8(adr);
                self.set_carry(n >= 128);
                n = n << 1;
                self.mem.borrow_mut().write_u8(adr, n);
                self.a = self.a | n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                8
            }
            0x14 => {
                self.pc = self.pc.wrapping_add(1);
                4
            }
            0x15 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc).wrapping_add(self.x) as u16;
                self.pc = self.pc.wrapping_add(1);
                let n = self.mem.borrow_mut().read_u8(adr);
                self.a = self.a | n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                4
            }
            0x16 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc).wrapping_add(self.x) as u16;
                self.pc = self.pc.wrapping_add(1);
                let mut n = self.mem.borrow_mut().read_u8(adr);
                self.set_carry(n >= 128);
                n = n << 1;
                self.set_zero(n == 0);
                self.set_negative(n >= 128);
                self.mem.borrow_mut().write_u8(adr as u16, n);
                6
            }
            0x17 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc).wrapping_add(self.x) as u16;
                self.pc = self.pc.wrapping_add(1);
                let mut n = self.mem.borrow_mut().read_u8(adr);
                self.set_carry(n >= 128);
                n = n << 1;
                self.mem.borrow_mut().write_u8(adr, n);
                self.a = self.a | n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                6
            }
            0x18 => {
                self.set_carry(false);
                2
            }
            0x19 => {
                let (adr, additional_cycles) = self.get_absolute_y_addr();
                let n = self.mem.borrow_mut().read_u8(adr);
                self.a = self.a | n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                4 + additional_cycles
            }
            0x1a => {
                2
            }
            0x1b => {
                let (adr, additional_cycles) = self.get_absolute_y_addr();
                let mut n = self.mem.borrow_mut().read_u8(adr);
                self.set_carry(n >= 128);
                n = n << 1;
                self.mem.borrow_mut().write_u8(adr, n);
                self.a = self.a | n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                7
            }
            0x1c => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                4 + additional_cycles
            }
            0x1d => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                let n = self.mem.borrow_mut().read_u8(adr);
                self.a = self.a | n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                4 + additional_cycles
            }
            0x1e => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                let mut n = self.mem.borrow_mut().read_u8(adr);
                self.set_carry(n >= 128);
                n = n << 1;
                self.set_zero(n == 0);
                self.set_negative(n >= 128);
                self.mem.borrow_mut().write_u8(adr as u16, n);
                7
            }
            0x1f => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                let mut n = self.mem.borrow_mut().read_u8(adr);
                self.set_carry(n >= 128);
                n = n << 1;
                self.mem.borrow_mut().write_u8(adr, n);
                self.a = self.a | n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                7
            }
            0x20 => {
                let jmp_adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.stack_push_u16(self.pc);
                self.pc = jmp_adr;
                6
            }
            0x21 => {
                let adr = self.get_indirect_x_addr();
                self.a = self.a & self.mem.borrow_mut().read_u8(adr);
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                6
            }
//            0x22 => { 34 }
            0x23 => {
                let adr = self.get_indirect_x_addr();
                let mut n = self.mem.borrow_mut().read_u8(adr);
                let c: u8;
                if self.get_carry() {
                    c = 1;
                } else {
                    c = 0;
                }
                self.set_carry((n & 128) == 128);
                n = (n << 1) | c;
                self.mem.borrow_mut().write_u8(adr, n);
                self.a = self.a & n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                8
            }
            0x24 => {
                let ad = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let n = self.mem.borrow_mut().read_u8(ad as u16);
                let res = n & self.a;
                self.set_zero(res == 0);
                self.set_negative((n >> 7) > 0);
                self.set_overflow(((n >> 6) & 0b1) > 0);
                3
            }
            0x25 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let n = self.mem.borrow_mut().read_u8(adr as u16);
                self.a = self.a & n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                3
            }
            0x26 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let mut n = self.mem.borrow_mut().read_u8(adr as u16);
                let c: u8;
                if self.get_carry() {
                    c = 1;
                } else {
                    c = 0;
                }
                self.set_carry((n & 128) == 128);
                n = (n << 1) | c;
                self.set_zero(n == 0);
                self.set_negative(n >= 128);
                self.mem.borrow_mut().write_u8(adr as u16, n);
                5
            }
            0x27 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc) as u16;
                self.pc = self.pc.wrapping_add(1);
                let mut n = self.mem.borrow_mut().read_u8(adr);
                let c: u8;
                if self.get_carry() {
                    c = 1;
                } else {
                    c = 0;
                }
                self.set_carry((n & 128) == 128);
                n = (n << 1) | c;
                self.mem.borrow_mut().write_u8(adr, n);
                self.a = self.a & n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                5
            }
            0x28 => {
                let old_b = self.p & 0b110000;
                self.p = self.stack_pop_u8();
                self.p = self.p & 0b11001111;
                self.p = self.p | old_b;
                4
            }
            0x29 => {
                let n: u8 = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.a = self.a & n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                2
            }
            0x2a => {
                let c: u8;
                if self.get_carry() {
                    c = 1;
                } else {
                    c = 0;
                }
                self.set_carry((self.a & 128) == 128);
                self.a = (self.a << 1) | c;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                2
            }
//            0x2b => { 43 }
            0x2c => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let n = self.mem.borrow_mut().read_u8(adr as u16);
                let res = n & self.a;
                self.set_zero(res == 0);
                self.set_negative((n >> 7) > 0);
                self.set_overflow(((n >> 6) & 0b1) > 0);
                4
            }
            0x2d => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let n = self.mem.borrow_mut().read_u8(adr as u16);
                self.a = self.a & n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                4
            }
            0x2e => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let mut n = self.mem.borrow_mut().read_u8(adr);
                let c: u8;
                if self.get_carry() {
                    c = 1;
                } else {
                    c = 0;
                }
                self.set_carry((n & 128) == 128);
                n = (n << 1) | c;
                self.set_zero(n == 0);
                self.set_negative(n >= 128);
                self.mem.borrow_mut().write_u8(adr, n);
                6
            }
            0x2f => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let mut n = self.mem.borrow_mut().read_u8(adr);
                let c: u8;
                if self.get_carry() {
                    c = 1;
                } else {
                    c = 0;
                }
                self.set_carry((n & 128) == 128);
                n = (n << 1) | c;
                self.mem.borrow_mut().write_u8(adr, n);
                self.a = self.a & n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                6
            }
            0x30 => {
                let branch = self.get_negative();
                self.branch_if(branch)
            }
            0x31 => {
                let (adr, additional_cycles) = self.get_indirect_y_addr();
                let n = self.mem.borrow_mut().read_u8(adr);
                self.a = self.a & n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                5 + additional_cycles
            }
//            0x32 => { 50 }
            0x33 => {
                let (adr, additional_cycles) = self.get_indirect_y_addr();
                let mut n = self.mem.borrow_mut().read_u8(adr);
                let c: u8;
                if self.get_carry() {
                    c = 1;
                } else {
                    c = 0;
                }
                self.set_carry((n & 128) == 128);
                n = (n << 1) | c;
                self.mem.borrow_mut().write_u8(adr, n);
                self.a = self.a & n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                8
            }
            0x34 => {
                self.pc = self.pc.wrapping_add(1);
                4
            }
            0x35 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc).wrapping_add(self.x) as u16;
                self.pc = self.pc.wrapping_add(1);
                let n = self.mem.borrow_mut().read_u8(adr);
                self.a = self.a & n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                4
            }
            0x36 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc).wrapping_add(self.x) as u16;
                self.pc = self.pc.wrapping_add(1);
                let mut n = self.mem.borrow_mut().read_u8(adr);
                let c: u8;
                if self.get_carry() {
                    c = 1;
                } else {
                    c = 0;
                }
                self.set_carry((n & 128) == 128);
                n = (n << 1) | c;
                self.set_zero(n == 0);
                self.set_negative(n >= 128);
                self.mem.borrow_mut().write_u8(adr as u16, n);
                6
            }
            0x37 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc).wrapping_add(self.x) as u16;
                self.pc = self.pc.wrapping_add(1);
                let mut n = self.mem.borrow_mut().read_u8(adr);
                let c: u8;
                if self.get_carry() {
                    c = 1;
                } else {
                    c = 0;
                }
                self.set_carry((n & 128) == 128);
                n = (n << 1) | c;
                self.mem.borrow_mut().write_u8(adr, n);
                self.a = self.a & n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                6
            }
            0x38 => {
                self.set_carry(true);
                2
            }
            0x39 => {
                let (adr, additional_cycles) = self.get_absolute_y_addr();
                let n = self.mem.borrow_mut().read_u8(adr);
                self.a = self.a & n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                4 + additional_cycles
            }
            0x3a => {
                2
            }
            0x3b => {
                let (adr, additional_cycles) = self.get_absolute_y_addr();
                let mut n = self.mem.borrow_mut().read_u8(adr);
                let c: u8;
                if self.get_carry() {
                    c = 1;
                } else {
                    c = 0;
                }
                self.set_carry((n & 128) == 128);
                n = (n << 1) | c;
                self.mem.borrow_mut().write_u8(adr, n);
                self.a = self.a & n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                7
            }
            0x3c => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                4 + additional_cycles
            }
            0x3d => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                let n = self.mem.borrow_mut().read_u8(adr);
                self.a = self.a & n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                4 + additional_cycles
            }
            0x3e => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                let mut n = self.mem.borrow_mut().read_u8(adr);
                let c: u8;
                if self.get_carry() {
                    c = 1;
                } else {
                    c = 0;
                }
                self.set_carry((n & 128) == 128);
                n = (n << 1) | c;
                self.set_zero(n == 0);
                self.set_negative(n >= 128);
                self.mem.borrow_mut().write_u8(adr as u16, n);
                7
            }
            0x3f => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                let mut n = self.mem.borrow_mut().read_u8(adr);
                let c: u8;
                if self.get_carry() {
                    c = 1;
                } else {
                    c = 0;
                }
                self.set_carry((n & 128) == 128);
                n = (n << 1) | c;
                self.mem.borrow_mut().write_u8(adr, n);
                self.a = self.a & n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                7
            }
            0x40 => {
                let old_b = self.p & 0b110000;
                self.p = self.stack_pop_u8() | old_b;
                self.pc = self.stack_pop_u16();
                6
            }
            0x41 => {
                let adr = self.get_indirect_x_addr();
                self.a = self.a ^ self.mem.borrow_mut().read_u8(adr);
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                6
            }
//            0x42 => { 66 }
            0x43 => {
                let adr = self.get_indirect_x_addr();
                let mut n = self.mem.borrow_mut().read_u8(adr as u16);
                self.set_carry((n & 1) == 1);
                n = n >> 1;
                self.mem.borrow_mut().write_u8(adr, n);
                self.a = self.a ^ n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                8
            }
            0x44 => {
                self.pc = self.pc.wrapping_add(1);
                3
            }
            0x45 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let n = self.mem.borrow_mut().read_u8(adr as u16);
                self.a = self.a ^ n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                3
            }
            0x46 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let mut n = self.mem.borrow_mut().read_u8(adr as u16);
                self.set_carry((n & 1) == 1);
                n = n >> 1;
                self.set_zero(n == 0);
                self.set_negative(n >= 128);
                self.mem.borrow_mut().write_u8(adr as u16, n);
                5
            }
            0x47 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc) as u16;
                self.pc = self.pc.wrapping_add(1);
                let mut n = self.mem.borrow_mut().read_u8(adr as u16);
                self.set_carry((n & 1) == 1);
                n = n >> 1;
                self.mem.borrow_mut().write_u8(adr, n);
                self.a = self.a ^ n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                5
            }
            0x48 => {
                self.stack_push_u8(self.a);
                3
            }
            0x49 => {
                let n = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.a = self.a ^ n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                2
            }
            0x4a => {
                self.set_carry((self.a & 1) == 1);
                self.a = self.a >> 1;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                2
            }
//            0x4b => { 75 }
            0x4c => {
                self.pc = self.mem.borrow_mut().read_u16(self.pc);
                3
            }
            0x4d => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let n = self.mem.borrow_mut().read_u8(adr);
                self.a = self.a ^ n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                4
            }
            0x4e => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let mut n = self.mem.borrow_mut().read_u8(adr);
                self.set_carry((n & 1) == 1);
                n = n >> 1;
                self.set_zero(n == 0);
                self.set_negative(n >= 128);
                self.mem.borrow_mut().write_u8(adr as u16, n);
                6
            }
            0x4f => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let mut n = self.mem.borrow_mut().read_u8(adr as u16);
                self.set_carry((n & 1) == 1);
                n = n >> 1;
                self.mem.borrow_mut().write_u8(adr, n);
                self.a = self.a ^ n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                6
            }
            0x50 => {
                let branch = self.get_overflow() == false;
                self.branch_if(branch)
            }
            0x51 => {
                let (adr, additional_cycles) = self.get_indirect_y_addr();
                let n = self.mem.borrow_mut().read_u8(adr);
                self.a = self.a ^ n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                5 + additional_cycles
            }
//            0x52 => { 82 }
            0x53 => {
                let (adr, additional_cycles) = self.get_indirect_y_addr();
                let mut n = self.mem.borrow_mut().read_u8(adr as u16);
                self.set_carry((n & 1) == 1);
                n = n >> 1;
                self.mem.borrow_mut().write_u8(adr, n);
                self.a = self.a ^ n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                8
            }
            0x54 => {
                self.pc = self.pc.wrapping_add(1);
                4
            }
            0x55 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc).wrapping_add(self.x) as u16;
                self.pc = self.pc.wrapping_add(1);
                let n = self.mem.borrow_mut().read_u8(adr);
                self.a = self.a ^ n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                4
            }
            0x56 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc).wrapping_add(self.x) as u16;
                self.pc = self.pc.wrapping_add(1);
                let mut n = self.mem.borrow_mut().read_u8(adr);
                self.set_carry((n & 1) == 1);
                n = n >> 1;
                self.set_zero(n == 0);
                self.set_negative(n >= 128);
                self.mem.borrow_mut().write_u8(adr as u16, n);
                6
            }
            0x57 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc).wrapping_add(self.x) as u16;
                self.pc = self.pc.wrapping_add(1);
                let mut n = self.mem.borrow_mut().read_u8(adr as u16);
                self.set_carry((n & 1) == 1);
                n = n >> 1;
                self.mem.borrow_mut().write_u8(adr, n);
                self.a = self.a ^ n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                6
            }
//            0x58 => { 88 }
            0x59 => {
                let (adr, additional_cycles) = self.get_absolute_y_addr();
                let n = self.mem.borrow_mut().read_u8(adr);
                self.a = self.a ^ n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                4 + additional_cycles
            }
            0x5a => {
                2
            }
            0x5b => {
                let (adr, additional_cycles) = self.get_absolute_y_addr();
                let mut n = self.mem.borrow_mut().read_u8(adr as u16);
                self.set_carry((n & 1) == 1);
                n = n >> 1;
                self.mem.borrow_mut().write_u8(adr, n);
                self.a = self.a ^ n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                7
            }
            0x5c => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                4 + additional_cycles
            }
            0x5d => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                let n = self.mem.borrow_mut().read_u8(adr);
                self.a = self.a ^ n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                4 + additional_cycles
            }
            0x5e => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                let mut n = self.mem.borrow_mut().read_u8(adr);
                self.set_carry((n & 1) == 1);
                n = n >> 1;
                self.set_zero(n == 0);
                self.set_negative(n >= 128);
                self.mem.borrow_mut().write_u8(adr as u16, n);
                7
            }
            0x5f => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                let mut n = self.mem.borrow_mut().read_u8(adr as u16);
                self.set_carry((n & 1) == 1);
                n = n >> 1;
                self.mem.borrow_mut().write_u8(adr, n);
                self.a = self.a ^ n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                7
            }
            0x60 => {
                self.pc = self.stack_pop_u16();
                self.pc = self.pc.wrapping_add(1);
                6
            }
            0x61 => {
                let adr = self.get_indirect_x_addr();
                let n = self.mem.borrow_mut().read_u8(adr);
                self.adc(n);
                6
            }
//            0x62 => { 98 }
            0x63 => {
                let adr = self.get_indirect_x_addr();
                let mut n = self.mem.borrow_mut().read_u8(adr as u16);
                let c: u8;
                if self.get_carry() {
                    c = 128;
                } else {
                    c = 0;
                }
                self.set_carry((n & 1) == 1);
                n = (n >> 1) | c;
                self.mem.borrow_mut().write_u8(adr as u16, n);
                self.adc(n);
                8
            }
            0x64 => {
                self.pc = self.pc.wrapping_add(1);
                3
            }
            0x65 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let n = self.mem.borrow_mut().read_u8(adr as u16);
                self.adc(n);
                3
            }
            0x66 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let mut n = self.mem.borrow_mut().read_u8(adr as u16);
                let c: u8;
                if self.get_carry() {
                    c = 128;
                } else {
                    c = 0;
                }
                self.set_carry((n & 1) == 1);
                n = (n >> 1) | c;
                self.set_zero(n == 0);
                self.set_negative(n >= 128);
                self.mem.borrow_mut().write_u8(adr as u16, n);
                5
            }
            0x67 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc) as u16;
                self.pc = self.pc.wrapping_add(1);
                let mut n = self.mem.borrow_mut().read_u8(adr as u16);
                let c: u8;
                if self.get_carry() {
                    c = 128;
                } else {
                    c = 0;
                }
                self.set_carry((n & 1) == 1);
                n = (n >> 1) | c;
                self.mem.borrow_mut().write_u8(adr as u16, n);
                self.adc(n);
                5
            }
            0x68 => {
                self.a = self.stack_pop_u8();
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                4
            }
            0x69 => {
                let n = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.adc(n);
                2
            }
            0x6a => {
                let c: u8;
                if self.get_carry() {
                    c = 128;
                } else {
                    c = 0;
                }
                self.set_carry((self.a & 1) == 1);
                self.a = (self.a >> 1) | c;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                2
            }
//            0x6b => { 107 }
            0x6c => {
                let adr_of_adr = self.mem.borrow_mut().read_u16(self.pc);
                let low_byte = self.mem.borrow_mut().read_u8(adr_of_adr);
                let high_byte: u8;
                if (adr_of_adr & 0xFF) == 0xFF {
                    high_byte = self.mem.borrow_mut().read_u8(adr_of_adr & 0xFF00);
                } else {
                    high_byte = self.mem.borrow_mut().read_u8(adr_of_adr.wrapping_add(1));
                }
                self.pc = ((high_byte as u16) << 8) | low_byte as u16;
                5
            }
            0x6d => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let n = self.mem.borrow_mut().read_u8(adr as u16);
                self.adc(n);
                4
            }
            0x6e => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let mut n = self.mem.borrow_mut().read_u8(adr);
                let c: u8;
                if self.get_carry() {
                    c = 128;
                } else {
                    c = 0;
                }
                self.set_carry((n & 1) == 1);
                n = (n >> 1) | c;
                self.set_zero(n == 0);
                self.set_negative(n >= 128);
                self.mem.borrow_mut().write_u8(adr, n);
                6
            }
            0x6f => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let mut n = self.mem.borrow_mut().read_u8(adr as u16);
                let c: u8;
                if self.get_carry() {
                    c = 128;
                } else {
                    c = 0;
                }
                self.set_carry((n & 1) == 1);
                n = (n >> 1) | c;
                self.mem.borrow_mut().write_u8(adr as u16, n);
                self.adc(n);
                6
            }
            0x70 => {
                let branch = self.get_overflow();
                self.branch_if(branch)
            }
            0x71 => {
                let (adr, additional_cycles) = self.get_indirect_y_addr();
                let n = self.mem.borrow_mut().read_u8(adr);
                self.adc(n);
                5 + additional_cycles
            }
//            0x72 => { 114 }
            0x73 => {
                let (adr, additional_cycles) = self.get_indirect_y_addr();
                let mut n = self.mem.borrow_mut().read_u8(adr as u16);
                let c: u8;
                if self.get_carry() {
                    c = 128;
                } else {
                    c = 0;
                }
                self.set_carry((n & 1) == 1);
                n = (n >> 1) | c;
                self.mem.borrow_mut().write_u8(adr as u16, n);
                self.adc(n);
                8
            }
            0x74 => {
                self.pc = self.pc.wrapping_add(1);
                4
            }
            0x75 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc).wrapping_add(self.x) as u16;
                self.pc = self.pc.wrapping_add(1);
                let n = self.mem.borrow_mut().read_u8(adr);
                self.adc(n);
                4
            }
            0x76 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc).wrapping_add(self.x) as u16;
                self.pc = self.pc.wrapping_add(1);
                let mut n = self.mem.borrow_mut().read_u8(adr);
                let c: u8;
                if self.get_carry() {
                    c = 128;
                } else {
                    c = 0;
                }
                self.set_carry((n & 1) == 1);
                n = (n >> 1) | c;
                self.set_zero(n == 0);
                self.set_negative(n >= 128);
                self.mem.borrow_mut().write_u8(adr as u16, n);
                6
            }
            0x77 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc).wrapping_add(self.x) as u16;
                self.pc = self.pc.wrapping_add(1);
                let mut n = self.mem.borrow_mut().read_u8(adr as u16);
                let c: u8;
                if self.get_carry() {
                    c = 128;
                } else {
                    c = 0;
                }
                self.set_carry((n & 1) == 1);
                n = (n >> 1) | c;
                self.mem.borrow_mut().write_u8(adr as u16, n);
                self.adc(n);
                6
            }
            0x78 => {
                self.set_interrupt_disable(true);
                2
            }
            0x79 => {
                let (adr, additional_cycles) = self.get_absolute_y_addr();
                let n = self.mem.borrow_mut().read_u8(adr);
                self.adc(n);
                4 + additional_cycles
            }
            0x7a => {
                2
            }
            0x7b => {
                let (adr, additional_cycles) = self.get_absolute_y_addr();
                let mut n = self.mem.borrow_mut().read_u8(adr as u16);
                let c: u8;
                if self.get_carry() {
                    c = 128;
                } else {
                    c = 0;
                }
                self.set_carry((n & 1) == 1);
                n = (n >> 1) | c;
                self.mem.borrow_mut().write_u8(adr as u16, n);
                self.adc(n);
                7
            }
            0x7c => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                4 + additional_cycles
            }
            0x7d => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                let n = self.mem.borrow_mut().read_u8(adr);
                self.adc(n);
                4 + additional_cycles
            }
            0x7e => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                let mut n = self.mem.borrow_mut().read_u8(adr);
                let c: u8;
                if self.get_carry() {
                    c = 128;
                } else {
                    c = 0;
                }
                self.set_carry((n & 1) == 1);
                n = (n >> 1) | c;
                self.set_zero(n == 0);
                self.set_negative(n >= 128);
                self.mem.borrow_mut().write_u8(adr as u16, n);
                7
            }
            0x7f => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                let mut n = self.mem.borrow_mut().read_u8(adr as u16);
                let c: u8;
                if self.get_carry() {
                    c = 128;
                } else {
                    c = 0;
                }
                self.set_carry((n & 1) == 1);
                n = (n >> 1) | c;
                self.mem.borrow_mut().write_u8(adr as u16, n);
                self.adc(n);
                7
            }
            0x80 => {
                self.pc = self.pc.wrapping_add(1);
                2
            }
            0x81 => {
                let adr_full = self.get_indirect_x_addr();
                self.mem.borrow_mut().write_u8(adr_full, self.a);
                6
            }
            0x82 => {
                self.pc = self.pc.wrapping_add(1);
                2
            }
            0x83 => {
                let adr = self.get_indirect_x_addr();
                let res = self.x & self.a;
                self.mem.borrow_mut().write_u8(adr, res);
                6
            }
            0x84 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.mem.borrow_mut().write_u8(adr as u16, self.y);
                3
            }
            0x85 => {
                let ad = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.mem.borrow_mut().write_u8(ad as u16, self.a);
                3
            }
            0x86 => {
                let ad = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.mem.borrow_mut().write_u8(ad as u16, self.x);
                3
            }
            0x87 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let res = self.x & self.a;
                self.mem.borrow_mut().write_u8(adr as u16, res);
                3
            }
            0x88 => {
                self.y = self.y.wrapping_sub(1);
                self.set_zero(self.y == 0);
                self.set_negative(self.y >= 128);
                2
            }
            0x89 => {
                self.pc = self.pc.wrapping_add(1);
                2
            }
            0x8a => {
                self.a = self.x;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                2
            }
//            0x8b => { 139 }
            0x8c => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                self.mem.borrow_mut().write_u8(adr, self.y);
                4
            }
            0x8d => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                self.mem.borrow_mut().write_u8(adr, self.a);
                4
            }
            0x8e => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                self.mem.borrow_mut().write_u8(adr, self.x);
                4
            }
            0x8f => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let res = self.x & self.a;
                self.mem.borrow_mut().write_u8(adr as u16, res);
                4
            }
            0x90 => {
                let branch = self.get_carry() == false;
                self.branch_if(branch)
            }
            0x91 => {
                let (adr, additional_cycles) = self.get_indirect_y_addr();
                self.mem.borrow_mut().write_u8(adr, self.a);
                6
            }
//            0x92 => { 146 }
//            0x93 => { 147 }
            0x94 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.mem.borrow_mut().write_u8(adr.wrapping_add(self.x) as u16, self.y);
                4
            }
            0x95 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc).wrapping_add(self.x) as u16;
                self.pc = self.pc.wrapping_add(1);
                self.mem.borrow_mut().write_u8(adr, self.a);
                4
            }
            0x96 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc).wrapping_add(self.y) as u16;
                self.pc = self.pc.wrapping_add(1);
                self.mem.borrow_mut().write_u8(adr, self.x);
                4
            }
            0x97 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc).wrapping_add(self.y);
                self.pc = self.pc.wrapping_add(1);
                let res = self.x & self.a;
                self.mem.borrow_mut().write_u8(adr as u16, res);
                4
            }
            0x98 => {
                self.a = self.y;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                2
            }
            0x99 => {
                let (adr, additional_cycles) = self.get_absolute_y_addr();
                self.mem.borrow_mut().write_u8(adr, self.a);
                5
            }
            0x9a => {
                self.s = self.x;
                2
            }
//            0x9b => { 155 }
//            0x9c => { 156 }
            0x9d => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                self.mem.borrow_mut().write_u8(adr, self.a);
                5
            }
//            0x9e => { 158 }
//            0x9f => { 159 }
            0xa0 => {
                let n = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.y = n;
                self.set_zero(self.y == 0);
                self.set_negative(self.y >= 128);
                2
            }
            0xa1 => {
                let adr_full = self.get_indirect_x_addr();
                self.a = self.mem.borrow_mut().read_u8(adr_full);
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                6
            }
            0xa2 => {
                let n = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.x = n;
                self.set_zero(self.x == 0);
                self.set_negative(self.x & 128 > 0);
                2
            }
            0xa3 => {
                let adr = self.get_indirect_x_addr();
                let n = self.mem.borrow_mut().read_u8(adr);
                self.x = n;
                self.a = n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                6
            }
            0xa4 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.y = self.mem.borrow_mut().read_u8(adr as u16);
                self.set_zero(self.y == 0);
                self.set_negative(self.y >= 128);
                3
            }
            0xa5 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.a = self.mem.borrow_mut().read_u8(adr as u16);
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                3
            }
            0xa6 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.x = self.mem.borrow_mut().read_u8(adr as u16);
                self.set_zero(self.x == 0);
                self.set_negative(self.x >= 128);
                3
            }
            0xa7 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let n = self.mem.borrow_mut().read_u8(adr as u16);
                self.x = n;
                self.a = n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                3
            }
            0xa8 => {
                self.y = self.a;
                self.set_zero(self.y == 0);
                self.set_negative(self.y >= 128);
                2
            }
            0xa9 => {
                let n = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.a = n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a & 128 > 0);
                2
            }
            0xaa => {
                self.x = self.a;
                self.set_zero(self.x == 0);
                self.set_negative(self.x >= 128);
                2
            }
//            0xab => { 171 }
            0xac => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                self.y = self.mem.borrow_mut().read_u8(adr);
                self.set_zero(self.y == 0);
                self.set_negative(self.y >= 128);
                4
            }
            0xad => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                self.a = self.mem.borrow_mut().read_u8(adr);
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                4
            }
            0xae => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                self.x = self.mem.borrow_mut().read_u8(adr);
                self.set_zero(self.x == 0);
                self.set_negative(self.x >= 128);
                4
            }
            0xaf => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let n = self.mem.borrow_mut().read_u8(adr);
                self.x = n;
                self.a = n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                4
            }
            0xb0 => {
                let branch = self.get_carry();
                self.branch_if(branch)
            }
            0xb1 => {
                let (adr, additional_cycles) = self.get_indirect_y_addr();
                self.a = self.mem.borrow_mut().read_u8(adr);
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                5 + additional_cycles
            }
//            0xb2 => { 178 }
            0xb3 => {
                let (adr, additional_cycles) = self.get_indirect_y_addr();
                let n = self.mem.borrow_mut().read_u8(adr);
                self.x = n;
                self.a = n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                5 + additional_cycles
            }
            0xb4 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc).wrapping_add(self.x) as u16;
                self.pc = self.pc.wrapping_add(1);
                self.y = self.mem.borrow_mut().read_u8(adr);
                self.set_zero(self.y == 0);
                self.set_negative(self.y >= 128);
                4
            }
            0xb5 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc).wrapping_add(self.x) as u16;
                self.pc = self.pc.wrapping_add(1);
                self.a = self.mem.borrow_mut().read_u8(adr);
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                4
            }
            0xb6 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc).wrapping_add(self.y) as u16;
                self.pc = self.pc.wrapping_add(1);
                self.x = self.mem.borrow_mut().read_u8(adr);
                self.set_zero(self.x == 0);
                self.set_negative(self.x >= 128);
                4
            }
            0xb7 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc).wrapping_add(self.y);
                self.pc = self.pc.wrapping_add(1);
                let n = self.mem.borrow_mut().read_u8(adr as u16);
                self.x = n;
                self.a = n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                4
            }
            0xb8 => {
                self.set_overflow(false);
                2
            }
            0xb9 => {
                let (adr, additional_cycles) = self.get_absolute_y_addr();
                self.a = self.mem.borrow_mut().read_u8(adr);
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                4 + additional_cycles
            }
            0xba => {
                self.x = self.s;
                self.set_zero(self.x == 0);
                self.set_negative(self.x >= 128);
                2
            }
//            0xbb => { 187 }
            0xbc => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                self.y = self.mem.borrow_mut().read_u8(adr);
                self.set_zero(self.y == 0);
                self.set_negative(self.y >= 128);
                4 + additional_cycles
            }
            0xbd => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                self.a = self.mem.borrow_mut().read_u8(adr);
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                4 + additional_cycles
            }
            0xbe => {
                let (adr, additional_cycles) = self.get_absolute_y_addr();
                self.x = self.mem.borrow_mut().read_u8(adr);
                self.set_zero(self.x == 0);
                self.set_negative(self.x >= 128);
                4 + additional_cycles
            }
            0xbf => {
                let (adr, additional_cycles) = self.get_absolute_y_addr();
                let n = self.mem.borrow_mut().read_u8(adr);
                self.x = n;
                self.a = n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                4 + additional_cycles
            }
            0xc0 => {
                let n = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.set_negative(self.y.wrapping_sub(n) >= 128);
                self.set_zero(self.y == n);
                self.set_carry(self.y >= n);
                2
            }
            0xc1 => {
                let adr = self.get_indirect_x_addr();
                let n = self.mem.borrow_mut().read_u8(adr);
                self.set_negative(self.a.wrapping_sub(n) >= 128);
                self.set_zero(self.a == n);
                self.set_carry(self.a >= n);
                6
            }
            0xc2 => {
                self.pc = self.pc.wrapping_add(1);
                2
            }
            0xc3 => {
                let adr = self.get_indirect_x_addr();
                let n_orig = self.mem.borrow_mut().read_u8(adr);
                let n_sub = n_orig.wrapping_sub(1);
                self.mem.borrow_mut().write_u8(adr, n_sub);
                self.set_negative(self.a.wrapping_sub(n_sub) >= 128);
                self.set_zero(self.a == n_sub);
                self.set_carry(self.a >= n_sub);
                8
            }
            0xc4 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let n = self.mem.borrow_mut().read_u8(adr as u16);
                self.set_negative(self.y.wrapping_sub(n) >= 128);
                self.set_zero(self.y == n);
                self.set_carry(self.y >= n);
                3
            }
            0xc5 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let n = self.mem.borrow_mut().read_u8(adr as u16);
                self.set_negative(self.a.wrapping_sub(n) >= 128);
                self.set_zero(self.a == n);
                self.set_carry(self.a >= n);
                3
            }
            0xc6 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let n = self.mem.borrow_mut().read_u8(adr as u16).wrapping_sub(1);
                self.mem.borrow_mut().write_u8(adr as u16, n);
                self.set_zero(n == 0);
                self.set_negative(n >= 128);
                5
            }
            0xc7 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc) as u16;
                self.pc = self.pc.wrapping_add(1);
                let n_orig = self.mem.borrow_mut().read_u8(adr);
                let n_sub = n_orig.wrapping_sub(1);
                self.mem.borrow_mut().write_u8(adr, n_sub);
                self.set_negative(self.a.wrapping_sub(n_sub) >= 128);
                self.set_zero(self.a == n_sub);
                self.set_carry(self.a >= n_sub);
                5
            }
            0xc8 => {
                self.y = self.y.wrapping_add(1);
                self.set_zero(self.y == 0);
                self.set_negative(self.y >= 128);
                2
            }
            0xc9 => {
                let n = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.set_negative(self.a.wrapping_sub(n) >= 128);
                self.set_zero(self.a == n);
                self.set_carry(self.a >= n);
                2
            }
            0xca => {
                self.x = self.x.wrapping_sub(1);
                self.set_zero(self.x == 0);
                self.set_negative(self.x >= 128);
                2
            }
//            0xcb => { 203 }
            0xcc => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let n = self.mem.borrow_mut().read_u8(adr as u16);
                self.set_negative(self.y.wrapping_sub(n) >= 128);
                self.set_zero(self.y == n);
                self.set_carry(self.y >= n);
                4
            }
            0xcd => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let n = self.mem.borrow_mut().read_u8(adr as u16);
                self.set_negative(self.a.wrapping_sub(n) >= 128);
                self.set_zero(self.a == n);
                self.set_carry(self.a >= n);
                4
            }
            0xce => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let n = self.mem.borrow_mut().read_u8(adr).wrapping_sub(1);
                self.mem.borrow_mut().write_u8(adr as u16, n);
                self.set_zero(n == 0);
                self.set_negative(n >= 128);
                6
            }
            0xcf => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let n_orig = self.mem.borrow_mut().read_u8(adr);
                let n_sub = n_orig.wrapping_sub(1);
                self.mem.borrow_mut().write_u8(adr, n_sub);
                self.set_negative(self.a.wrapping_sub(n_sub) >= 128);
                self.set_zero(self.a == n_sub);
                self.set_carry(self.a >= n_sub);
                6
            }
            0xd0 => {
                let branch = self.get_zero() == false;
                self.branch_if(branch)
            }
            0xd1 => {
                let (adr, additional_cycles) = self.get_indirect_y_addr();
                let n = self.mem.borrow_mut().read_u8(adr);
                self.set_negative(self.a.wrapping_sub(n) >= 128);
                self.set_zero(self.a == n);
                self.set_carry(self.a >= n);
                5 + additional_cycles
            }
//            0xd2 => { 210 }
            0xd3 => {
                let (adr, additional_cycles) = self.get_indirect_y_addr();
                let n_orig = self.mem.borrow_mut().read_u8(adr);
                let n_sub = n_orig.wrapping_sub(1);
                self.mem.borrow_mut().write_u8(adr, n_sub);
                self.set_negative(self.a.wrapping_sub(n_sub) >= 128);
                self.set_zero(self.a == n_sub);
                self.set_carry(self.a >= n_sub);
                8
            }
            0xd4 => {
                self.pc = self.pc.wrapping_add(1);
                4
            }
            0xd5 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc).wrapping_add(self.x) as u16;
                self.pc = self.pc.wrapping_add(1);
                let n = self.mem.borrow_mut().read_u8(adr);
                self.set_negative(self.a.wrapping_sub(n) >= 128);
                self.set_zero(self.a == n);
                self.set_carry(self.a >= n);
                4
            }
            0xd6 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc).wrapping_add(self.x) as u16;
                self.pc = self.pc.wrapping_add(1);
                let n = self.mem.borrow_mut().read_u8(adr).wrapping_sub(1);
                self.mem.borrow_mut().write_u8(adr as u16, n);
                self.set_zero(n == 0);
                self.set_negative(n >= 128);
                6
            }
            0xd7 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc).wrapping_add(self.x) as u16;
                self.pc = self.pc.wrapping_add(1);
                let n_orig = self.mem.borrow_mut().read_u8(adr);
                let n_sub = n_orig.wrapping_sub(1);
                self.mem.borrow_mut().write_u8(adr, n_sub);
                self.set_negative(self.a.wrapping_sub(n_sub) >= 128);
                self.set_zero(self.a == n_sub);
                self.set_carry(self.a >= n_sub);
                6
            }
            0xd8 => {
                self.set_decimal(false);
                2
            }
            0xd9 => {
                let (adr, additional_cycles) = self.get_absolute_y_addr();
                let n = self.mem.borrow_mut().read_u8(adr);
                self.set_negative(self.a.wrapping_sub(n) >= 128);
                self.set_zero(self.a == n);
                self.set_carry(self.a >= n);
                4 + additional_cycles
            }
            0xda => {
                2
            }
            0xdb => {
                let (adr, additional_cycles) = self.get_absolute_y_addr();
                let n_orig = self.mem.borrow_mut().read_u8(adr);
                let n_sub = n_orig.wrapping_sub(1);
                self.mem.borrow_mut().write_u8(adr, n_sub);
                self.set_negative(self.a.wrapping_sub(n_sub) >= 128);
                self.set_zero(self.a == n_sub);
                self.set_carry(self.a >= n_sub);
                7
            }
            0xdc => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                4 + additional_cycles
            }
            0xdd => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                let n = self.mem.borrow_mut().read_u8(adr);
                self.set_negative(self.a.wrapping_sub(n) >= 128);
                self.set_zero(self.a == n);
                self.set_carry(self.a >= n);
                4 + additional_cycles
            }
            0xde => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                let n = self.mem.borrow_mut().read_u8(adr).wrapping_sub(1);
                self.mem.borrow_mut().write_u8(adr as u16, n);
                self.set_zero(n == 0);
                self.set_negative(n >= 128);
                7
            }
            0xdf => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                let n_orig = self.mem.borrow_mut().read_u8(adr);
                let n_sub = n_orig.wrapping_sub(1);
                self.mem.borrow_mut().write_u8(adr, n_sub);
                self.set_negative(self.a.wrapping_sub(n_sub) >= 128);
                self.set_zero(self.a == n_sub);
                self.set_carry(self.a >= n_sub);
                7
            }
            0xe0 => {
                let n = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.set_negative(self.x.wrapping_sub(n) >= 128);
                self.set_zero(self.x == n);
                self.set_carry(self.x >= n);
                2
            }
            0xe1 => {
                let adr = self.get_indirect_x_addr();
                let n = self.mem.borrow_mut().read_u8(adr);
                self.sbc(n);
                6
            }
            0xe2 => {
                self.pc = self.pc.wrapping_add(1);
                2
            }
            0xe3 => {
                let adr = self.get_indirect_x_addr();
                let n = self.mem.borrow_mut().read_u8(adr).wrapping_add(1);
                self.mem.borrow_mut().write_u8(adr, n);
                self.sbc(n);
                8
            }
            0xe4 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let n = self.mem.borrow_mut().read_u8(adr as u16);
                self.set_negative(self.x.wrapping_sub(n) >= 128);
                self.set_zero(self.x == n);
                self.set_carry(self.x >= n);
                3
            }
            0xe5 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let n = self.mem.borrow_mut().read_u8(adr as u16);
                self.sbc(n);
                3
            }
            0xe6 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                let n = self.mem.borrow_mut().read_u8(adr as u16).wrapping_add(1);
                self.mem.borrow_mut().write_u8(adr as u16, n);
                self.set_zero(n == 0);
                self.set_negative(n >= 128);
                5
            }
            0xe7 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc) as u16;
                self.pc = self.pc.wrapping_add(1);
                let n = self.mem.borrow_mut().read_u8(adr).wrapping_add(1);
                self.mem.borrow_mut().write_u8(adr, n);
                self.sbc(n);
                5
            }
            0xe8 => {
                self.x = self.x.wrapping_add(1);
                self.set_zero(self.x == 0);
                self.set_negative(self.x >= 128);
                2
            }
            0xe9 => {
                let n: u8 = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.sbc(n);
                2
            }
            0xea => {
                2
            }
            0xeb => {
                let n: u8 = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.sbc(n);
                2
            }
            0xec => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let n = self.mem.borrow_mut().read_u8(adr as u16);
                self.set_negative(self.x.wrapping_sub(n) >= 128);
                self.set_zero(self.x == n);
                self.set_carry(self.x >= n);
                4
            }
            0xed => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let n = self.mem.borrow_mut().read_u8(adr as u16);
                self.sbc(n);
                4
            }
            0xee => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let n = self.mem.borrow_mut().read_u8(adr).wrapping_add(1);
                self.mem.borrow_mut().write_u8(adr as u16, n);
                self.set_zero(n == 0);
                self.set_negative(n >= 128);
                6
            }
            0xef => {
                let adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(2);
                let n = self.mem.borrow_mut().read_u8(adr).wrapping_add(1);
                self.mem.borrow_mut().write_u8(adr, n);
                self.sbc(n);
                6
            }
            0xf0 => {
                let branch = self.get_zero();
                self.branch_if(branch)
            }
            0xf1 => {
                let (adr, additional_cycles) = self.get_indirect_y_addr();
                let n = self.mem.borrow_mut().read_u8(adr);
                self.sbc(n);
                5 + additional_cycles
            }
//            0xf2 => { 242 }
            0xf3 => {
                let (adr, additional_cycles) = self.get_indirect_y_addr();
                let n = self.mem.borrow_mut().read_u8(adr).wrapping_add(1);
                self.mem.borrow_mut().write_u8(adr, n);
                self.sbc(n);
                8
            }
            0xf4 => {
                self.pc = self.pc.wrapping_add(1);
                4
            }
            0xf5 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc).wrapping_add(self.x) as u16;
                self.pc = self.pc.wrapping_add(1);
                let n = self.mem.borrow_mut().read_u8(adr);
                self.sbc(n);
                4
            }
            0xf6 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc).wrapping_add(self.x) as u16;
                self.pc = self.pc.wrapping_add(1);
                let n = self.mem.borrow_mut().read_u8(adr).wrapping_add(1);
                self.mem.borrow_mut().write_u8(adr as u16, n);
                self.set_zero(n == 0);
                self.set_negative(n >= 128);
                6
            }
            0xf7 => {
                let adr = self.mem.borrow_mut().read_u8(self.pc).wrapping_add(self.x) as u16;
                self.pc = self.pc.wrapping_add(1);
                let n = self.mem.borrow_mut().read_u8(adr).wrapping_add(1);
                self.mem.borrow_mut().write_u8(adr, n);
                self.sbc(n);
                6
            }
            0xf8 => {
                self.set_decimal(true);
                2
            }
            0xf9 => {
                let (adr, additional_cycles) = self.get_absolute_y_addr();
                let n = self.mem.borrow_mut().read_u8(adr);
                self.sbc(n);
                4 + additional_cycles
            }
            0xfa => {
                2
            }
            0xfb => {
                let (adr, additional_cycles) = self.get_absolute_y_addr();
                let n = self.mem.borrow_mut().read_u8(adr).wrapping_add(1);
                self.mem.borrow_mut().write_u8(adr, n);
                self.sbc(n);
                7
            }
            0xfc => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                4 + additional_cycles
            }
            0xfd => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                let n = self.mem.borrow_mut().read_u8(adr);
                self.sbc(n);
                4 + additional_cycles
            }
            0xfe => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                let n = self.mem.borrow_mut().read_u8(adr).wrapping_add(1);
                self.mem.borrow_mut().write_u8(adr as u16, n);
                self.set_zero(n == 0);
                self.set_negative(n >= 128);
                7
            }
            0xff => {
                let (adr, additional_cycles) = self.get_absolute_x_addr();
                let n = self.mem.borrow_mut().read_u8(adr).wrapping_add(1);
                self.mem.borrow_mut().write_u8(adr, n);
                self.sbc(n);
                7
            }
            _ => panic!("Unimplemented!: 0x{:X} (at 0x{:X}, cycles:{:?})", opcode, self.pc, self.cycles)
        }
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cpu {{ pc: 0x{:X}, a: 0x{:X}, x: 0x{:X}, y: 0x{:X}, s: 0x{:X}, p: 0x{:X} }}",
               self.pc, self.a, self.x, self.y, self.s, self.p)
    }
}