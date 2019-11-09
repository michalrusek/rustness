use std::cell::RefCell;
use std::rc::Rc;
use crate::nes::Mem;
use piston_window::{PistonWindow, Image};
use opengl_graphics::OpenGL;

pub struct Ppu {
    pub mem: Rc<RefCell<Mem>>,
    gl: opengl_graphics::GlGraphics,
    canvas: im::ImageBuffer<im::Rgba<u8>, Vec<u8>>,
    texture: opengl_graphics::Texture,
    img: Image
}

impl Ppu {
    pub fn new(mem: &Rc<RefCell<Mem>>, window: &mut PistonWindow, opengl: OpenGL,
               (width, height): (u32, u32)) -> Ppu {
        let mut gl = opengl_graphics::GlGraphics::new(opengl);

        let mut canvas = im::ImageBuffer::new(width, height);
        let mut texture = opengl_graphics::Texture::from_image(&canvas, &opengl_graphics::TextureSettings::new());
        let img = Image::new().rect(graphics::rectangle::rectangle_by_corners(0.0, 0.0,width as f64, height as f64));
        Ppu { mem: Rc::clone(mem), gl, canvas, texture, img}
    }

    pub fn emulate(&mut self, cycles: u8) {

    }

    pub fn render(&mut self, r: piston_window::RenderArgs) {
//        for i in 0..400_000 {
//            let x = i % 800;
//            let y = i / 800;
//            self.canvas.put_pixel(x, y, im::Rgba([255, 255, 255, 255]));
//        }

        self.render_chr();

        self.texture.update(&self.canvas);

        let c = self.gl.draw_begin(r.viewport());
        graphics::clear([0.0, 0.0, 0.0, 1.0], &mut self.gl);

        graphics::image(&self.texture, c.transform, &mut self.gl);

        self.gl.draw_end();
    }

    fn render_chr(&mut self) {
        //Parse out chr data and just render it to screen in b&w
        //TODO: Add palettes here
        let render_start_x: u16 = 600;
        let render_start_y: u16 = 0;

        //Render CHR1 ($0000-$0FFF)
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

            //TODO: Render it out
            let tile_start_x = render_start_x + (tile_no % 16) * 8;
            let tile_start_y = render_start_y + (tile_no / 16) * 8;
            for i in 0..8 {
                for j in 0..8 {
                    let x = (tile_start_x + (7 - j) as u16) as u32;
                    let y = (tile_start_y + i) as u32;
                    let color = ((low_bits[i as usize] >> (j as u8)) & 0b1) * 255;
                    self.canvas.put_pixel(x, y, im::Rgba([color, color, color, 255]));
                }
            }
        }
    }
}