extern crate getopts;

use std::default::Default;
use std::os;
use getopts::{optopt,optflag,getopts};

mod chip8impl;

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

    let mut emustate = chip8impl::Chip8 { ..Default::default() };
    emustate.initialize();
    if ! emustate.load_program( &romfile ) {
        println!("failed to load ROM file");
    }

    emustate.emulate_cycle();
    
}
