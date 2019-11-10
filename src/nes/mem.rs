pub struct Mem {
    ram: [u8; 0x800],
    vram: [u8; 0x4000],
    pgr_rom: Vec<u8>,
    chr_rom: Vec<u8>,
    pub log_string: String,
    pub irq: u8,
    ppu_target_adr: u16,
    val_to_write_to_vram: u8,
    ppu_writing_high_adress_bit: bool,
    ppu_stat: u8,
    nmi_occured: bool,
    nmi_output: bool,
    ppu_ctrl: u8,
    trigger_nmi: bool
}

impl Mem {
    pub fn new(pgr_rom: Vec<u8>, chr_rom: Vec<u8>) -> Mem {
        Mem {
            ram: [0; 0x800],
            vram: [0; 0x4000],
            pgr_rom,
            chr_rom,
            log_string: "".to_string(),
            irq: 1,
            ppu_target_adr: 0,
            ppu_writing_high_adress_bit: true,
            val_to_write_to_vram: 0,
            ppu_stat: 0,
            nmi_occured: false,
            nmi_output: false,
            ppu_ctrl: 0,
            trigger_nmi: true
        }
    }
    pub fn should_increment_by_1(&mut self) -> bool {
        return (self.ppu_ctrl & 0b100) == 0
    }
    pub fn use_chr_0(&mut self) -> bool {
        return (self.ppu_ctrl & 0b10000) == 0
    }
    pub fn get_nmi_enable(&mut self) -> bool {
        (self.ppu_ctrl & 128) > 0
    }
    pub fn set_nmi_output(&mut self, set: bool) {
        self.nmi_output = set;
    }
    pub fn get_nmi_output(&mut self) -> bool {
        self.nmi_output
    }
    pub fn set_nmi_occured(&mut self, set: bool) {
        self.nmi_occured = set;
    }
    pub fn get_nmi_occured(&mut self) -> bool {
        self.nmi_occured
    }
    pub fn set_trigger_nmi(&mut self, set: bool) {
        self.trigger_nmi = set;
    }
    pub fn get_trigger_nmi(&mut self) -> bool {
        self.trigger_nmi
    }
    pub fn read_u8(&mut self, addr: u16) -> u8 {
        match addr {
            0x0..=0x17FF => {
                self.ram[addr as usize % self.ram.len()]
            }
            0x2000..=0x3FFF => {
                let ppu_reg = addr % 8;
                println!("Reading ppu reg: {:?}", ppu_reg);
                match ppu_reg {
                    0 => {
                        self.ppu_ctrl
                    }
                    2 => {
                        let mut data = self.ppu_stat;
                        if self.nmi_occured {
                            data = data | 128;
                        } else {
                            data = data & 0b01111111;
                        }
                        self.set_nmi_occured(false);
                        self.ppu_writing_high_adress_bit = true;
                        return data;
                    }
                    7 => {
                        let ret_val = self.read_vram(self.ppu_target_adr);
                        if self.should_increment_by_1() {
                            self.ppu_target_adr = self.ppu_target_adr.wrapping_add(1);
                        } else {
                            self.ppu_target_adr = self.ppu_target_adr.wrapping_add(32);
                        }
                        if self.ppu_target_adr == 0x4000 {
                            self.ppu_target_adr = 0;
                        }
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
                println!("Writing: 0x{:X} to ppu register {:?}", val, ppu_reg);
                match ppu_reg {
                    0 => {
                        self.ppu_ctrl = val;
                        self.set_nmi_output(self.ppu_ctrl >= 128);
                    }
                    6 => {
                        if self.ppu_writing_high_adress_bit {
                            self.ppu_target_adr = (self.ppu_target_adr & 0xFF) | ((val as u16) << 8);
                            self.ppu_writing_high_adress_bit = false;
                        } else {
                            self.ppu_target_adr = (self.ppu_target_adr & 0xFF00) | (val as u16);
                            self.ppu_writing_high_adress_bit = true;
                        }
                        println!("TARGET ADDRESS CHANGED TO: 0x{:X}", self.ppu_target_adr);
                    }
                    7 => {
                        self.val_to_write_to_vram = val;
                        self.write_vram(self.ppu_target_adr, self.val_to_write_to_vram);
                        if self.should_increment_by_1() {
                            self.ppu_target_adr = self.ppu_target_adr.wrapping_add(1);
                        } else {
                            self.ppu_target_adr = self.ppu_target_adr.wrapping_add(32);
                        }
                        if self.ppu_target_adr == 0x4000 {
                            self.ppu_target_adr = 0;
                        }
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
//                panic!("Yet to be implemented (write {:X} to {:X})", val, addr);
            }
            0x2000..=0x3FFF => {
//                println!("VRAM: writing {:X} to {:X}", val, addr);
                self.vram[addr as usize] = val;
            }
            _ => {
                panic!("Out of range (write {:X} to {:X})", val, addr);
            }
        }
    }
}