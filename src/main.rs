extern crate piston_window;
mod nes;

use piston_window::*;
use crate::nes::Nes;

fn main() {
    let mut window: PistonWindow =
        WindowSettings::new("Rustness!", [640, 480])
            .exit_on_esc(true).build().unwrap();

    //Initialize the emulator
    let mut n = Nes::new("./roms/nestest.nes");

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
        }
    }
}