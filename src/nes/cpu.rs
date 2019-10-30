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
    pub fn new(mem: Rc<RefCell<Mem>>) -> Cpu {
        let pc = 0xC000; // automatic tests in nestest start at this address
        Cpu { pc, a: 0, x: 0, y: 0, s: 0xFD, p: 0x24, mem: Rc::clone(&mem), cycles: 7 }
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
                5
            } else {
                3
            }
        } else {
            self.pc = self.pc.wrapping_add(1);
            2
        }
    }

    pub fn run_next_opcode(&mut self) -> u8 {
        //Emulates one opcode and returns the amount of cycles one opcode took
        let opcode = self.mem.borrow_mut().read_u8(self.pc);
        #[cfg(debug_assertions)]
            self.log_me(opcode);
        self.pc = self.pc.wrapping_add(1);
        match opcode {
//            0x0 => { 0 }
//            0x1 => { 1 }
//            0x2 => { 2 }
//            0x3 => { 3 }
//            0x4 => { 4 }
//            0x5 => { 5 }
//            0x6 => { 6 }
//            0x7 => { 7 }
            0x8 => {
                self.stack_push_u8(self.p | 0b10000);
                3
            }
//            0x9 => { 9 }
//            0xa => { 10 }
//            0xb => { 11 }
//            0xc => { 12 }
//            0xd => { 13 }
//            0xe => { 14 }
//            0xf => { 15 }
            0x10 => {
                let branch = self.get_negative() == false;
                self.branch_if(branch)
            }
//            0x11 => { 17 }
//            0x12 => { 18 }
//            0x13 => { 19 }
//            0x14 => { 20 }
//            0x15 => { 21 }
//            0x16 => { 22 }
//            0x17 => { 23 }
            0x18 => {
                self.set_carry(false);
                2
            }
//            0x19 => { 25 }
//            0x1a => { 26 }
//            0x1b => { 27 }
//            0x1c => { 28 }
//            0x1d => { 29 }
//            0x1e => { 30 }
//            0x1f => { 31 }
            0x20 => {
                let jmp_adr = self.mem.borrow_mut().read_u16(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.stack_push_u16(self.pc);
                self.pc = jmp_adr;
                6
            }
//            0x21 => { 33 }
//            0x22 => { 34 }
//            0x23 => { 35 }
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
//            0x25 => { 37 }
//            0x26 => { 38 }
//            0x27 => { 39 }
//            0x28 => { 40 }
            0x29 => {
                let n: u8 = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.a = self.a & n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                2
            }
//            0x2a => { 42 }
//            0x2b => { 43 }
//            0x2c => { 44 }
//            0x2d => { 45 }
//            0x2e => { 46 }
//            0x2f => { 47 }
            0x30 => {
                let branch = self.get_negative();
                self.branch_if(branch)
            }
//            0x31 => { 49 }
//            0x32 => { 50 }
//            0x33 => { 51 }
//            0x34 => { 52 }
//            0x35 => { 53 }
//            0x36 => { 54 }
//            0x37 => { 55 }
            0x38 => {
                self.set_carry(true);
                2
            }
//            0x39 => { 57 }
//            0x3a => { 58 }
//            0x3b => { 59 }
//            0x3c => { 60 }
//            0x3d => { 61 }
//            0x3e => { 62 }
//            0x3f => { 63 }
//            0x40 => { 64 }
//            0x41 => { 65 }
//            0x42 => { 66 }
//            0x43 => { 67 }
//            0x44 => { 68 }
//            0x45 => { 69 }
//            0x46 => { 70 }
//            0x47 => { 71 }
//            0x48 => { 72 }
//            0x49 => { 73 }
//            0x4a => { 74 }
//            0x4b => { 75 }
            0x4c => {
                self.pc = self.mem.borrow_mut().read_u16(self.pc);
                3
            }
//            0x4d => { 77 }
//            0x4e => { 78 }
//            0x4f => { 79 }
            0x50 => {
                let branch = self.get_overflow() == false;
                self.branch_if(branch)
            }
//            0x51 => { 81 }
//            0x52 => { 82 }
//            0x53 => { 83 }
//            0x54 => { 84 }
//            0x55 => { 85 }
//            0x56 => { 86 }
//            0x57 => { 87 }
//            0x58 => { 88 }
//            0x59 => { 89 }
//            0x5a => { 90 }
//            0x5b => { 91 }
//            0x5c => { 92 }
//            0x5d => { 93 }
//            0x5e => { 94 }
//            0x5f => { 95 }
            0x60 => {
                self.pc = self.stack_pop_u16();
                self.pc = self.pc.wrapping_add(1);
                6
            }
//            0x61 => { 97 }
//            0x62 => { 98 }
//            0x63 => { 99 }
//            0x64 => { 100 }
//            0x65 => { 101 }
//            0x66 => { 102 }
//            0x67 => { 103 }
            0x68 => {
                self.a = self.stack_pop_u8();
                self.set_zero(self.a == 0);
                self.set_negative(self.a >= 128);
                4
            }
//            0x69 => { 105 }
//            0x6a => { 106 }
//            0x6b => { 107 }
//            0x6c => { 108 }
//            0x6d => { 109 }
//            0x6e => { 110 }
//            0x6f => { 111 }
            0x70 => {
                let branch = self.get_overflow();
                self.branch_if(branch)
            }
//            0x71 => { 113 }
//            0x72 => { 114 }
//            0x73 => { 115 }
//            0x74 => { 116 }
//            0x75 => { 117 }
//            0x76 => { 118 }
//            0x77 => { 119 }
            0x78 => {
                self.set_interrupt_disable(true);
                2
            }
//            0x79 => { 121 }
//            0x7a => { 122 }
//            0x7b => { 123 }
//            0x7c => { 124 }
//            0x7d => { 125 }
//            0x7e => { 126 }
//            0x7f => { 127 }
//            0x80 => { 128 }
//            0x81 => { 129 }
//            0x82 => { 130 }
//            0x83 => { 131 }
//            0x84 => { 132 }
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
//            0x87 => { 135 }
//            0x88 => { 136 }
//            0x89 => { 137 }
//            0x8a => { 138 }
//            0x8b => { 139 }
//            0x8c => { 140 }
//            0x8d => { 141 }
//            0x8e => { 142 }
//            0x8f => { 143 }
            0x90 => {
                let branch = self.get_carry() == false;
                self.branch_if(branch)
            }
//            0x91 => { 145 }
//            0x92 => { 146 }
//            0x93 => { 147 }
//            0x94 => { 148 }
//            0x95 => { 149 }
//            0x96 => { 150 }
//            0x97 => { 151 }
//            0x98 => { 152 }
//            0x99 => { 153 }
//            0x9a => { 154 }
//            0x9b => { 155 }
//            0x9c => { 156 }
//            0x9d => { 157 }
//            0x9e => { 158 }
//            0x9f => { 159 }
//            0xa0 => { 160 }
//            0xa1 => { 161 }
            0xa2 => {
                let n = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.x = n;
                self.set_zero(self.x == 0);
                self.set_negative(self.x & 128 > 0);
                2
            }
//            0xa3 => { 163 }
//            0xa4 => { 164 }
//            0xa5 => { 165 }
//            0xa6 => { 166 }
//            0xa7 => { 167 }
//            0xa8 => { 168 }
            0xa9 => {
                let n = self.mem.borrow_mut().read_u8(self.pc);
                self.pc = self.pc.wrapping_add(1);
                self.a = n;
                self.set_zero(self.a == 0);
                self.set_negative(self.a & 128 > 0);
                2
            }
//            0xaa => { 170 }
//            0xab => { 171 }
//            0xac => { 172 }
//            0xad => { 173 }
//            0xae => { 174 }
//            0xaf => { 175 }
            0xb0 => {
                let branch = self.get_carry();
                self.branch_if(branch)
            }
//            0xb1 => { 177 }
//            0xb2 => { 178 }
//            0xb3 => { 179 }
//            0xb4 => { 180 }
//            0xb5 => { 181 }
//            0xb6 => { 182 }
//            0xb7 => { 183 }
//            0xb8 => { 184 }
//            0xb9 => { 185 }
//            0xba => { 186 }
//            0xbb => { 187 }
//            0xbc => { 188 }
//            0xbd => { 189 }
//            0xbe => { 190 }
//            0xbf => { 191 }
//            0xc0 => { 192 }
//            0xc1 => { 193 }
//            0xc2 => { 194 }
//            0xc3 => { 195 }
//            0xc4 => { 196 }
//            0xc5 => { 197 }
//            0xc6 => { 198 }
//            0xc7 => { 199 }
//            0xc8 => { 200 }
//            0xc9 => { 201 }
//            0xca => { 202 }
//            0xcb => { 203 }
//            0xcc => { 204 }
//            0xcd => { 205 }
//            0xce => { 206 }
//            0xcf => { 207 }
            0xd0 => {
                let branch = self.get_zero() == false;
                self.branch_if(branch)
            }
//            0xd1 => { 209 }
//            0xd2 => { 210 }
//            0xd3 => { 211 }
//            0xd4 => { 212 }
//            0xd5 => { 213 }
//            0xd6 => { 214 }
//            0xd7 => { 215 }
//            0xd8 => { 216 }
//            0xd9 => { 217 }
//            0xda => { 218 }
//            0xdb => { 219 }
//            0xdc => { 220 }
//            0xdd => { 221 }
//            0xde => { 222 }
//            0xdf => { 223 }
//            0xe0 => { 224 }
//            0xe1 => { 225 }
//            0xe2 => { 226 }
//            0xe3 => { 227 }
//            0xe4 => { 228 }
//            0xe5 => { 229 }
//            0xe6 => { 230 }
//            0xe7 => { 231 }
//            0xe8 => { 232 }
//            0xe9 => { 233 }
            0xea => {
                2
            }
//            0xeb => { 235 }
//            0xec => { 236 }
//            0xed => { 237 }
//            0xee => { 238 }
//            0xef => { 239 }
            0xf0 => {
                let branch = self.get_zero();
                self.branch_if(branch)
            }
//            0xf1 => { 241 }
//            0xf2 => { 242 }
//            0xf3 => { 243 }
//            0xf4 => { 244 }
//            0xf5 => { 245 }
//            0xf6 => { 246 }
//            0xf7 => { 247 }
            0xf8 => {
                self.set_decimal(true);
                2
            }
//            0xf9 => { 249 }
//            0xfa => { 250 }
//            0xfb => { 251 }
//            0xfc => { 252 }
//            0xfd => { 253 }
//            0xfe => { 254 }
//            0xff => { 255 }
            _ => panic!("Unimplemented!: 0x{:X}", opcode)
        }
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cpu {{ pc: 0x{:X}, a: 0x{:X}, x: 0x{:X}, y: 0x{:X}, s: 0x{:X}, p: 0x{:X} }}",
               self.pc, self.a, self.x, self.y, self.s, self.p)
    }
}