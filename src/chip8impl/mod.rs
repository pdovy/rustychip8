extern crate std;

use std::default::Default;
use std::slice::bytes;
use std::io::File;
use std::rand;

#[cfg(test)]
mod tests;

static fontset : [u8, ..80] =
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

static STACK_SIZE:     uint = 16;
static MEMORY_SIZE:    uint = 4096;
static REGISTER_COUNT: uint = 16;
static SCREEN_WIDTH:   uint = 64;
static SCREEN_HEIGHT:  uint = 32;
static KEY_COUNT:      uint = 16;

pub struct Chip8 {
    pc          : u16,
    i           : u16,
    delay_timer : u8,
    sound_timer : u8,
    sp          : u8,
    v           : [u8, ..REGISTER_COUNT],
    mem         : [u8, ..MEMORY_SIZE],
    gfx         : [[u8, ..SCREEN_WIDTH], ..SCREEN_HEIGHT],
    stack       : [u16, ..STACK_SIZE],
    key         : [u8, ..KEY_COUNT]
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
            gfx         : [[0, ..SCREEN_WIDTH], ..SCREEN_HEIGHT],
            stack       : [0, ..STACK_SIZE],
            key         : [0, ..KEY_COUNT]
        }
    }
}

impl Chip8 {

    pub fn initialize(& mut self) {
        // load the font data into memory
        self.mem.copy_from( fontset );
    }

    pub fn load_program(& mut self, filename: &String) -> bool {
        let path = Path::new(filename.as_slice());
        if !path.exists() {
            println!("program file {} does not exist", filename)
            return false
        }

        let mut file = File::open(&path);
        return match file.read_to_end() {
            Ok(data) => {
                if data.len() > (0xFFF - 0x200) {
                    return false;
                }
                let dst = self.mem.mut_slice(0x200, 0x200 + data.len());
                bytes::copy_memory(dst, data.as_slice());
                true
            },
            Err(e) => { false }
        };
    }

    fn fetch_opcode(&self) -> u16 {
        ( self.mem[ self.pc as uint ] as u16 << 8 ) |
        self.mem[ self.pc as uint + 1 ] as u16
    }

    fn advance_pc(& mut self, instruction_count: u16) {
        self.pc += (instruction_count * 2);
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
        // push current pc onto stack before moving to call location
        self.stack[self.sp as uint] = self.pc;
        self.sp += 1;
        self.pc = dst;
    }

    // Instruction: Skip next instruction if Vx == val
    fn execute_skipifeq(& mut self, vx: uint, val: u8) {
        if self.v[vx] == val {
            self.advance_pc(2);
        }
        else {
            self.advance_pc(1);
        }
    }

    // Instruction: Skip next instruction if Vx != val
    fn execute_skipifneq(& mut self, vx: uint, val: u8) {
        if self.v[vx] != val {
            self.advance_pc(2)
        }
        else {
            self.advance_pc(1);
        }
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
        if self.v[vx] == self.v[vy] {
            self.advance_pc(2);
        }
        else {
            self.advance_pc(1);
        }
    }

    // Instruction: Clear Display
    fn execute_clearscreen(& mut self) {
        // TODO
    }

    // Instruction: Return from current call
    fn execute_return(& mut self) {
        assert!(self.sp > 0);
        self.pc = self.stack[(self.sp - 1) as uint];
        self.sp -= 1;
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

    // Instruction: Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    fn execute_draw(& mut self, vx: uint, vy: uint, nibble: u8) {
        // TODO
        self.advance_pc(1);
    }

    // Instruction: Vx = Delay Timer
    fn execute_loaddtimer(& mut self, vx: uint) {
        self.v[vx] = self.delay_timer;
        self.advance_pc(1);
    }

    // Instruction: Wait for key press, store key in Vx
    fn execute_waitkey(& mut self, vx: uint) {
        // TODO
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
        // TODO
        self.advance_pc(1);
    }

    // Instruction: Store BCD representation of Vx in memory locations I, I+1, and I+2.
    fn execute_storebin(& mut self, reg: uint) {
        // TODO
        self.advance_pc(1);
    }

    // Instruction: Read V0 through Vx from memory starting at location I
    fn execute_storeregs(& mut self, vx: uint) {
        for vi in range(0u, vx) {
            self.mem[self.i as uint + vi] = self.v[vi];
        }
        self.advance_pc(1)
    }

    // Instruction: Store V0 through Vx in memory starting at location I
    fn execute_loadregs(& mut self, vx: uint) {
        for vi in range(0u, vx) {
            self.v[vi] = self.mem[self.i as uint + vi];
        }
        self.advance_pc(1)
    }

    // Instruction: Skip next instruction if key in Vx is pressed
    fn execute_skipifkeypress(& mut self, vx: uint) {
        // TODO
    }

    // Instruction: Skip next instruction if key in Vx is not pressed
    fn execute_skipifnkeypress(& mut self, vx: uint) {
        // TODO
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
        // TODO
        self.advance_pc(1);
    }

    // Instruction: Vx = Vx - Vy, VF = ~borrow
    fn execute_sub(& mut self, vx: uint, vy: uint) {
        // TODO
        self.advance_pc(1);
    }

    // Instruction: Vx = Vx >> 1, VF = LSB of Vx before shifting
    fn execute_shr(& mut self, vx: uint, vy: uint) {
        self.v[0xF] = (vx as u8) & 1;
        self.v[vx] = self.v[vx] >> 1;
        self.advance_pc(1);
    }

    // Instruction: Vx = Vx << 1, VF = MSB of Vx before shifting
    fn execute_shl(& mut self, vx: uint, vy: uint) {
        self.v[0xF] = (vx as u8) >> 7;
        self.v[vx] = self.v[vx] << 1;
        self.advance_pc(1);
    }

    // Instruction: Vx = Vy - Vx, VF = 0 if there is a borrow, 1 otherwise
    fn execute_sub_inverse(& mut self, vx: uint, vy: uint) {
        // TODO
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
                0x8 => match opcode & 0xF {
                    0x0 => self.execute_setregister_reg(vx, vy),
                    0x1 => self.execute_bitor(vx, vy),
                    0x2 => self.execute_bitand(vx, vy),
                    0x3 => self.execute_bitxor(vx, vy),
                    0x4 => self.execute_add(vx, vy),
                    0x5 => self.execute_sub(vx, vy),
                    0x6 => self.execute_shr(vx, vy),
                    0x7 => self.execute_sub_inverse(vx, vy),
                    0xE => self.execute_shl(vx, vy),
                      _ => println!("invalid instruction")
                },
                0x9 => self.execute_addregister(vx, byte),
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
                    0x33 => self.execute_storebin(vx),
                    0x55 => self.execute_storeregs(vx),
                    0x64 => self.execute_loadregs(vx),
                       _ => println!("invalid instruction")
                },
                _ => println!("invalid instruction1") 
            }
        }
    }

    pub fn emulate_cycle(& mut self) {
        let opcode = self.fetch_opcode();
        self.decode_and_execute(opcode);

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }
}
