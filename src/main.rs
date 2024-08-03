pub mod font;
pub mod chip8;

fn main() {
    let mut chip8 = chip8::Chip8::new();
    chip8.load_rom(String::from("danm8ku.ch8"));
    chip8.start();
}
