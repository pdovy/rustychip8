extern crate std;
extern crate getopts;
use std::default::Default;
use std::os;
use std::slice::bytes;
use std::io::File;
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
    opcode      : u8,
    i           : u8,
    delay_timer : u8,
    sound_timer : u8,
    sp          : u8,
    v           : [u8, ..16],
    mem         : [u8, ..4096],
    gfx         : [[u8, ..64], ..32],
    stack       : [u8, ..16],
    key         : [u8, ..16]
}

impl Default for Chip8 {
    fn default () -> Chip8 {
        Chip8 {
            pc          : 0x200,
            opcode      : 0,
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

    pub fn emulate_cycle(& mut self) {
        let opcode = self.fetch_opcode();

        self.pc += 2;

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
