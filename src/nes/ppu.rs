use std::cell::RefCell;
use std::rc::Rc;
use crate::nes::Mem;
use piston_window::{PistonWindow, Image};
use opengl_graphics::OpenGL;

type Tile = [[u8; 8]; 8];

pub struct Ppu {
    pub mem: Rc<RefCell<Mem>>,
    gl: opengl_graphics::GlGraphics,
    canvas: im::ImageBuffer<im::Rgba<u8>, Vec<u8>>,
    texture: opengl_graphics::Texture,
    img: Image,
    chr_tiles: [Tile; 256],
    nametable: [Tile; 960],
    current_scanline: i32,
    cycles_total: u64,
    cycles_for_current_scanline: u16,
    triggered_nmi_this_scanline: bool
}

const CYCLES_PER_SCANLINE: u16 = 341;

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
            chr_tiles: [[[0; 8]; 8]; 256],
            nametable: [[[0; 8]; 8]; 960],
            current_scanline: -1,
            cycles_total: 0,
            cycles_for_current_scanline: 0,
            triggered_nmi_this_scanline: true
        }
    }

    pub fn emulate(&mut self, cycles: u8) {
        self.cycles_total = self.cycles_total + cycles as u64;
        self.cycles_for_current_scanline += cycles as u16;
        if self.cycles_for_current_scanline >= CYCLES_PER_SCANLINE {
            self.cycles_for_current_scanline -= CYCLES_PER_SCANLINE;
            self.current_scanline += 1;
            self.triggered_nmi_this_scanline = false;
        }
        if self.current_scanline == 241 && !self.triggered_nmi_this_scanline {
            self.mem.borrow_mut().set_vblank(true);
            self.mem.borrow_mut().set_nmi_occured(true);
            self.triggered_nmi_this_scanline = true;
        }
        if self.current_scanline == 262 {
            self.mem.borrow_mut().set_vblank(false);
            self.mem.borrow_mut().set_nmi_occured(false);
            self.current_scanline = -1;
        }
    }

    pub fn render(&mut self, r: piston_window::RenderArgs) {
//        for i in 0..400_000 {
//            let x = i % 800;
//            let y = i / 800;
//            self.canvas.put_pixel(x, y, im::Rgba([255, 255, 255, 255]));
//        }

        self.render_chr();
        let x: u32 = 500;
        let y: u32 = 128;
        self.render_nametable(0x2000, x, y);
//        self.render_nametable(0x2400, x, y);
//        self.render_nametable(0x2800, x, y);
//        self.render_nametable(0x2C00, x, y);

        self.texture.update(&self.canvas);

        let c = self.gl.draw_begin(r.viewport());
        graphics::clear([0.0, 0.0, 0.0, 1.0], &mut self.gl);

        graphics::image(&self.texture, c.transform, &mut self.gl);

        self.gl.draw_end();
    }

    fn render_nametable(&mut self, base_adr: u16, render_start_x: u32, render_start_y: u32) {
        //FIXME: Only recalculate if there were changes in Nametable
        //Parse out nametable 0 (960 bytes; 32 tiles wide; 30 tiles high)
        //TODO: parse nametables 1, 2, 3 as well

        for rows in 0..30u16 {
            for cols in 0..32u16 {
                //Get the tile and save it
                let index = ((rows * 32) + cols);
                let tile_no = self.mem.borrow_mut().read_vram(index + base_adr);
                self.nametable[index as usize] = self.chr_tiles[tile_no as usize];

                //Render it out
                let tile_start_x = render_start_x + ((cols as u32) * 8);
                let tile_start_y = render_start_y + ((rows as u32) * 8);
                for i in 0..8 {
                    for j in 0..8 {
                        let x = (tile_start_x + j);
                        let y = (tile_start_y + i);
                        let color = self.chr_tiles[tile_no as usize][i as usize][j as usize];
                        self.canvas.put_pixel(x, y, im::Rgba([80, color * (255 / 4), color * (255 / 4), 255]));
                    }
                }
            }
        }
    }

    fn render_chr(&mut self) {
        //FIXME: Only recalculate if there were changes in CHR
        //Parse out chr data and just render it to screen in b&w
        let render_start_x: u32 = 500;
        let render_start_y: u32 = 0;

        //Render CHR1 ($0000-$0FFF)
        //TODO: Parse/render CHR2 as well
        let adr_base = 0x0;
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
                    self.chr_tiles[tile_no as usize][i as usize][(7 - j) as usize] = color;
                    self.canvas.put_pixel(x, y, im::Rgba([color * (255 / 4), color * (255 / 4), color * (255 / 4), 255]));
                }
            }
        }
    }
}