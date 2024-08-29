mod cpu;

use std::env;
use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (cpu::SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (cpu::SCREEN_HEIGHT as u32) * SCALE;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: cargo run path/to/game");
        return;
    }

    let sdl_context = sdl2::init().unwrap();
    let video_subsys = sdl_context.video().unwrap();
    let window = video_subsys
        .window("yachip8emu", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    
        let mut canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .unwrap();
        
        canvas.clear();
        canvas.present();
}