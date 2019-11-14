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

    let (width, height) = (1280, 720);
    let mut window: PistonWindow =
        WindowSettings::new("Rustness!", (width, height))
            .exit_on_esc(true)
            .graphics_api(opengl)
            .build()
            .unwrap();

    //Initialize the emulator
    let mut n = Nes::new("./roms/lode.nes", &mut window, opengl, (width, height));

    //TODO: REMOVE THIS TESTING CODE
//    let file = File::open("./roms/nestest.log.txt").unwrap();
//    let reader = BufReader::new(file);
//    let mut lines_iter = reader.lines().map(|l| l.unwrap());
//    let mut line_number = 1;
    //END OF TODO

    //Loop through window events - this loops at *refresh_rate* right now
    //TODO: Make it loop at 60fps constantly


    while let Some(event) = window.next() {
        if let Some(r) = event.render_args() {
            n.render_frame(r);
        }

        if let Some(u) = event.update_args() {
            n.emulate_frame();
//            assert_eq!(n.mem.borrow_mut().log_string, lines_iter.next().unwrap());
//            println!("Line {:?} is okay.", line_number);
//            line_number += 1;
        }

        if let Some(Button::Keyboard(k)) = event.press_args() {
            //Send key presses to the game
            n.button_press(k);
        }
        if let Some(Button::Keyboard(k)) = event.release_args() {
            //Send lif to game
            n.button_lift(k);
        }
    }
}