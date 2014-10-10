extern crate std;
extern crate getopts;
use std::default::Default;
use std::os;
use std::slice::bytes;
use std::io::File;
use std::rand;
use getopts::{optopt,optflag,getopts,OptGroup};

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

struct Chip8 {
    pc          : u16,
    i           : u16,
    delay_timer : u8,
    sound_timer : u8,
    sp          : u8,
    v           : [u8, ..16],
    mem         : [u8, ..4096],
    gfx         : [[u8, ..64], ..32],
    stack       : [u16, ..16],
    key         : [u8, ..16]
}

impl Default for Chip8 {
    fn default () -> Chip8 {
        Chip8 {
            pc          : 0x200,
            i           : 0,
            delay_timer : 0,
            sound_timer : 0,
            sp          : 0,
            v           : [0, ..16],
            mem         : [0, ..4096],
            gfx         : [[0, ..64], ..32],
            stack       : [0, ..16],
            key         : [0, ..16]
        }
    }
}

impl Chip8 {
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

    fn execute_jump(& mut self, dst : u16) {
        self.pc = dst;
    }

    fn execute_call(& mut self, dst : u16) {
        // push current pc onto stack before moving to call location
        self.stack[self.sp as uint] = self.pc;
        self.sp += 1;
        self.pc = dst;
    }

    fn execute_skipifeq(& mut self, reg : u8, val : u8) {
        if self.v[reg as uint] == val {
            self.pc += 4;
        }
        else {
            self.pc += 2;
        }
    }

    fn execute_skipifneq(& mut self, reg : u8, val : u8) {
        if self.v[reg as uint] != val {
            self.pc += 4;
        }
        else {
            self.pc += 2;
        }
    }

    fn execute_setregister(& mut self, reg : u8, val : u8) {
        self.v[reg as uint] = val;
        self.pc += 2;
    }

    fn execute_addregister(& mut self, reg : u8, val : u8) {
        self.v[reg as uint] += val;
        self.pc += 2;
    }

    fn execute_doubleargop(& mut self, opcode : u16) {
        self.pc += 2;
    }

    fn execute_skipifeq_register(& mut self, rega : u8, regb : u8) {
        if self.v[rega as uint] == self.v[regb as uint] {
            self.pc += 4;
        }
        else {
            self.pc += 2;
        }
    }

    fn execute_clrscreen(& mut self) {
        // TODO
    }

    fn execute_return(& mut self) {
        assert!(self.sp > 0);
        self.pc = self.stack[(self.sp - 1) as uint];
        self.sp -= 1;
    }

    fn execute_seti(& mut self, val : u16) {
        self.i = val;
        self.pc += 2;
    }

    fn execute_jumpv0(& mut self, val : u16) {
        self.pc = self.v[0] as u16 + val;
    }

    fn execute_setrandand(& mut self, reg : u8, val : u8) {
        self.v[reg as uint] = val & rand::random::<u8>();
        self.pc += 2;
    }

    fn execute_draw(& mut self, opcode : u16) {
        // TODO
        self.pc += 2;
    }

    fn execute_skipkey(& mut self, opcode : u16) {
        // TODO
        self.pc += 2;
    }

    fn execute_loaddtimer(& mut self, reg: u8) {
        self.v[reg as uint] = self.delay_timer;
        self.pc += 2;
    }

    fn execute_waitkey(& mut self, reg: u8) {
        // TODO
        self.pc += 2;
    }

    fn execute_setdtimer(& mut self, reg: u8) {
        self.delay_timer = self.v[reg as uint];
        self.pc += 2;
    }

    fn execute_setstimer(& mut self, reg: u8) {
        self.sound_timer = self.v[reg as uint];
        self.pc += 2;
    }

    fn execute_addi(& mut self, reg: u8) {
        self.i += self.v[reg as uint] as u16;
        self.pc += 2;
    }

    fn execute_setifont(& mut self, reg: u8) {
        // TODO
        self.pc += 2;
    }

    fn execute_storebin(& mut self, reg: u8) {
        // TODO
        self.pc += 2;
    }

    fn execute_storeregs(& mut self, reg: u8) {
        for vi in range(0u, reg as uint) {
            self.mem[self.i as uint + vi] = self.v[vi];
        }
    }

    fn execute_loadregs(& mut self, reg: u8) {
        for vi in range(0u, reg as uint) {
            self.v[vi] = self.mem[self.i as uint + vi];
        }
    }

    fn decode_and_execute(& mut self, opcode: u16) {
        let longval = opcode & 0x0FFF;
        let rega = ((opcode & 0x0F00) >> 8) as u8;
        let regb = ((opcode & 0x00F0) >> 4) as u8;
        let val = (opcode & 0xFF) as u8;

        match opcode {
            0x00E0 => self.execute_clrscreen(),
            0x00EE => self.execute_return(),
            _ => match (opcode & 0xF000) >> 12 {
                0x1 => self.execute_jump(longval),
                0x2 => self.execute_call(longval),
                0x3 => self.execute_skipifeq(rega, val),
                0x4 => self.execute_skipifneq(rega, val),
                0x5 => self.execute_setregister(rega, val),
                0x6 => self.execute_addregister(rega, val),
                0x8 => self.execute_doubleargop(opcode),
                0x9 => self.execute_skipifeq_register(rega, regb),
                0xA => self.execute_seti(longval),
                0xB => self.execute_jumpv0(longval),
                0xC => self.execute_setrandand(rega, val),
                0xD => self.execute_draw(opcode),
                0xE => self.execute_skipkey(opcode),
                0xF => match opcode & 0xFF {
                    0x07 => self.execute_loaddtimer(rega),
                    0x0A => self.execute_waitkey(rega),
                    0x15 => self.execute_setdtimer(rega),
                    0x18 => self.execute_setstimer(rega),
                    0x1E => self.execute_addi(rega),
                    0x29 => self.execute_setifont(rega),
                    0x33 => self.execute_storebin(rega),
                    0x55 => self.execute_storeregs(rega),
                    0x64 => self.execute_loadregs(rega),
                    _ => println!("not yet handled")
                },
                _ => println!("not yet handled 1") 
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

fn main() {
    let args: Vec<String> = os::args();

    let program = args[0].clone();
    let opts = [
        optopt("f", "ROM filename", "ROM file to load", "FILENAME"),
        optflag("h", "help", "print this help")
    ];
    let matches = match getopts(args.tail(), opts) {
        Ok(m) => { m },
        Err(f) => { fail!(f.to_string()) }
    };
    let romfile = match matches.opt_str("f") {
        Some(f) => { f },
        None    => { String::new() }
    };
    if matches.opt_present("h") || romfile.len() == 0 {
        println!("{}", getopts::short_usage(program.as_slice(), opts));
        return;
    }

    let mut emustate = Chip8 { ..Default::default() };
    emustate.mem.copy_from( fontset );
    if ! emustate.load_program( &romfile ) {
        println!("failed to load ROM file");
    }

    emustate.emulate_cycle();
    
}
