pub mod font;
pub mod chip8;

fn main() {
    let mut chip8 = chip8::Chip8::new();
    chip8.load_rom(String::from("5-quirks.ch8"));
    chip8.start();
}
