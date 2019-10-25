extern crate piston_window;
mod nes;

use piston_window::*;
use crate::nes::Nes;
use std::fs::File;
use std::io::{BufReader, BufRead};

fn main() {
    let mut window: PistonWindow =
        WindowSettings::new("Rustness!", [640, 480])
            .exit_on_esc(true).build().unwrap();

    //Initialize the emulator
    let mut n = Nes::new("./roms/nestest.nes");

    //TODO: REMOVE THIS TESTING CODE
    let file = File::open("./roms/nestest.log.txt").unwrap();
    let reader = BufReader::new(file);
    let mut lines_iter = reader.lines().map(|l| l.unwrap());
    let mut line_number = 1;
    //END OF TODO

    //Loop through window events - this loops at *refresh_rate* right now
    //TODO: Make it loop at 60fps constantly
    while let Some(event) = window.next() {
        if let Some(r) = event.render_args() {
            window.draw_2d(&event, |context, graphics, _device| {
                n.render_frame(context, graphics);
            });
        }

        if let Some(u) = event.update_args() {
            n.emulate_frame();
            assert_eq!(n.mem.borrow_mut().log_string, lines_iter.next().unwrap());
            println!("Line {:?} is okay.", line_number);
            line_number += 1;
        }
    }
}