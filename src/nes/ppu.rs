use std::cell::RefCell;
use std::rc::Rc;
use crate::nes::palette::get_rgb_color;
use piston_window::{PistonWindow, Image};
use opengl_graphics::OpenGL;
use crate::nes::mem::Mem;
use std::borrow::Borrow;

type Tile = [[u8; 8]; 8];

pub struct Ppu {
    pub mem: Rc<RefCell<Mem>>,
    gl: opengl_graphics::GlGraphics,
    canvas: im::ImageBuffer<im::Rgba<u8>, Vec<u8>>,
    texture: opengl_graphics::Texture,
    img: Image,
    chr_tiles0: [Tile; 256],
    chr_tiles1: [Tile; 256],
    nametable0: [Tile; 960],
    nametable1: [Tile; 960],
    nametable2: [Tile; 960],
    nametable3: [Tile; 960],
    bg_palette0: [(u8, u8, u8); 4],
    bg_palette1: [(u8, u8, u8); 4],
    bg_palette2: [(u8, u8, u8); 4],
    bg_palette3: [(u8, u8, u8); 4],
    pallete_per_tile0: [u8; 960],
    pallete_per_tile1: [u8; 960],
    pallete_per_tile2: [u8; 960],
    pallete_per_tile3: [u8; 960],
    current_scanline: i32,
    cycles_total: u64,
    cycles_for_current_scanline: u16,
    triggered_nmi_this_scanline: bool,
}

const CYCLES_PER_SCANLINE: u16 = 340;
const CHR_0_X_Y: (u32, u32) = (700, 0);
const CHR_1_X_Y: (u32, u32) = (828, 0);
const NAMETABLE_0_X_Y: (u32, u32) = (700, 128);
const NAMETABLE_1_X_Y: (u32, u32) = (700 + 256, 128);
const NAMETABLE_2_X_Y: (u32, u32) = (700 + 256, 128 + 240);
const NAMETABLE_3_X_Y: (u32, u32) = (700, 128 + 240);
const SCREEN_X_Y: (u32, u32) = (0, 0);

impl Ppu {
    pub fn new(mem: &Rc<RefCell<Mem>>, window: &mut PistonWindow, opengl: OpenGL,
               (width, height): (u32, u32)) -> Ppu {
        let mut gl = opengl_graphics::GlGraphics::new(opengl);

        let mut canvas = im::ImageBuffer::new(width, height);
        let mut texture = opengl_graphics::Texture::from_image(&canvas, &opengl_graphics::TextureSettings::new());
        let img = Image::new().rect(graphics::rectangle::rectangle_by_corners(0.0, 0.0, width as f64, height as f64));
        Ppu {
            mem: Rc::clone(mem),
            gl,
            canvas,
            texture,
            img,
            chr_tiles0: [[[0; 8]; 8]; 256],
            chr_tiles1: [[[0; 8]; 8]; 256],
            nametable0: [[[0; 8]; 8]; 960],
            nametable1: [[[0; 8]; 8]; 960],
            nametable2: [[[0; 8]; 8]; 960],
            nametable3: [[[0; 8]; 8]; 960],
            bg_palette0: [(0, 0, 0); 4],
            bg_palette1: [(0, 0, 0); 4],
            bg_palette2: [(0, 0, 0); 4],
            bg_palette3: [(0, 0, 0); 4],
            pallete_per_tile0: [0; 960],
            pallete_per_tile1: [0; 960],
            pallete_per_tile2: [0; 960],
            pallete_per_tile3: [0; 960],
            current_scanline: -1,
            cycles_total: 0,
            cycles_for_current_scanline: 0,
            triggered_nmi_this_scanline: true,
        }
    }

    pub fn emulate(&mut self, cycles: u8) {
        for i in 0..cycles {
            if self.current_scanline >= 0 && self.current_scanline < 240
                && self.cycles_for_current_scanline < 256 {
                let (nametable_x, nametable_y) = match self.mem.borrow_mut().get_nametable_index() {
                    0 => NAMETABLE_0_X_Y,
                    1 => NAMETABLE_1_X_Y,
                    2 => NAMETABLE_2_X_Y,
                    3 => NAMETABLE_3_X_Y,
                    _ => (0, 0)
                };
                let scroll_x = self.mem.borrow_mut().get_scroll_x() as u32;
                let scroll_y = self.mem.borrow_mut().get_scroll_y() as u32;
                let pixel = self.canvas.get_pixel(
                    ((self.cycles_for_current_scanline as u32 + scroll_x) % 512) + nametable_x,
                    ((self.current_scanline as u32 + scroll_y) % 480) + nametable_y,
                );
                let target_x = self.cycles_for_current_scanline as u32 * 2;
                let target_y = self.current_scanline as u32 * 2;
                let px_clone = pixel.clone();
                self.canvas.put_pixel(target_x, target_y, px_clone);
                self.canvas.put_pixel(target_x + 1, target_y, px_clone);
                self.canvas.put_pixel(target_x + 1, target_y + 1, px_clone);
                self.canvas.put_pixel(target_x, target_y + 1, px_clone);
            }

            self.cycles_total = self.cycles_total + 1;
            self.cycles_for_current_scanline += 1;
            if self.cycles_for_current_scanline >= CYCLES_PER_SCANLINE {
                self.cycles_for_current_scanline -= CYCLES_PER_SCANLINE;
                self.current_scanline += 1;
                self.triggered_nmi_this_scanline = false;
            }
        }

        if self.current_scanline == 240 && !self.triggered_nmi_this_scanline {
            self.mem.borrow_mut().set_nmi_occured(true);
            let nmi_out = self.mem.borrow_mut().get_nmi_output();
            if nmi_out {
                self.mem.borrow_mut().set_trigger_nmi(true);
            }
            self.triggered_nmi_this_scanline = true;
        }
        if self.current_scanline == 261 {
            self.mem.borrow_mut().set_nmi_occured(false);
            self.current_scanline = 0;
        }
    }

    fn prepare_bg_stuff(&mut self) {
        let universal_bg_color = self.get_universal_bg_color();
        self.bg_palette0 = self.get_palette(0x3F01, universal_bg_color);
        self.bg_palette1 = self.get_palette(0x3F05, universal_bg_color);
        self.bg_palette2 = self.get_palette(0x3F09, universal_bg_color);
        self.bg_palette3 = self.get_palette(0x3F0D, universal_bg_color);

        self.pallete_per_tile0 = self.parse_attr_to_tiles(0x23C0);
        self.pallete_per_tile1 = self.parse_attr_to_tiles(0x27C0);
        self.pallete_per_tile2 = self.parse_attr_to_tiles(0x2BC0);
        self.pallete_per_tile3 = self.parse_attr_to_tiles(0x2FC0);

        self.chr_tiles0 = self.render_chr(0x0000, CHR_0_X_Y.0, CHR_0_X_Y.1);
        self.chr_tiles1 = self.render_chr(0x1000, CHR_1_X_Y.0, CHR_1_X_Y.1);

        self.nametable0 = self.render_nametable(0x2000,
                                                NAMETABLE_0_X_Y.0,
                                                NAMETABLE_0_X_Y.1,
                                                self.pallete_per_tile0);
        self.nametable1 = self.render_nametable(0x2400,
                                                NAMETABLE_1_X_Y.0,
                                                NAMETABLE_1_X_Y.1,
                                                self.pallete_per_tile1);
        self.nametable2 = self.render_nametable(0x2800,
                                                NAMETABLE_2_X_Y.0,
                                                NAMETABLE_2_X_Y.1,
                                                self.pallete_per_tile2);
        self.nametable3 = self.render_nametable(0x2C00,
                                                NAMETABLE_3_X_Y.0,
                                                NAMETABLE_3_X_Y.1,
                                                self.pallete_per_tile3);
    }

    fn draw_all_sprites_in_a_very_stupid_manner(&mut self) {
        if self.mem.borrow_mut().draw_sprites() {
            let chr_for_sprites = match self.mem.borrow_mut().get_oam_chr_number() {
                0 => self.chr_tiles0,
                1 => self.chr_tiles1,
                _ => self.chr_tiles0
            };
            let universal_bg_color = self.get_universal_bg_color();
            let sprite_palette0 = self.get_palette(0x3F11, universal_bg_color);
            let sprite_palette1 = self.get_palette(0x3F15, universal_bg_color);
            let sprite_palette2 = self.get_palette(0x3F19, universal_bg_color);
            let sprite_palette3 = self.get_palette(0x3F1D, universal_bg_color);

            //64 sprites, 4 bytes each
            for tile_no in 0..64 {
                //TODO: self.mem.borrow_mut().should_use_big_sprites()
                let oam_adr_for_tile = tile_no * 4;
                let y = self.mem.borrow_mut().oam[oam_adr_for_tile as usize];
                if y < 0xEF {
                    let tile = chr_for_sprites[self.mem.borrow_mut().oam[(oam_adr_for_tile + 1) as usize] as usize];
                    let attr = self.mem.borrow_mut().oam[(oam_adr_for_tile + 2) as usize];
                    let x = self.mem.borrow_mut().oam[(oam_adr_for_tile + 3) as usize];
                    if attr & 0b100000 == 0 {
                        let pal_no = attr & 0b11;
                        let horiz_flip = attr & 0b01000000 > 0;
                        let vert_flip = attr & 0b10000000 > 0;
                        for i in 0..8 {
                            for j in 0..8 {
                                let mut x_final = (x as u32 + j) * 2;
                                if horiz_flip {
                                    x_final = (x as u32 + (7 - j)) * 2;
                                }
                                let mut y_final = (y as u32 + i) * 2;
                                if vert_flip {
                                    y_final = (y as u32 + (7 - i)) * 2;
                                }
                                if x_final < 512 && y_final < 480 {
                                    let color = tile[i as usize][j as usize];
                                    if color > 0 {
                                        let (r, g, b) = match pal_no {
                                            0 => sprite_palette0[color as usize],
                                            1 => sprite_palette1[color as usize],
                                            2 => sprite_palette2[color as usize],
                                            3 => sprite_palette3[color as usize],
                                            _ => sprite_palette0[color as usize]
                                        };
                                        self.canvas.put_pixel(x_final, y_final, im::Rgba([r, g, b, 255]));
                                        self.canvas.put_pixel(x_final + 1, y_final, im::Rgba([r, g, b, 255]));
                                        self.canvas.put_pixel(x_final + 1, y_final + 1, im::Rgba([r, g, b, 255]));
                                        self.canvas.put_pixel(x_final, y_final + 1, im::Rgba([r, g, b, 255]));
                                    }
                                }
                            }
                        }
                    }
                }
            }

//        self.mem.borrow_mut().set_sprite_0_hit(true);
        }
    }

    pub fn render(&mut self, r: piston_window::RenderArgs) {
        self.prepare_bg_stuff();
        self.draw_all_sprites_in_a_very_stupid_manner();
        self.texture.update(&self.canvas);

        let c = self.gl.draw_begin(r.viewport());
        graphics::clear([0.0, 0.0, 0.0, 1.0], &mut self.gl);

        graphics::image(&self.texture, c.transform, &mut self.gl);

        self.gl.draw_end();
    }

    fn parse_attr_to_tiles(&mut self, base_adr: u16) -> [u8; 960] {
        let mut pal_num_per_tile: [u8; 960] = [0; 960];
        for row_num in 0..30 {
            for col_num in 0..32 {
                let i = row_num * 32 + col_num;
                let quad_block_number = ((row_num / 4) * 8) + (col_num / 4);
                let block_number = ((row_num / 2) * 16) + (col_num / 2);
                let block_number_in_quad_block_bit = (((row_num / 2) % 2) << 1) + (block_number % 2);
                let quad_bit = self.mem.borrow_mut().read_vram((base_adr + (quad_block_number as u16)));
                let pal_number = (quad_bit >> ((block_number_in_quad_block_bit) * 2)) & 0b11;
                pal_num_per_tile[i as usize] = pal_number;
            }
        }
        pal_num_per_tile
    }

    fn get_universal_bg_color(&mut self) -> (u8, u8, u8) {
        let col_num = self.mem.borrow_mut().read_vram(0x3F00);
        get_rgb_color(col_num)
    }

    fn get_palette(&mut self, base_adr: u16, ubg: (u8, u8, u8))
                   -> [(u8, u8, u8); 4] {
        let mut pal = [(0, 0, 0); 4];
        pal[0] = ubg;
        for i in 0..3 {
            let col_num = self.mem.borrow_mut().read_vram(base_adr + i);
            pal[(i + 1) as usize] = get_rgb_color(col_num);
        }
        pal
    }

    fn render_nametable(&mut self, base_adr: u16, render_start_x: u32,
                        render_start_y: u32, palette_per_tile: [u8; 960]) -> [Tile; 960] {
        //FIXME: Only recalculate if there were changes in Nametable
        //Parse out nametable 0 (960 bytes; 32 tiles wide; 30 tiles high)
        let mut nametable = [[[0; 8]; 8]; 960];

        for rows in 0..30u16 {
            for cols in 0..32u16 {
                //Get the tile and save it
                let index = ((rows * 32) + cols);
                let tile_no = self.mem.borrow_mut().read_vram(index + base_adr);
                if self.mem.borrow_mut().use_chr_0() {
                    nametable[index as usize] = self.chr_tiles0[tile_no as usize];
                } else {
                    nametable[index as usize] = self.chr_tiles1[tile_no as usize];
                }

                //Render it out
                let tile_start_x = render_start_x + ((cols as u32) * 8);
                let tile_start_y = render_start_y + ((rows as u32) * 8);
                let pal_for_tile = match palette_per_tile[index as usize] {
                    0 => { self.bg_palette0 }
                    1 => { self.bg_palette1 }
                    2 => { self.bg_palette2 }
                    3 => { self.bg_palette3 }
                    _ => [(0, 0, 0); 4]
                };
                for i in 0..8 {
                    for j in 0..8 {
                        let x = (tile_start_x + j);
                        let y = (tile_start_y + i);
                        let color = nametable[index as usize][i as usize][j as usize];
                        let (r, g, b) = pal_for_tile[color as usize];
                        self.canvas.put_pixel(x, y, im::Rgba([r, g, b, 255]));
                    }
                }
            }
        }
        nametable
    }

    fn render_chr(&mut self, adr_base: u16, render_start_x: u32,
                  render_start_y: u32) -> [Tile; 256] {
        //FIXME: Only recalculate if there were changes in CHR
        //Parse out chr data and just render it to screen in b&w
        let mut ret_tiles = [[[0; 8]; 8]; 256];

        //Render CHR1 ($0000-$0FFF)
        for tile_no in 0..256 {
            //256 tiles, each takes up 16 bytes, and consists of two pictures of 8 bytes
            // that are later on added together to form the resulting picture
            let mut low_bits: [u8; 8] = [0; 8];
            let mut high_bits: [u8; 8] = [0; 8];

            for i in 0..8 {
                let low_bit_adr = adr_base + (tile_no * 16) + i;
                let high_bit_adr = low_bit_adr + 8;
                low_bits[i as usize] = self.mem.borrow_mut().read_vram(low_bit_adr);
                high_bits[i as usize] = self.mem.borrow_mut().read_vram(high_bit_adr);
            }

            //Render it out
            let tile_start_x = render_start_x + (tile_no as u32 % 16) * 8;
            let tile_start_y = render_start_y + (tile_no as u32 / 16) * 8;
            for i in 0..8 {
                for j in 0..8 {
                    let x = (tile_start_x + (7 - j));
                    let y = (tile_start_y + i);
                    let color =
                        (
                            ((low_bits[i as usize] >> (j as u8)) & 0b1) |
                                (((high_bits[i as usize] >> (j as u8)) & 0b1) << 1)
                        );
                    ret_tiles[tile_no as usize][i as usize][(7 - j) as usize] = color;
                    let (r, g, b) = self.bg_palette0[color as usize];
                    self.canvas.put_pixel(x, y, im::Rgba([r, g, b, 255]));
                }
            }
        }
        ret_tiles
    }
}