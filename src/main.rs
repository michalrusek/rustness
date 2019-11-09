extern crate piston_window;
extern crate image as im;
mod nes;

use piston_window::*;
use crate::nes::Nes;
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::borrow::BorrowMut;

fn main() {
    let opengl = OpenGL::V4_5;

    let (width, height) = (800, 600);
    let mut window: PistonWindow =
        WindowSettings::new("Rustness!", (width, height))
            .exit_on_esc(true)
            .graphics_api(opengl)
            .build()
            .unwrap();

    //Initialize the emulator
    let mut n = Nes::new("./roms/nestest.nes", &mut window);

    //TODO: REMOVE THIS TESTING CODE
//    let file = File::open("./roms/nestest.log.txt").unwrap();
//    let reader = BufReader::new(file);
//    let mut lines_iter = reader.lines().map(|l| l.unwrap());
//    let mut line_number = 1;
    //END OF TODO

    //Loop through window events - this loops at *refresh_rate* right now
    //TODO: Make it loop at 60fps constantly
    let mut gl = opengl_graphics::GlGraphics::new(opengl);

    let mut canvas = im::ImageBuffer::new(width, height);
    let mut texture = opengl_graphics::Texture::from_image(&canvas, &opengl_graphics::TextureSettings::new());
    let img = Image::new().rect(graphics::rectangle::rectangle_by_corners(0.0, 0.0,width as f64, height as f64));

    while let Some(event) = window.next() {
        if let Some(r) = event.render_args() {
            texture.update(&canvas);

            let c = gl.draw_begin(r.viewport());
            graphics::clear([0.0, 0.0, 0.0, 1.0], &mut gl);

            graphics::image(&texture, c.transform, &mut gl);

            gl.draw_end();
        }

        if let Some(u) = event.update_args() {
            for i in 0..400_000 {
                let x = i % 800;
                let y = i / 800;
                canvas.put_pixel(x, y, im::Rgba([255, 255, 255, 255]));
            }
            n.emulate_frame();
//            assert_eq!(n.mem.borrow_mut().log_string, lines_iter.next().unwrap());
//            println!("Line {:?} is okay.", line_number);
//            line_number += 1;
        }
    }
}