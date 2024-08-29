mod cpu;

use std::env;
use std::fs::File;
use std::io::Read;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;

const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (cpu::SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (cpu::SCREEN_HEIGHT as u32) * SCALE;
const TICKS_PER_FRAME: usize = 10;

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

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut chip8_inst = cpu::Emu::new();
    let mut rom = File::open(&args[1]).expect("Unable to open file");
    let mut buffer = Vec::new();
    rom.read_to_end(&mut buffer).unwrap();
    chip8_inst.load(&buffer);

    'gameloop: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit{..} => {
                    break 'gameloop;
                },
                _ => ()
            }
        }
        
        for _ in 0.. TICKS_PER_FRAME {
            chip8_inst.tick();
        }
        draw_screen(&chip8_inst, &mut canvas);
    }
}

fn draw_screen(emu: &cpu::Emu, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let screen_buf = emu.get_display();

    canvas.set_draw_color(Color::RGB(255, 255, 255));

    for (i, pixel) in screen_buf.iter().enumerate() {
        if *pixel {
            let x = (i % cpu::SCREEN_WIDTH) as u32;
            let y = (i / cpu::SCREEN_WIDTH) as u32;

            let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            canvas.fill_rect(rect).unwrap();
        }
    }
}