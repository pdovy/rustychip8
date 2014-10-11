extern crate std;

use std::default::Default;
use super::Chip8;

#[test]
fn test_stack() {
    let mut emu = Chip8 { ..Default::default() };
    emu.stack_push(1);
    emu.stack_push(2);
    emu.stack_push(3);
    emu.stack_push(4);
    assert_eq!(emu.stack_pop(), 4);
    assert_eq!(emu.stack_pop(), 3);
    assert_eq!(emu.stack_pop(), 2);
    assert_eq!(emu.stack_pop(), 1);
}

#[test]
fn test_instr_jump() {
    let mut emu = Chip8 { ..Default::default() };
    let dst = 0xABC as u16;

    emu.execute_jump(dst);
    assert_eq!(emu.pc, dst);
}

#[test]
fn test_instr_call() {
    let mut emu = Chip8 { ..Default::default() };
    let dst = 0xABC as u16;
    let prev_sp = emu.sp;

    emu.execute_call(dst);
    assert_eq!(emu.pc, dst);
    assert_eq!(emu.sp, prev_sp + 1);
}

#[test]
fn test_instr_setregister() {
    let mut emu = Chip8 { ..Default::default() };
    for regidx in range(0u, 16u) {
        let val = (regidx + 1) as u8;
        emu.execute_setregister_const(regidx, val);
        assert_eq!(emu.v[regidx], val);
    }
}
