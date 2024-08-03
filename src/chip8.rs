use std::{fs, io::Read, path::Path, thread, time::{Duration, Instant}};
use rand::Rng;
use sdl2::{audio::{AudioCallback, AudioDevice, AudioSpecDesired}, event::Event, keyboard::Scancode, pixels::Color, rect::Rect, render::Canvas, video::Window, EventPump};

use crate::font::write_font;

// CONFIG
const LOOPS_PER_SECOND: u128 = 240;
const INSTRUCTIONS_PER_SECOND: u128 = 700; 

const OLD_SHIFT_FUNCTIONALITY: bool = true;
const B_JUMP_REG_OFFSET: bool = false;
const MOVABLE_INDEX_ON_SAVE_LOAD: bool = false;
const WRAP_SPRITES: bool = true;

pub struct Chip8 {
    pixels: Vec<Vec<bool>>,
    canvas: Canvas<Window>,
    audio_device: AudioDevice<SquareWave>,
    pixel_width: u32,
    events: EventPump,
    memory: [u8; 4096],
    registers: [u8; 16],
    register_i: u16,
    pc: u16,
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
    quit: bool
}

impl Chip8 {

    pub fn start(&mut self) {
        let mut quit = false;
        while !quit {
            quit = self.single_loop();
        }
    }

    fn single_instruction(&mut self) {
        self.check_keys_pressed();
        let instruction: u16;
        instruction = u16::from(self.memory[self.pc as usize]) << 8 | u16::from(self.memory[(self.pc + 1) as usize]) << 0;
        println!("Instruction: {:04X}, PC: {:012X}", instruction, self.pc);

        self.pc += 2;

        if self.pc == (0x3BC + 0x002) {
            // Spot for easy breakpoint based on pc addresses
            println!("breakpoint");

        }

        let first_nibble = (instruction & 0xF000) >> 12;

        match first_nibble {
            0x0 => {
                if instruction == 0x00E0 { self.clear(); }
                if instruction == 0x00EE { self.pc = self.stack.pop().unwrap(); }
            },
            0x1 => { 

                let jump_location = 0x0FFF & instruction;
                self.pc = jump_location;
            },
            0x2 => { 
                self.stack.push(self.pc);
                let jump_location = 0x0FFF & instruction;
                self.pc = jump_location;
            },
            0x3 => { 
                let reg = ((instruction & 0x0F00) >> 8) as usize;
                let val = (instruction & 0x00FF) as u8;
                if self.registers[reg] == val { self.pc += 2; }
            },
            0x4 => { 
                let reg = ((instruction & 0x0F00) >> 8) as usize;
                let val = (instruction & 0x00FF) as u8;
                if self.registers[reg] != val { self.pc += 2; }
            },
            0x5 => { 
                let reg_x = ((instruction & 0x0F00) >> 8) as usize;
                let reg_y = ((instruction & 0x00F0) >> 4) as usize;
                if self.registers[reg_x] == self.registers[reg_y] { self.pc += 2; }
            },
            0x6 => {
                let second_nibble = (instruction & 0x0F00) >> 8;
                let value = instruction & 0x00FF;
                self.registers[second_nibble as usize] = value as u8;
            },
            0x7 => { 
                let second_nibble = (instruction & 0x0F00) >> 8;
                let value = instruction & 0x00FF;
                self.registers[second_nibble as usize] = self.registers[second_nibble as usize].wrapping_add(value as u8);
            },
            0x8 => {
                let last_nibble = instruction & 0x000F;
                let x = ((instruction & 0x0F00) >> 8) as usize;
                let y = ((instruction & 0x00F0) >> 4) as usize;
                match last_nibble {
                    0x0000 => { self.registers[x] = self.registers[y]; },
                    0x0001 => { self.registers[x] = self.registers[x] | self.registers[y]; },
                    0x0002 => { self.registers[x] = self.registers[x] & self.registers[y]; },
                    0x0003 => { self.registers[x] = self.registers[x] ^ self.registers[y]; },
                    0x0004 => { 
                        let flag = if self.registers[x].checked_add(self.registers[y]) == None { 1 } else { 0 };
                        self.registers[x] = self.registers[x].wrapping_add(self.registers[y]); 
                        self.registers[0xF] = flag;
                    },
                    0x0005 => {
                        let flag = if self.registers[x] >= self.registers[y] { 1 } else { 0 };
                        self.registers[x] = self.registers[x].wrapping_sub(self.registers[y]); 
                        self.registers[0xF] = flag;
                    },
                    0x0006 => {
                        let flag = if (0b00000001 & self.registers[x]) > 0 { 1 } else { 0 };
                        if OLD_SHIFT_FUNCTIONALITY { self.registers[x] = self.registers[y]; }
                        self.registers[x] >>= 1;
                        self.registers[0xF] = flag;
                    },
                    0x0007 => {
                        let flag = if self.registers[y] >= self.registers[x] { 1 } else { 0 };
                        self.registers[x] = self.registers[y].wrapping_sub(self.registers[x]); 
                        self.registers[0xF] = flag;
                    },
                    0x000E => {
                        if OLD_SHIFT_FUNCTIONALITY { self.registers[x] = self.registers[y]; }
                        let flag = if (0b10000000 & self.registers[x]) > 0 { 1 } else { 0 };
                        self.registers[x] <<= 1;
                        self.registers[0xF] = flag;
                    },
                    _ => { panic!("There should absolutely be a last nibble on an 0x8XYN instruction")}
                }
            },
            0x9 => { 
                let reg_x = ((instruction & 0x0F00) >> 8) as usize;
                let reg_y = ((instruction & 0x00F0) >> 4) as usize;
                if self.registers[reg_x] != self.registers[reg_y] { self.pc += 2; }
            },
            0xA => { 

                self.register_i = instruction & 0x0FFF; 
            },
            0xB => {
                let mut offset = self.registers[0];
                if B_JUMP_REG_OFFSET {
                    offset = self.registers[((0x0F00 & instruction) >> 8) as usize];
                }

                self.pc = offset as u16 + (0x0FFF & instruction);
            },
            0xC => {
                let reg = ((0x0F00 & instruction) >> 8) as usize;
                let nn = 0x00FF & instruction;
                let num = rand::thread_rng().gen_range(0..0xFF);

                self.registers[reg] = (nn & num) as u8;

            },
            0xD => { // Display instruction
                let x_r = (0x0F00 & instruction) >> 8; 
                let y_r = (0x00F0 & instruction) >> 4;
                let n = (0x000F & instruction) >> 0;

                let x = self.registers[x_r as usize];
                let y = self.registers[y_r as usize];

                self.draw_sprite(self.register_i, n as u8, x, y);
            },
            0xE => { // Skip if key instructions
                let keys_pressed = self.check_keys_pressed();
                let key_reg = ((0x0F00 & instruction) >> 8) as usize;
                let key = self.registers[key_reg];
                let which = 0x00FF & instruction;
                //println!("{:?}", keys_pressed);
                if which == 0x009E {

                    if keys_pressed[key as usize] {
                        self.pc += 2;
                    }
                } else if which == 0x00A1 {
                    if !keys_pressed[key as usize] {
                        self.pc += 2;
                    }
                }
            },
            0xF => {
                let second_half = 0x00FF & instruction;
                let reg = ((0x0F00 & instruction) >> 8) as usize;
                match second_half {
                    0x07 => { self.registers[reg] = self.delay_timer; },
                    0x0A => { // Repeat until a key is pressed
                        
                        let keys_pressed = self.check_keys_pressed();
                        let mut key_pressed = 1000;
                        for (i, key) in keys_pressed.as_ref().iter().enumerate() {
                            if *key {
                                key_pressed = i;
                                break;
                            }
                        }

                        if key_pressed >= 1000 { 
                            self.pc -= 2; 
                        } else {
                            self.registers[reg] = key_pressed as u8;
                        }
                    },
                    0x15 => { self.delay_timer = self.registers[reg]; }
                    0x18 => { self.sound_timer = self.registers[reg]; }
                    0x1E => { 
                        if self.register_i + self.registers[reg] as u16 >= 0x1000 { self.registers[0xF] = 1; }
                        self.register_i += self.registers[reg] as u16;
                    },
                    0x29 => {
                        let character = self.registers[reg] & 0x0F;
                        // 0x050 is the first address for the font, and each character is 5 bytes long.
                        // See font.rs
                        self.register_i = 0x050 + (5 * character) as u16;

                    },
                    0x33 => {
                        let num = self.registers[reg];
                        self.memory[self.register_i as usize] = num / 100;
                        self.memory[(self.register_i + 1) as usize] = (num / 10) % 10;
                        self.memory[(self.register_i + 2) as usize] = num % 10;
                    }, 
                    0x55 => { 

                        for num in 0..=reg { 
                            self.memory[self.register_i as usize + num] = self.registers[num]; 
                            if MOVABLE_INDEX_ON_SAVE_LOAD { self.register_i += 1; }
                        }
                    },
                    0x65 => { 
                        for num in 0..=reg { 
                            self.registers[num] = self.memory[self.register_i as usize + num]; 
                            if MOVABLE_INDEX_ON_SAVE_LOAD { self.register_i += 1; }
                        }
                    }
                    _ => { panic!("There was an error with an 0xF type instruction!"); }
                }
            },
            _ => { panic!("The instructions were decoded or written wrong."); }
        }
    } 

    fn single_loop(&mut self) -> bool {
        let loop_start = Instant::now();

        self.handle_delay_timer();
        self.handle_sound_timer();

        for _ in 0..(INSTRUCTIONS_PER_SECOND / LOOPS_PER_SECOND) {
            self.single_instruction();
        }
        self.display();

        let one_loop_nano: u128 = 1_000_000_000 / LOOPS_PER_SECOND;
        let loop_length: u128 = loop_start.elapsed().as_nanos();
        let time_to_wait = if one_loop_nano > loop_length { one_loop_nano - loop_length } else { 0 };
        thread::sleep(Duration::from_nanos(time_to_wait as u64));
        return self.quit;
    }

    fn check_keys_pressed(&mut self) -> [bool; 16] {
        let mut keys_pressed: [bool; 16] = [false; 16];
        let mut quit = false;

        for scancode in self.events.keyboard_state().pressed_scancodes() {
            match scancode {
                Scancode::Num1 => { keys_pressed[0x1] = true; },
                Scancode::Num2 => { keys_pressed[0x2] = true; },
                Scancode::Num3 => {keys_pressed[0x3] = true;  },
                Scancode::Num4 => { keys_pressed[0xC] = true; },
                Scancode::Q => { keys_pressed[0x4] = true; },
                Scancode::W => { keys_pressed[0x5] = true; },
                Scancode::E => { keys_pressed[0x6] = true; },
                Scancode::R => { keys_pressed[0xD] = true; },
                Scancode::A => { keys_pressed[0x7] = true; },
                Scancode::S => { keys_pressed[0x8] = true; },
                Scancode::D => { keys_pressed[0x9] = true; },
                Scancode::F => { keys_pressed[0xE] = true; },
                Scancode::Z => { keys_pressed[0xA] = true; },
                Scancode::X => { keys_pressed[0x0] = true; },
                Scancode::C => { keys_pressed[0xB] = true; },
                Scancode::V => { keys_pressed[0xF] = true; },
                _ => {}
            }
        }

        for event in self.events.poll_iter() {
            match event {
                Event::Quit { .. } => { quit = true; }
                _ => {}
            }
        }

        if !self.quit { self.quit = quit; }

        return keys_pressed;
    }

    pub fn new() -> Self {

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

        let mut memory: [u8; 4096] = [0; 4096];

        write_font(&mut memory);
        //println!("{:?}", memory);
        let registers: [u8; 16] = [0; 16];

        Chip8 {
            canvas,
            pixels,
            audio_device: device,
            pixel_width,
            events: event_pump,
            memory,
            registers,
            delay_timer: 0,
            pc: 0x200,
            sound_timer: 0,
            register_i: 0x0,
            stack: Vec::new(),
            quit: false
        }
    }

    pub fn draw_sprite(&mut self, i: u16, bytes: u8, offset_x: u8, offset_y: u8) {
        self.registers[0xF] = 0;
        let offset_x = offset_x % 64;
        let offset_y = offset_y % 32;
        for byte_num in 0..bytes { // Each byte is a horizontal row in the sprite
            let byte: u8;
            byte = self.memory[(i + byte_num as u16) as usize];
            for bit in 0..8 { // Each row of the sprite can be up to 8 columns long
                let mask = 1 << 7 - bit;
                let bit_set = (mask & byte) > 0;
                if bit_set {
                    let mut y_coord = (offset_y + byte_num as u8) as usize;
                    let mut x_coord = (offset_x + bit) as usize;
                    if (x_coord >= 64 || y_coord >= 32) && !WRAP_SPRITES { 
                        continue; 
                    } else {
                        x_coord %= 64;
                        y_coord %= 32;
                    }
                    if self.pixels[y_coord][x_coord] { self.registers[0xF] = 1; }
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

    pub fn display(&mut self) {
        
        let black = Color::RGB(0, 0, 0);
        let white = Color::RGB(255, 255, 255);

        for (y_index ,row) in self.pixels.iter().enumerate() {
            for (x_index, pixel) in row.iter().enumerate() {
                let y = (y_index * self.pixel_width as usize) as i32;
                let x = (x_index * self.pixel_width as usize) as i32;
                let rect = Rect::new(x, y, self.pixel_width, self.pixel_width);

                if *pixel {
                    self.canvas.set_draw_color(white);
                } else {
                    self.canvas.set_draw_color(black);
                }

                let _ = self.canvas.fill_rect(rect);
            }
        }

        self.canvas.present();

    }

    pub fn load_rom(&mut self, name: String) {
        for mem in 0x200..4096 {
            self.memory[mem] = 0;
        }
        let rom_data = fs::File::open(Path::new("./roms/").join(name)).unwrap();
        let rom_data: Vec<Result<u8, std::io::Error>> = rom_data.bytes().collect();

        for (offset, byte) in rom_data.iter().enumerate() {
            let byte = byte.as_ref().unwrap();
            self.memory[0x200 + offset] = *byte;
        }
    }

    fn handle_delay_timer(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
    }

    fn handle_sound_timer(&mut self) {
        if self.sound_timer > 0 {
            self.audio_device.resume();
            self.sound_timer -= 1;
        } else {
            self.audio_device.pause();
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
