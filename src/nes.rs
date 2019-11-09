pub mod cpu;
pub mod ppu;

use crate::nes::cpu::Cpu;
use crate::nes::ppu::Ppu;
use std::fs;
use std::io::Read;
use std::cell::RefCell;
use std::rc::Rc;
use piston_window::PistonWindow;
use opengl_graphics::OpenGL;


pub struct Mem {
    ram: [u8; 0x800],
    vram: [u8; 0x4000],
    pgr_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    pub log_string: String,
    pub irq: u8,
    ppu_target_adr: u16,
    ppu_writing_high_adress_bit: bool,
}

impl Mem {
    pub fn read_u8(&mut self, addr: u16) -> u8 {
        match addr {
            0x0..=0x17FF => {
                self.ram[addr as usize % self.ram.len()]
            }
            0x2000..=0x3FFF => {
                let ppu_reg = addr % 8;
//                println!("Reading from: {:?}", ppu_reg);
                match ppu_reg {
                    2 => {
                        return 128;
                    }
                    7 => {
                        let ret_val = self.read_vram(self.ppu_target_adr);
                        self.ppu_target_adr = self.ppu_target_adr.wrapping_add(1);
                        ret_val
                    }
                    _ => 0
                }
            }
            0x8000..=0xFFFF => {
                let mut real_addr = addr - 0x8000;
//                if self.pgr_rom.len() <= 0x4000 {
                real_addr = real_addr % 0x4000;
//                }
                self.pgr_rom[real_addr as usize]
            }
            _ => { 0 }
        }
    }

    pub fn read_signed(&mut self, addr: u16) -> i8 {
        self.read_u8(addr) as i8
    }

    pub fn read_u16(&mut self, addr: u16) -> u16 {
        let lower_byte = self.read_u8(addr);
        let upper_byte = self.read_u8(addr.wrapping_add(1));

        (lower_byte as u16) | ((upper_byte as u16) << 8)
    }

    pub fn write_u8(&mut self, addr: u16, val: u8) {
        match addr {
            0x0..=0x17FF => {
                self.ram[addr as usize % self.ram.len()] = val;
            }
            0x2000..=0x3FFF => {
                let ppu_reg = addr % 8;
//                println!("Writing: 0x{:X} to ppu register {:?}", val, ppu_reg);
                match ppu_reg {
                    6 => {
                        if self.ppu_writing_high_adress_bit {
                            self.ppu_target_adr = (self.ppu_target_adr & 0xFF) | ((val as u16) << 8);
                            self.ppu_writing_high_adress_bit = false;
                        } else {
                            self.ppu_target_adr = self.ppu_target_adr | (val as u16);
                            self.ppu_writing_high_adress_bit = true;
                        }
                    }
                    7 => {
                        self.write_vram(self.ppu_target_adr, val);
                        self.ppu_target_adr = self.ppu_target_adr.wrapping_add(1);
                    }
                    _ => ()
                }
            }
            _ => {}
        }
    }

    pub fn read_vram(&mut self, addr: u16) -> u8 {
        match addr {
            0..=0x1FFF => {
                self.chr_rom[addr as usize]
            }
            _ => self.vram[addr as usize]
        }
    }

    pub fn write_vram(&mut self, addr: u16, val: u8) {
        match addr {
            0..=0x1FFF => {
                panic!("Yet to be implemented (write {:X} to {:X})", val, addr);
            }
            0x2000..=0x3FFF => {
                println!("VRAM: writing {:X} to {:X}", val, addr);
                self.vram[addr as usize] = val;
            }
            _ => {
                panic!("Out of range (write {:X} to {:X})", val, addr);
            }
        }
    }
}


pub struct Nes {
    rom_header: Vec<u8>,
    cpu: Cpu,
    ppu: Ppu,
    pub mem: Rc<RefCell<Mem>>,
}

impl Nes {
    pub fn new(filepath: &str, window: &mut PistonWindow, opengl: OpenGL,
               (width, height): (u32, u32)) -> Nes {
        //Load in the game rom and return the emulator
        let mut file = fs::File::open(filepath).unwrap();
        let mut rom_bytes: Vec<u8> = vec![];
        file.read_to_end(&mut rom_bytes).unwrap();

        //Split INES file into header and rom bytes
        // (there are also CHR rom data there, but we don't really care about them right now)
        //TODO: Parse rom header properly
        let (rom_header_bytes, rom) = rom_bytes.split_at(16);
        let pgr_length = rom_header_bytes[4] as u64 * 16384;
        let chr_length = rom_header_bytes[5] as u64 * 8192;
        let (pgr_rom, chr_rom_and_rest) = rom.split_at(pgr_length as usize);
        let (chr_rom, rest) = chr_rom_and_rest.split_at(chr_length as usize);
        let mem = Rc::new(RefCell::new(
            Mem {
                ram: [0; 0x800],
                vram: [0; 0x4000],
                pgr_rom: pgr_rom.to_vec(),
                chr_rom: chr_rom.to_vec(),
                log_string: "".to_string(),
                irq: 1,
                ppu_target_adr: 0,
                ppu_writing_high_adress_bit: true,
            }
        ));

        Nes {
            rom_header: rom_header_bytes.to_vec(),
            mem: Rc::clone(&mem),
            cpu: Cpu::new(&mem),
            ppu: Ppu::new(&mem, window, opengl, (width, height)),
        }
    }

    pub fn emulate_frame(&mut self) {
        //Emulate a fixed amount of cycles pef frame (roughly 1,79 / 60)
        let mut i: i32 = 29829;
        while i > 0 {
//            #[cfg(debug_assertions)]
//            println!("{:?}", self.cpu);
            let cycles_taken = self.cpu.emulate();
            self.ppu.emulate(cycles_taken);
            i -= cycles_taken as i32;
        }
//        println!("LOOP!");
    }

    pub fn render_frame(&mut self, r: piston_window::RenderArgs) {
        self.ppu.render(r);
    }
}

