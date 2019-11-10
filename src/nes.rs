pub mod cpu;
pub mod ppu;
pub mod palette;
pub mod mem;

use crate::nes::cpu::Cpu;
use crate::nes::ppu::Ppu;
use crate::nes::mem::Mem;
use std::fs;
use std::io::Read;
use std::cell::RefCell;
use std::rc::Rc;
use piston_window::PistonWindow;
use opengl_graphics::OpenGL;


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
            Mem::new(pgr_rom.to_vec(), chr_rom.to_vec())
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
            self.ppu.emulate(cycles_taken * 3);
            i -= cycles_taken as i32;
        }
//        println!("LOOP!");
    }

    pub fn render_frame(&mut self, r: piston_window::RenderArgs) {
        self.ppu.render(r);
    }
}

