Rust Chip-8 Emulator
==========

A simple Chip-8 emulator written in Rust.  I was interested in learning Rust and thought a simple emulator would be a great place to start digging into the language.

![Space Invaders](screenshots/invaders.png)

## What is Chip-8?

Chip-8 was never a real in-hardware system, instead it's a simple programming language that can be run on any system with a properly implemented virtual machine.  It was first used in the 70s and 80s on early computers like the [COSMAC VIP](http://en.wikipedia.org/wiki/COSMAC_VIP), [DREAM 6800](http://www.mjbauer.biz/DREAM6800.htm) and other early DIY machines.  Later in the 90s it regained some popularity as an easy way to program games on graphing calculators like the HP48 (and who didn't try and program games on their graphing calculator?  Thankfully it was easier for my generation poking at the TI89).

## Building and Running

This is super easy to build using [Cargo](cargo.io), just install Cargo as per the instructions on the website, clone the repo and run:

    cargo build --release
    
Note that Rust is still in a pre-1.0 state and breaking changes may be made that will cause the code in this repo not to compile.  You can safely assume that it built against the nightly Rust build on the date of the last commit.  If you come across anything broken, I'll happily accept PRs.

To run, you'll need to first source some Chip-8 ROMs - there are quite a few available in the "program pack" hosted by [Chip8.com](http://chip8.com/).  Then to run, just specify the path to your ROM:

    target/rustychip8 -f ~/chip8roms/PONG
    
## Task List

* [x] Implement main fetch/decode/execute loop with support for all instructions.
* [x] Add rendering of the graphics buffer
* [x] Add keyboard input
* [ ] Testing with various public domain ROMs


