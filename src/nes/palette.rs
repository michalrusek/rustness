pub fn get_rgb_color(index: u8) -> (u8, u8, u8) {
    match index {
        0x00 => { (0x54, 0x54, 0x54) }
        0x01 => { (0x00, 0x1E, 0x74) }
        0x02 => { (0x08, 0x10, 0x90) }
        0x03 => { (0x30, 0x00, 0x88) }
        0x04 => { (0x44, 0x00, 0x64) }
        0x05 => { (0x5C, 0x00, 0x30) }
        0x06 => { (0x54, 0x04, 0x00) }
        0x07 => { (0x3C, 0x18, 0x00) }
        0x08 => { (0x20, 0x2A, 0x00) }
        0x09 => { (0x08, 0x3A, 0x00) }
        0x0A => { (0x00, 0x40, 0x00) }
        0x0B => { (0x00, 0x3C, 0x00) }
        0x0C => { (0x00, 0x32, 0x3C) }
        0x0D => { (0x00, 0x00, 0x00) }
        0x0E => { (0x00, 0x00, 0x00) }
        0x0F => { (0x00, 0x00, 0x00) }
        0x10 => { (0x98, 0x96, 0x98) }
        0x11 => { (0x08, 0x4C, 0xC4) }
        0x12 => { (0x30, 0x32, 0xEC) }
        0x13 => { (0x5C, 0x1E, 0xE4) }
        0x14 => { (0x88, 0x14, 0xB0) }
        0x15 => { (0xA0, 0x14, 0x64) }
        0x16 => { (0x98, 0x22, 0x20) }
        0x17 => { (0x78, 0x3C, 0x00) }
        0x18 => { (0x54, 0x5A, 0x00) }
        0x19 => { (0x28, 0x72, 0x00) }
        0x1A => { (0x08, 0x7C, 0x00) }
        0x1B => { (0x00, 0x76, 0x28) }
        0x1C => { (0x00, 0x66, 0x78) }
        0x1D => { (0x00, 0x00, 0x00) }
        0x1E => { (0x00, 0x00, 0x00) }
        0x1F => { (0x00, 0x00, 0x00) }
        0x20 => { (0xEC, 0xEE, 0xEC) }
        0x21 => { (0x4C, 0x9A, 0xEC) }
        0x22 => { (0x78, 0x7C, 0xEC) }
        0x23 => { (0xB0, 0x62, 0xEC) }
        0x24 => { (0xE4, 0x54, 0xEC) }
        0x25 => { (0xEC, 0x58, 0xB4) }
        0x26 => { (0xEC, 0x6A, 0x64) }
        0x27 => { (0xD4, 0x88, 0x20) }
        0x28 => { (0xA0, 0xAA, 0x00) }
        0x29 => { (0x74, 0xC4, 0x00) }
        0x2A => { (0x4C, 0xD0, 0x20) }
        0x2B => { (0x38, 0xCC, 0x6C) }
        0x2C => { (0x38, 0xB4, 0xCC) }
        0x2D => { (0x3C, 0x3C, 0x3C) }
        0x2E => { (0x00, 0x00, 0x00) }
        0x2F => { (0x00, 0x00, 0x00) }
        0x30 => { (0xEC, 0xEE, 0xEC) }
        0x31 => { (0xA8, 0xCC, 0xEC) }
        0x32 => { (0xBC, 0xBC, 0xEC) }
        0x33 => { (0xD4, 0xB2, 0xEC) }
        0x34 => { (0xEC, 0xAE, 0xEC) }
        0x35 => { (0xEC, 0xAE, 0xD4) }
        0x36 => { (0xEC, 0xB4, 0xB0) }
        0x37 => { (0xE4, 0xC4, 0x90) }
        0x38 => { (0xCC, 0xD2, 0x78) }
        0x39 => { (0xB4, 0xDE, 0x78) }
        0x3A => { (0xA8, 0xE2, 0x90) }
        0x3B => { (0x98, 0xE2, 0xB4) }
        0x3C => { (0xA0, 0xD6, 0xE4) }
        0x3D => { (0xA0, 0xA2, 0xA0) }
        0x3E => { (0x00, 0x00, 0x00) }
        0x3F => { (0x00, 0x00, 0x00) }
        _ => (index * (255 / 4), index * (255 / 4), index * (255 / 4))
    }
}