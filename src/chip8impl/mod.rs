extern crate std;
extern crate time;
extern crate sdl;

use std::default::Default;
use std::slice::bytes;
use std::io::File;
use std::rand;
use std::io::fs::PathExtensions;
use sdl::video::Surface;
use std::io::Timer;
use std::time::Duration;

#[cfg(test)]
mod tests;

const STACK_SIZE:      uint = 16;
const MEMORY_SIZE:     uint = 4096;
const REGISTER_COUNT:  uint = 16;
const SCREEN_WIDTH:    uint = 64;
const SCREEN_HEIGHT:   uint = 32;
const KEY_COUNT:       uint = 16;
const FONTSET_SIZE:    uint = 80;
const FONT_DIGIT_SIZE: u16  = 5;

static FONTSET : [u8, ..FONTSET_SIZE] =
    [0xF0, 0x90, 0x90, 0x90, 0xF0,  // 0
     0x20, 0x60, 0x20, 0x20, 0x70,  // 1
     0xF0, 0x10, 0xF0, 0x80, 0xF0,  // 2
     0xF0, 0x10, 0xF0, 0x10, 0xF0,  // 3
     0x90, 0x90, 0xF0, 0x10, 0x10,  // 4
     0xF0, 0x80, 0xF0, 0x10, 0xF0,  // 5
     0xF0, 0x80, 0xF0, 0x90, 0xF0,  // 6
     0xF0, 0x10, 0x20, 0x40, 0x40,  // 7
     0xF0, 0x90, 0xF0, 0x90, 0xF0,  // 8
     0xF0, 0x90, 0xF0, 0x10, 0xF0,  // 9
     0xF0, 0x90, 0xF0, 0x90, 0x90,  // A
     0xE0, 0x90, 0xE0, 0x90, 0xE0,  // B
     0xF0, 0x80, 0x80, 0x80, 0xF0,  // C
     0xE0, 0x90, 0x90, 0x90, 0xE0,  // D
     0xF0, 0x80, 0xF0, 0x80, 0xF0,  // E
     0xF0, 0x80, 0xF0, 0x80, 0x80]; // F

pub struct Chip8 {
    pc          : u16,
    i           : u16,
    delay_timer : u8,
    sound_timer : u8,
    sp          : u8,
    v           : [u8, ..REGISTER_COUNT],
    mem         : [u8, ..MEMORY_SIZE],
    gfx         : [u8, ..SCREEN_WIDTH * SCREEN_HEIGHT],
    stack       : [u16, ..STACK_SIZE],
    key         : [u8, ..KEY_COUNT],
    gfx_update  : bool
}

impl Default for Chip8 {
    fn default () -> Chip8 {
        Chip8 {
            pc          : 0x200,
            i           : 0,
            delay_timer : 0,
            sound_timer : 0,
            sp          : 0,
            v           : [0, ..REGISTER_COUNT],
            mem         : [0, ..MEMORY_SIZE],
            gfx         : [0, ..SCREEN_WIDTH * SCREEN_HEIGHT],
            stack       : [0, ..STACK_SIZE],
            key         : [0, ..KEY_COUNT],
            gfx_update  : false
        }
    }
}

// public Chip8 methods
impl Chip8 {

    pub fn new() -> Chip8 {
        let mut rv = Chip8 { ..Default::default() };
        rv.mem.clone_from_slice(&FONTSET);
        return rv;
    }

    pub fn load_program(& mut self, filename: &String) -> bool {
        let path = Path::new(filename.as_slice());
        if !path.exists() {
            println!("program file {} does not exist", filename);
            return false;
        }

        let mut file = File::open(&path);
        return match file.read_to_end() {
            Ok(data) => {
                if data.len() > (0xFFF - 0x200) {
                    return false;
                }
                let dst = self.mem.slice_mut(0x200, 0x200 + data.len());
                bytes::copy_memory(dst, data.as_slice());
                true
            },
            _ => { false }
        };
    }

    pub fn run(& mut self, screen: &mut sdl::video::Surface ) {
        let mut timer = Timer::new().unwrap();
        let target_cycle_duration_ms = (1000.0f64 / 120.0f64) as u64;

        'mainloop : loop {
            let cycle_start = time::precise_time_ns();

            let opcode = self.fetch_opcode();
            self.decode_and_execute(opcode);

            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }
            if self.sound_timer > 0 {
                if self.sound_timer == 1 {
                    // TODO emit beep
                }
                self.sound_timer -= 1;
            }

            if self.gfx_update {
                self.draw_screen(screen);
                screen.flip();
                self.gfx_update = false;
            }

            'eventloop : loop {
                match sdl::event::poll_event() {
                    sdl::event::Event::Quit => break 'mainloop,
                    sdl::event::Event::None => break 'eventloop,
                    sdl::event::Event::Key(k, pressed, _, _) =>
                        match k {
                            sdl::event::Key::Escape => break 'mainloop,
                            _ => self.handle_keypress(k, pressed),
                        },
                    _ => {}
                }
            }

            let cycle_duration_ms = (time::precise_time_ns() - cycle_start) / 1000000;
            if cycle_duration_ms < target_cycle_duration_ms {
                timer.sleep(Duration::milliseconds((target_cycle_duration_ms - cycle_duration_ms) as i64));
            }
        }
    }
}

// Chip8 internals
impl Chip8 {

    fn map_key(key: sdl::event::Key) -> Option<u8> {
        /*
        Keypad                   Keyboard
        +-+-+-+-+                +-+-+-+-+
        |1|2|3|C|                |1|2|3|4|
        +-+-+-+-+                +-+-+-+-+
        |4|5|6|D|                |Q|W|E|R|
        +-+-+-+-+       =>       +-+-+-+-+
        |7|8|9|E|                |A|S|D|F|
        +-+-+-+-+                +-+-+-+-+
        |A|0|B|F|                |Z|X|C|V|
        +-+-+-+-+                +-+-+-+-+
        */

        return match key {
            sdl::event::Key::Num1 => Some(0x1),
            sdl::event::Key::Num2 => Some(0x2),
            sdl::event::Key::Num3 => Some(0x3),
            sdl::event::Key::Num4 => Some(0xC),
            sdl::event::Key::Q    => Some(0x4),
            sdl::event::Key::W    => Some(0x5),
            sdl::event::Key::E    => Some(0x6),
            sdl::event::Key::R    => Some(0xD),
            sdl::event::Key::A    => Some(0x7),
            sdl::event::Key::S    => Some(0x8),
            sdl::event::Key::D    => Some(0x9),
            sdl::event::Key::F    => Some(0xE),
            sdl::event::Key::Z    => Some(0xA),
            sdl::event::Key::X    => Some(0x0),
            sdl::event::Key::C    => Some(0xB),
            sdl::event::Key::V    => Some(0xF),
            _ => None
        }
    }

    fn handle_keypress(&mut self, key: sdl::event::Key, pressed: bool) {
        match Chip8::map_key(key) {
            Some(k) => self.key[k as uint] = if pressed { 1u8 } else { 0u8 },
            None => {}
        }
    }

    fn draw_screen(&mut self, screen: &mut sdl::video::Surface) {
        let pixelsize = 8 as u16;
        let white = sdl::video::RGB(0xFF, 0xFF, 0xFF);
        let black = sdl::video::RGB(0, 0, 0);

        for row in range(0u, SCREEN_HEIGHT) {
            for col in range(0u, SCREEN_WIDTH) {
                let color = if self.gfx[col + row * SCREEN_WIDTH] == 1 { white } else { black };
                screen.fill_rect(Some(sdl::Rect {
                    x: (col as i16) * (pixelsize as i16),
                    y: (row as i16) * (pixelsize as i16),
                    w: pixelsize,
                    h: pixelsize
                }), color);
            }
        }
    }

    fn fetch_opcode(&self) -> u16 {
        ( self.mem[ self.pc as uint ] as u16 << 8 ) |
        self.mem[ self.pc as uint + 1 ] as u16
    }

    fn advance_pc(& mut self, instruction_count: u16) {
        self.pc += instruction_count * 2;
    }

    fn stack_push(& mut self, val: u16) {
        assert!(self.sp as uint <= REGISTER_COUNT);
        self.stack[self.sp as uint] = val;
        self.sp += 1;
    }

    fn stack_pop(& mut self) -> u16 {
        assert!(self.sp > 0);
        self.sp -= 1;
        return self.stack[self.sp as uint];
    }

    // Instruction: Jump to location
    fn execute_jump(& mut self, dst: u16) {
        self.pc = dst;
    }

    // Instruction: Call subroutine
    fn execute_call(& mut self, dst: u16) {
        // TODO: a temporary should not be required here
        //  this appears to be related to https://github.com/rust-lang/rust/issues/6268
        let currpc = self.pc;
        self.stack_push(currpc);
        self.pc = dst;
    }

    // Instruction: Skip next instruction if Vx == val
    fn execute_skipifeq(& mut self, vx: uint, val: u8) {
        let numinstr = if self.v[vx] == val { 2 } else { 1 };
        self.advance_pc(numinstr);
    }

    // Instruction: Skip next instruction if Vx != val
    fn execute_skipifneq(& mut self, vx: uint, val: u8) {
        let numinstr = if self.v[vx] != val { 2 } else { 1 };
        self.advance_pc(numinstr);
    }

    // Instruction: Vx = val
    fn execute_setregister_const(& mut self, vx: uint, val: u8) {
        self.v[vx] = val;
        self.advance_pc(1);
    }

    // Instruction: Vx = Vx + val
    fn execute_addregister(& mut self, vx: uint, val: u8) {
        self.v[vx] += val;
        self.advance_pc(1);
    }

    // Instruction: Skip next instruction if Vx == Vy
    fn execute_skipifeq_register(& mut self, vx: uint, vy: uint) {
        let numinstr = if self.v[vx] == self.v[vy] { 2 } else { 1 };
        self.advance_pc(numinstr);
    }

    // Instruction: Skip next instruction if Vx != Vy
    fn execute_skipifneq_register(& mut self, vx: uint, vy: uint) {
        let numinstr = if self.v[vx] != self.v[vy] { 2 } else { 1 };
        self.advance_pc(numinstr);
    }

    // Instruction: Clear Display
    fn execute_clearscreen(& mut self) {
        self.gfx = [0, ..SCREEN_WIDTH * SCREEN_HEIGHT];
        self.gfx_update = true;
        self.advance_pc(1);
    }

    // Instruction: Return from current call
    fn execute_return(& mut self) {
        self.pc = self.stack_pop();
        self.advance_pc(1);
    }

    // Instruction: I = val
    fn execute_seti(& mut self, val: u16) {
        self.i = val;
        self.advance_pc(1);
    }
    
    // Instruction: Jump to v0 + val
    fn execute_jumpv0(& mut self, val: u16) {
        self.pc = self.v[0] as u16 + val;
    }

    // Instruction: Vx = rand byte & val
    fn execute_setrandand(& mut self, vx: uint, val: u8) {
        self.v[vx] = val & rand::random::<u8>();
        self.advance_pc(1);
    }

    // Instruction: Display n-row sprite starting at memory location I at (Vx, Vy), set VF = collision.
    fn execute_draw(& mut self, vx: uint, vy: uint, rows: u8) {
        let xcoord = self.v[vx] as uint;
        let ycoord = self.v[vy] as uint;

        self.v[0xF] = 0;
        for rowidx in range(0u, rows as uint) {
            let spriterow = self.mem[self.i as uint + rowidx];

            // our representation of pixels is as bytes, but the source
            // pixels are bitwise in memory
            for colidx in range(0u, 8) {
                if spriterow & (0x80 >> colidx) > 0 {
                    // check for collision and set VF if needed
                    if self.gfx[colidx + xcoord + (rowidx + ycoord) * SCREEN_WIDTH] == 1 {
                        self.v[0xF] = 1;
                    }

                    self.gfx[colidx + xcoord + (rowidx + ycoord) * SCREEN_WIDTH] ^= 1;
                }
            }
        }

        self.gfx_update = true;
        self.advance_pc(1);
    }

    // Instruction: Vx = Delay Timer
    fn execute_loaddtimer(& mut self, vx: uint) {
        self.v[vx] = self.delay_timer;
        self.advance_pc(1);
    }

    // Instruction: Wait for key press, store key in Vx
    fn execute_waitkey(& mut self, vx: uint) {
        'waitloop : loop {
            match sdl::event::poll_event() {
                sdl::event::Event::None => {},
                sdl::event::Event::Key(k, pressed, _, _) => {
                    self.handle_keypress(k, pressed);
                    if pressed {
                        match Chip8::map_key(k) {
                            Some(mkey) => {
                                self.v[vx] = mkey;
                            },
                            None => {}
                        }
                        break 'waitloop;
                    }
                },
                _ => ()
            }
        }

        self.advance_pc(1);
    }

    // Instruction: Delay Timer = Vx
    fn execute_setdtimer(& mut self, vx: uint) {
        self.delay_timer = self.v[vx];
        self.advance_pc(1);
    }

    // Instruction: Sound Timer = Vx
    fn execute_setstimer(& mut self, vx: uint) {
        self.sound_timer = self.v[vx];
        self.advance_pc(1);
    }

    // Instruction: I = I + Vx
    fn execute_addi(& mut self, vx: uint) {
        self.i += self.v[vx] as u16;
        self.advance_pc(1);
    }

    // Instruction: I = location of sprite for digit Vx
    fn execute_setifont(& mut self, vx: uint) {
        assert!(self.v[vx] <= 0xF);
        self.i = self.v[vx] as u16 * FONT_DIGIT_SIZE;
        self.advance_pc(1);
    }

    // Instruction: Store BCD representation of Vx in memory locations I, I+1, and I+2.
    fn execute_storebcd(& mut self, vx: uint) {
        let hundreds = self.v[vx] / 100;
        let tens = (self.v[vx] - hundreds * 100) / 10;
        let ones = self.v[vx] - hundreds * 100 - tens * 10;
        self.mem[self.i as uint] = hundreds;
        self.mem[self.i as uint + 1] = tens;
        self.mem[self.i as uint+ 2] = ones;
        self.advance_pc(1);
    }

    // Instruction: Read V0 through Vx from memory starting at location I
    fn execute_storeregs(& mut self, vx: uint) {
        for vi in range(0u, vx + 1) {
            self.mem[self.i as uint + vi] = self.v[vi];
        }
        self.advance_pc(1)
    }

    // Instruction: Store V0 through Vx in memory starting at location I
    fn execute_loadregs(& mut self, vx: uint) {
        for vi in range(0u, vx + 1) {
            self.v[vi] = self.mem[self.i as uint + vi];
        }
        self.advance_pc(1)
    }

    // Instruction: Skip next instruction if key in Vx is pressed
    fn execute_skipifkeypress(& mut self, vx: uint) {
        assert!(self.v[vx] < KEY_COUNT as u8);
        let instrcount = if self.key[self.v[vx] as uint] == 1 { 2 } else { 1 };
        self.advance_pc(instrcount);
    }

    // Instruction: Skip next instruction if key in Vx is not pressed
    fn execute_skipifnkeypress(& mut self, vx: uint) {
        assert!(self.v[vx] < KEY_COUNT as u8);
        let instrcount = if self.key[self.v[vx] as uint] == 0 { 2 } else { 1 };
        self.advance_pc(instrcount);
    }

    // Instruction: Vx = Vy
    fn execute_setregister_reg(& mut self, vx: uint, vy: uint) {
        self.v[vx] = self.v[vy];
        self.advance_pc(1);
    }

    // Instruction: Vx = Vx | Vy
    fn execute_bitor(& mut self, vx: uint, vy: uint) {
        self.v[vx] = self.v[vx] | self.v[vy];
        self.advance_pc(1);
    }

    // Instruction: Vx = Vx & Vy
    fn execute_bitand(& mut self, vx: uint, vy: uint) {
        self.v[vx] = self.v[vx] & self.v[vy];
        self.advance_pc(1);
    }

    // Instruction: Vx = Vx ^ Vy
    fn execute_bitxor(& mut self, vx: uint, vy: uint) {
        self.v[vx] = self.v[vx] ^ self.v[vy];
        self.advance_pc(1);
    }

    // Instruction: Vx = Vx + Vy, VF = carry
    fn execute_add(& mut self, vx: uint, vy: uint) {
        let result = self.v[vx] as u16 + self.v[vy] as u16;
        self.v[vx] = result as u8;
        self.v[0xF] = if result > std::u8::MAX as u16 { 1 } else { 0 };
        self.advance_pc(1);
    }

    // Instruction: Vx = Vx - Vy, VF = ~borrow
    fn execute_sub(& mut self, vx: uint, vy: uint) {
        self.v[0xF] = if self.v[vy] > self.v[vx] { 0 } else { 1 };
        self.v[vx] = self.v[vx] - self.v[vy];
        self.advance_pc(1);
    }

    // Instruction: Vx = Vx >> 1, VF = LSB of Vx before shifting
    fn execute_shr(& mut self, vx: uint) {
        self.v[0xF] = self.v[vx] & 1;
        self.v[vx] = self.v[vx] >> 1;
        self.advance_pc(1);
    }

    // Instruction: Vx = Vx << 1, VF = MSB of Vx before shifting
    fn execute_shl(& mut self, vx: uint) {
        self.v[0xF] = self.v[vx] >> 7;
        self.v[vx] = self.v[vx] << 1;
        self.advance_pc(1);
    }

    // Instruction: Vx = Vy - Vx, VF = 0 if there is a borrow, 1 otherwise
    fn execute_sub_inverse(& mut self, vx: uint, vy: uint) {
        self.v[0xF] = if self.v[vx] > self.v[vy] { 0 } else { 1 };
        self.v[vx] = self.v[vy] - self.v[vx];
        self.advance_pc(1);
    }

    fn decode_and_execute(& mut self, opcode: u16) {
        let short = opcode & 0x0FFF;
        let vx = ((opcode & 0x0F00) >> 8) as uint;
        let vy = ((opcode & 0x00F0) >> 4) as uint;
        let byte = (opcode & 0xFF) as u8;
        let nibble = (opcode & 0xF) as u8;

        match opcode {
            0x00E0 => self.execute_clearscreen(),
            0x00EE => self.execute_return(),
            _ => match (opcode & 0xF000) >> 12 {
                0x1 => self.execute_jump(short),
                0x2 => self.execute_call(short),
                0x3 => self.execute_skipifeq(vx, byte),
                0x4 => self.execute_skipifneq(vx, byte),
                0x5 => self.execute_skipifeq_register(vx, vy),
                0x6 => self.execute_setregister_const(vx, byte),
                0x7 => self.execute_addregister(vx, byte),
                0x8 => match opcode & 0xF {
                    0x0 => self.execute_setregister_reg(vx, vy),
                    0x1 => self.execute_bitor(vx, vy),
                    0x2 => self.execute_bitand(vx, vy),
                    0x3 => self.execute_bitxor(vx, vy),
                    0x4 => self.execute_add(vx, vy),
                    0x5 => self.execute_sub(vx, vy),
                    0x6 => self.execute_shr(vx),
                    0x7 => self.execute_sub_inverse(vx, vy),
                    0xE => self.execute_shl(vx),
                      _ => println!("invalid instruction")
                },
                0x9 => self.execute_skipifneq_register(vx, vy), 
                0xA => self.execute_seti(short),
                0xB => self.execute_jumpv0(short),
                0xC => self.execute_setrandand(vx, byte),
                0xD => self.execute_draw(vx, vy, nibble),
                0xE => match opcode & 0xFF {
                    0x9E => self.execute_skipifkeypress(vx),
                    0xA1 => self.execute_skipifnkeypress(vx),
                       _ => println!("invalid instruction")
                },
                0xF => match opcode & 0xFF {
                    0x07 => self.execute_loaddtimer(vx),
                    0x0A => self.execute_waitkey(vx),
                    0x15 => self.execute_setdtimer(vx),
                    0x18 => self.execute_setstimer(vx),
                    0x1E => self.execute_addi(vx),
                    0x29 => self.execute_setifont(vx),
                    0x33 => self.execute_storebcd(vx),
                    0x55 => self.execute_storeregs(vx),
                    0x65 => self.execute_loadregs(vx),
                       _ => println!("invalid instruction")
                },
                _ => println!("invalid instruction1") 
            }
        }

        // println!("{:X}, PC: {:X}, I: {:X}", opcode, self.pc, self.i);
    }
}
