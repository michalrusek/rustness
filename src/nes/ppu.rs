use std::cell::RefCell;
use std::rc::Rc;
use crate::nes::Mem;
use piston_window::{PistonWindow};

pub struct Ppu {
    pub mem: Rc<RefCell<Mem>>
}

impl Ppu {
    pub fn new(mem: &Rc<RefCell<Mem>>, window: &mut PistonWindow) -> Ppu {
        Ppu { mem: Rc::clone(mem)}
    }

    pub fn emulate(&mut self, cycles: u8) {}

    pub fn render(&mut self, c: piston_window::context::Context,
                  g: &mut piston_window::G2d) {
        //Clear whole screen
        piston_window::clear([0.0; 4], g);

        //Render the frame


        for i in 0..50_000 {
            let x = i % 500;
            let y = i / 500;
            piston_window::rectangle(
                [1.0, 0.0, 0.0, 1.0],
                [x as f64, y as f64, 1.0, 1.0],
                c.transform,
                g);
        }

//        piston_window::polygon([0.0, 0.0, 255.0, 255.0], [[11.0, 30.0]], c.transform, g);
//        piston_window::polygon([0.0, 0.0, 255.0, 255.0], [[12.0, 30.0]], c.transform, g);
//        piston_window::polygon([0.0, 0.0, 255.0, 255.0], [[13.0, 30.0]], c.transform, g);
    }
}