extern crate getopts;
extern crate sdl;

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

    // initialize SDL for graphical output and keyboard input
    sdl::init([sdl::InitVideo]);
    sdl::wm::set_caption("RustyChip8", "");

    let mut screen = match sdl::video::set_video_mode(
        640, 320, 32, [sdl::video::HWSurface], [sdl::video::DoubleBuf])
    {
        Ok(screen) => screen,
        Err(err) => fail!("failed to set video mode: {}", err)
    };

    // fire up the emulator
    let mut emu = chip8impl::Chip8::new();
    if ! emu.load_program( &romfile ) {
        println!("failed to load ROM file");
    }

    emu.run(&mut screen);

    sdl::quit();
}
