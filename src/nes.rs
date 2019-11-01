pub mod cpu;

use cpu::Cpu;
use std::fs;
use std::io::Read;
use std::cell::RefCell;
use std::rc::Rc;


pub struct Mem {
    ram: [u8; 0x800],
    pgr_rom: Vec<u8>,
    pub log_string: String
}

impl Mem {
    pub fn read_u8(&mut self, addr: u16) -> u8 {
        match addr {
            0x0..=0x17FF => {
                self.ram[addr as usize % self.ram.len()]
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
            _ => { }
        }
    }
}


pub struct Nes {
    rom_header: Vec<u8>,
    cpu: Cpu,
    pub mem: Rc<RefCell<Mem>>,
}

impl Nes {
    pub fn new(filepath: &str) -> Nes {
        //Load in the game rom and return the emulator
        let mut file = fs::File::open(filepath).unwrap();
        let mut rom_bytes: Vec<u8> = vec![];
        file.read_to_end(&mut rom_bytes).unwrap();

        //Split INES file into header and rom bytes
        // (there are also CHR rom data there, but we don't really care about them right now)
        //TODO: Care about CHR rom data
        //TODO: Parse rom header properly
        let (rom_header_bytes, rom) = rom_bytes.split_at(16);
        let mem = Rc::new(RefCell::new(
            Mem { ram: [0; 0x800], pgr_rom: rom.to_vec(), log_string: "".to_string()}
        ));

        Nes {
            rom_header: rom_header_bytes.to_vec(),
            mem: Rc::clone(&mem),
            cpu: Cpu::new(mem),
        }
    }

    pub fn emulate_frame(&mut self) {
        //Emulate a fixed amount of cycles pef frame
        let mut i: i32 = 1789773;
//        while i > 0 {
//            #[cfg(debug_assertions)]
//            println!("{:?}", self.cpu);
            let cycles_taken = self.cpu.emulate();
            i -= cycles_taken as i32;
//        }
    }

    pub fn render_frame(&mut self, c: piston_window::context::Context,
                        g: &mut piston_window::G2d) {
        //Clear whole screen
        piston_window::clear([0.0; 4], g);

        //Render the frame
        piston_window::rectangle(
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 0.0, 100.0, 100.0],
            c.transform,
            g);
    }
}

