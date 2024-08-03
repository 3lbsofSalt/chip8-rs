# Minimalist Chip8 Emulator

This is chip8 emulator I wrote in rust. 
It's not the cleanest, and probably doesn't even have all of the necessary features, but I had a desire to
dip my toes in the emulation world and this accomplished that for me.

## Running The Emulator
Place your rom in the `/roms` directory. To select the rom you would like to run, navigate to `src/main.rs`
and input the name of the rom file without the `/roms` prefix on line 6.  
Run `cargo run` in the terminal and you'll be on your way.

## Configuration
Because different chip8 emulators have different idiosyncrasies, you may find it necessary to configure the emulator
speed or ambiguous instructions. Configuration variables may be found at the top of `/src/chip8.rs`. The variables
function as such:

- `DARK_COLOR` - This is the color that gets drawn when a pixel is considered "on"
- `LIGHT_COLOR` - This is the color drawn when a pixel is "off"
- `LOOPS_PER_SECOND` - Refresh rate. The speed at which a chip8 program runs in independant of this variable.
- `INSTRUCTIONS_PER_SECOND` - The rate at which instructions are executed. If a game needs to run faster or slower, this is the variable to update.
- `OLD_SHIFT_FUNCTIONALITY` - If true, on shift instructions it will place the value in the second register denoted and then perform the shift. Otherwise the value will not be placed in the second register.
- `B_JUMP_REG_OFFSET` - If true, B-type instructions will get their offset from the register listed in the second half-byte of the instruction. Otherwise it will get the offset from register 0.
- `MOVABLE_INDEX_ON_SAVE_LOAD` - If true, the index register will update as the software executes the load or store instructions. Note: There appears to be something wrong with this one. If it is true, things don't work quite right. This is future development.
- `WRAP_SPRITES` - If true, sprites will wrap to the other side of the screen when drawing them would put them past the edge. If false, the sprites will clip if drawn past the edge of the screen.

It is unfortunate to note that not every game seems to work on this emulator. I hope to remedy that in the future.

## References
I used several excellent refrences to be able to write this piece of software.
- https://tobiasvl.github.io/blog/write-a-chip-8-emulator/
- http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
- https://johnearnest.github.io/chip8Archive/?sort=platform
