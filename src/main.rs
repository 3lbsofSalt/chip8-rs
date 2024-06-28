pub mod font;

use std::{alloc::{alloc, Layout}, fs, io::Read, path::Path, thread, time::{Duration, Instant}};
use sdl2::{audio::{AudioCallback, AudioDevice, AudioSpecDesired}, keyboard::Scancode, pixels::Color, rect::Rect, render::Canvas, video::Window, AudioSubsystem, EventPump};
use sdl2::event::Event;

use crate::font::write_font;

const LOOPS_PER_SECOND: u64 = 60;
const INSTRUCTIONS_PER_SECOND: u64 = 700; 

fn main() {
    let memory_layout = Layout::from_size_align(4096, 8).unwrap();
    let ptr;
    unsafe{
        ptr = alloc(memory_layout);
    }

    write_font(ptr);

    let rom_name = "IBM Logo.ch8";
    let rom_data = fs::File::open(Path::new("./roms/").join(rom_name)).unwrap();
    let rom_data: Vec<Result<u8, std::io::Error>> = rom_data.bytes().collect();
    for (offset, byte) in rom_data.iter().enumerate() {
        let byte = byte.as_ref().unwrap();
        unsafe {
            ptr.add(0x200 + offset).write(*byte);
        }
    }

    let mut screen = init_display();

    let stack: Vec<u16> = Vec::new();
    let mut delay_timer: u8 = 0;
    let mut sound_timer: u8 = 0;

    let mut registers: Vec<u8> = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut register_i: u16 = 0;
    let mut pc = 0x200;

    'running: loop {

        let loop_start = Instant::now();

        if delay_timer > 0 { delay_timer -= 1; }
        if sound_timer > 0 {
            // Play sound here
            screen.audio_device.resume();
            sound_timer -= 1;
        } else { screen.audio_device.pause(); }

        for event in screen.events.poll_iter() {
            match event {
                Event::Quit { .. } => { break 'running }
                Event::KeyDown { scancode: Some(Scancode::Num1), .. } => { },
                Event::KeyDown { scancode: Some(Scancode::Num2), .. } => { },
                Event::KeyDown { scancode: Some(Scancode::Num3), .. } => { },
                Event::KeyDown { scancode: Some(Scancode::Num4), .. } => { },
                Event::KeyDown { scancode: Some(Scancode::Q), .. } => { },
                Event::KeyDown { scancode: Some(Scancode::W), .. } => { },
                Event::KeyDown { scancode: Some(Scancode::E), .. } => { },
                Event::KeyDown { scancode: Some(Scancode::R), .. } => { },
                Event::KeyDown { scancode: Some(Scancode::A), .. } => { },
                Event::KeyDown { scancode: Some(Scancode::S), .. } => { },
                Event::KeyDown { scancode: Some(Scancode::D), .. } => { },
                Event::KeyDown { scancode: Some(Scancode::F), .. } => { },
                Event::KeyDown { scancode: Some(Scancode::Z), .. } => { },
                Event::KeyDown { scancode: Some(Scancode::X), .. } => { },
                Event::KeyDown { scancode: Some(Scancode::C), .. } => { },
                Event::KeyDown { scancode: Some(Scancode::V), .. } => { },
                _ => {}
            }
        }

        // fetch decode execute loop
        for _ in 0..(INSTRUCTIONS_PER_SECOND / LOOPS_PER_SECOND) {
            let instruction: u16;
            unsafe {
                instruction = u16::from(ptr.add(pc).read()) << 8 | u16::from(ptr.add(pc + 1).read()) << 0;
            }
            pc += 2;

            let first_nibble = (instruction & 0xF000) >> 12;

            match first_nibble {
                0x0 => {
                    if instruction == 0x00E0 { screen.clear(); }
                },
                0x1 => { 
                    let jump_location = 0x0FFF & instruction;
                    pc = jump_location as usize;
                },
                0x2 => { },
                0x3 => { },
                0x4 => { },
                0x5 => { },
                0x6 => {
                    let second_nibble = (instruction & 0x0F00) >> 8;
                    let value = instruction & 0x00FF;
                    registers[second_nibble as usize] = value as u8;
                },
                0x7 => { 
                    let second_nibble = (instruction & 0x0F00) >> 8;
                    let value = instruction & 0x00FF;
                    registers[second_nibble as usize] += value as u8;
                },
                0x8 => { },
                0x9 => { },
                0xA => { 
                    register_i = instruction & 0x0FFF; 
                    println!("RI: {:04X}", register_i);
                },
                0xB => { },
                0xC => { },
                0xD => {
                    let x_r = (0x0F00 & instruction) >> 8; 
                    let y_r = (0x00F0 & instruction) >> 4;
                    let n = (0x000F & instruction) >> 0;

                    let x = registers[x_r as usize];
                    let y = registers[y_r as usize];

                    screen.draw_sprite(register_i, n as u8, x, y, ptr);
                },
                0xE => { },
                0xF => { },
                _ => { panic!("The instructions were decoded or written wrong."); }
            }
        } 

        display(&mut screen);

        let time_to_wait: u64 = 1 / LOOPS_PER_SECOND - loop_start.elapsed().as_secs();
        thread::sleep(Duration::from_secs(time_to_wait));
    }

}

fn init_display() -> Screen {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let event_pump = sdl_context.event_pump().unwrap();

    let window = video_subsystem.window("Chip-8 Emulator", 64 * 16, 32 * 16)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(255, 255, 255));

    Screen::new(canvas, audio_subsystem, event_pump)
}

fn display(screen: &mut Screen) {

    let black = Color::RGB(0, 0, 0);
    let white = Color::RGB(255, 255, 255);

    for (y_index ,row) in screen.pixels.iter().enumerate() {
        for (x_index, pixel) in row.iter().enumerate() {
            let y = (y_index * screen.pixel_width as usize) as i32;
            let x = (x_index * screen.pixel_width as usize) as i32;
            let rect = Rect::new(x, y, screen.pixel_width, screen.pixel_width);

            if *pixel {
                screen.canvas.set_draw_color(white);
            } else {
                screen.canvas.set_draw_color(black);
            }

            let _ = screen.canvas.fill_rect(rect);
        }
    }

    screen.canvas.present();
}

struct Screen {
    pixels: Vec<Vec<bool>>,
    canvas: Canvas<Window>,
    audio_device: AudioDevice<SquareWave>,
    pixel_width: u32,
    events: EventPump
}

impl Screen {
    pub fn new(canvas: Canvas<Window>, audio_subsystem: AudioSubsystem, events: EventPump) -> Self {
        let size = canvas.window().size(); 
        let pixel_width = size.0 / 64;
        let mut pixels = Vec::new();
        // Row
        for i in 0..32 {
            pixels.push(Vec::new());
            // Column
            for _ in 0..64 {
                pixels[i].push(false);
            }
        }

        let canvas = canvas;

        let desired_spec = AudioSpecDesired {
            freq: Some(07640),
            channels: Some(1),
            samples: None
        };

        let device = audio_subsystem.open_playback(None, &desired_spec, |spec| {
            SquareWave {
                phase_inc: 440.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.05
            }
        }).unwrap();

        Screen {
            canvas,
            pixels,
            audio_device: device,
            pixel_width,
            events,
        }
    }

    pub fn draw_sprite(&mut self, i: u16, bytes: u8, offset_x: u8, offset_y: u8, ptr: *mut u8) {
        for byte_num in 0..bytes {
            let byte: u8;
            unsafe {
                byte = ptr.add((i + byte_num as u16) as usize).read();
            }
            println!("byte: {:02X} at {:04X} for i = {:04X}", byte, i + byte_num as u16, i);
            for bit in 0..8 {
                let mask = 1 << bit;
                let bit_set = (mask & byte) > 0;
                if bit_set {
                    let y_coord = (offset_y + byte_num as u8) as usize;
                    let x_coord = (offset_x + bit) as usize;
                    self.pixels[y_coord][x_coord] = !self.pixels[y_coord][x_coord];
                }
            }
        }
    }

    pub fn clear(&mut self) {
        for row in self.pixels.iter_mut() {
            for pixel in row.iter_mut() {
                *pixel = false;
            }
        }
    }
}

  
struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = if self.phase <= 0.5 {
                self.volume
            } else {
                -self.volume
            };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}
