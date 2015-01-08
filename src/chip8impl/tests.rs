extern crate std;

use std::default::Default;
use super::Chip8;

#[test]
fn test_stack() {
    let mut emu = Chip8::new();
    for index in range(0u16, 16) {
        emu.stack_push(index);
    }
    for index in range(16u16, 0) {
        assert_eq!(emu.stack_pop(), index);
    }
}

#[test]
fn test_instr_jump() {
    let mut emu = Chip8::new();
    let dst = 0xABC as u16;

    emu.execute_jump(dst);
    assert_eq!(emu.pc, dst);
}

#[test]
fn test_instr_call() {
    let mut emu = Chip8::new();
    let dst = 0xABC as u16;
    let prev_sp = emu.sp;

    emu.execute_call(dst);
    assert_eq!(emu.pc, dst);
    assert_eq!(emu.sp, prev_sp + 1);
}

#[test]
fn test_instr_setregister_const() {
    let mut emu = Chip8::new();
    for regidx in range(0u, 16u) {
        let val = (regidx + 1) as u8;
        emu.execute_setregister_const(regidx, val);
        assert_eq!(emu.v[regidx], val);
    }
}

#[test]
fn test_instr_skipifeq() {
    let mut emu = Chip8::new();
    let mut pcstart = emu.pc;

    emu.v[0] = 1u8;
    emu.execute_skipifeq(0, 1u8);
    assert_eq!(pcstart + 4, emu.pc);

    pcstart = emu.pc;
    emu.execute_skipifeq(0, 2u8);
    assert_eq!(pcstart + 2, emu.pc);
}

#[test]
fn test_instr_skipifneq() {
    let mut emu = Chip8::new();
    let mut pcstart = emu.pc;

    emu.v[0] = 1u8;
    emu.execute_skipifneq(0, 1u8);
    assert_eq!(pcstart + 2, emu.pc);

    pcstart = emu.pc;
    emu.execute_skipifneq(0, 2u8);
    assert_eq!(pcstart + 4, emu.pc);
}

#[test]
fn test_instr_skipifeq_register() {
    let mut emu = Chip8::new();
    let mut pcstart = emu.pc;

    emu.v[0] = 1u8;
    emu.v[1] = emu.v[0];
    emu.execute_skipifeq_register(0, 1);
    assert_eq!(pcstart + 4, emu.pc);

    pcstart = emu.pc;
    emu.v[1] = 0;
    emu.execute_skipifeq_register(0, 1);
    assert_eq!(pcstart + 2, emu.pc);
}

#[test]
fn test_instr_skipifneq_register() {
    let mut emu = Chip8::new();
    let mut pcstart = emu.pc;

    emu.v[0] = 1u8;
    emu.v[1] = emu.v[0];
    emu.execute_skipifneq_register(0, 1);
    assert_eq!(pcstart + 2, emu.pc);

    pcstart = emu.pc;
    emu.v[1] = 0;
    emu.execute_skipifneq_register(0, 1);
    assert_eq!(pcstart + 4, emu.pc);
}

#[test]
fn test_instr_addregister() {
    let mut emu = Chip8::new();
    emu.v[0] = 1u8;
    emu.execute_addregister(0, 10u8);
    assert_eq!(emu.v[0], 11u8);
}

#[test]
fn test_instr_clearscreen() {
    let mut emu = Chip8::new();
    emu.gfx = [1u8; 64 * 32];
    emu.execute_clearscreen();
    for idx in range(0u, 64 * 32) {
        assert_eq!(emu.gfx[idx], 0);
    }
}

#[test]
fn test_instr_return() {
    let mut emu = Chip8::new();

    let pcstart = emu.pc;
    emu.execute_call(0xABC);
    emu.execute_call(0xDEF);

    emu.execute_return();
    assert_eq!(emu.pc, 0xABC + 2);

    emu.execute_return();
    assert_eq!(emu.pc, pcstart + 2);
}

#[test]
fn test_seti() {
    let mut emu = Chip8::new();
    emu.execute_seti(0xABC);
    assert_eq!(emu.i, 0xABC);
}

#[test]
fn test_instr_jumpv0() {
    let mut emu = Chip8::new();
    emu.v[0] = 0xF;
    emu.execute_jumpv0(0x200);
    assert_eq!(emu.pc, 0x20F);
}

#[test]
fn test_instr_setrandand() {
    let mut emu = Chip8::new();
    emu.v[0] = 0xF;
    emu.execute_setrandand(0, 0);
    assert_eq!(emu.v[0], 0);

    for idx in range(0u, 10000u) {
        emu.execute_setrandand(0, 0b11);
        assert_eq!(emu.v[0] & !(0b11 as u8), 0);
    }
}

#[test]
fn test_instr_draw() {
    // TODO
}

#[test]
fn test_instr_loaddtimer() {
    let mut emu = Chip8::new();
    emu.delay_timer = 8u8;
    emu.execute_loaddtimer(1);
    assert_eq!(emu.v[1], emu.delay_timer);
}

#[test]
fn test_instr_setdtimer() {
    let mut emu = Chip8::new();
    emu.v[2] = 3;
    emu.execute_setdtimer(2);
    assert_eq!(emu.v[2], emu.delay_timer);
}

#[test]
fn test_instr_setstimer() {
    let mut emu = Chip8::new();
    emu.v[2] = 3;
    emu.execute_setstimer(2);
    assert_eq!(emu.v[2], emu.sound_timer);
}

#[test]
fn test_instr_addi() {
    let mut emu = Chip8::new();
    let istart = emu.i;

    emu.v[0] = 0xA;
    emu.execute_addi(0);
    assert_eq!(emu.i, istart + emu.v[0] as u16);
}

#[test]
fn test_instr_setifont() {
    let mut emu = Chip8::new();
    emu.v[0] = 0xA;
    emu.execute_setifont(0);
    assert_eq!(emu.i, 0xAu16 * 5);
}

#[test]
fn test_instr_storebcd() {
    let mut emu = Chip8::new();
    emu.i = 0x100;

    emu.v[0] = 5;
    emu.execute_storebcd(0);
    assert_eq!(emu.mem[emu.i as uint], 0);
    assert_eq!(emu.mem[emu.i as uint + 1], 0);
    assert_eq!(emu.mem[emu.i as uint + 2], 5);

    emu.v[0] = 26;
    emu.execute_storebcd(0);
    assert_eq!(emu.mem[emu.i as uint], 0);
    assert_eq!(emu.mem[emu.i as uint + 1], 2);
    assert_eq!(emu.mem[emu.i as uint + 2], 6);

    emu.v[0] = 137;
    emu.execute_storebcd(0);
    assert_eq!(emu.mem[emu.i as uint], 1);
    assert_eq!(emu.mem[emu.i as uint + 1], 3);
    assert_eq!(emu.mem[emu.i as uint + 2], 7);
}

#[test]
fn test_instr_storeregs() {
    let mut emu = Chip8::new();

    for idx in range(0u, 16u) {
        emu.v[idx] = idx as u8;
    }

    emu.i = 0x100;
    emu.execute_storeregs(5);

    for idx in range(0u, 6u) {
        assert_eq!(emu.mem[0x100 + idx], idx as u8);
    }
    for idx in range(6u, 16u) {
        assert_eq!(emu.mem[0x100 + idx], 0);
    }
}

#[test]
fn test_instr_loadregs() {
    let mut emu = Chip8::new();

    for idx in range(0u, 16u) {
        emu.mem[0x100 + idx] = idx as u8;
    }

    emu.i = 0x100;
    emu.execute_loadregs(5);

    for idx in range(0u, 6u) {
        assert_eq!(emu.v[idx], idx as u8);
    }
    for idx in range(6u, 16u) {
        assert_eq!(emu.v[idx], 0);
    }    
}

#[test]
fn test_instr_setregister_reg() {
    let mut emu = Chip8::new();
    emu.v[1] = 2;
    emu.execute_setregister_reg(0, 1);
    assert_eq!(emu.v[0], 2);
}

#[test]
fn test_instr_bitor() {
    let mut emu = Chip8::new();
    emu.v[0] = 0b1010;
    emu.v[1] = 0b0110;
    emu.execute_bitor(0, 1);
    assert_eq!(emu.v[0], 0b1110);
}    

#[test]
fn test_instr_bitand() {
    let mut emu = Chip8::new();
    emu.v[0] = 0b1010;
    emu.v[1] = 0b0110;
    emu.execute_bitand(0, 1);
    assert_eq!(emu.v[0], 0b0010);
}    

#[test]
fn test_instr_bitxor() {
    let mut emu = Chip8::new();
    emu.v[0] = 0b1010;
    emu.v[1] = 0b0110;
    emu.execute_bitxor(0, 1);
    assert_eq!(emu.v[0], 0b1100);
}

#[test]
fn test_instr_add() {
    let mut emu = Chip8::new();

    // add with carry
    emu.v[0] = 0xFF;
    emu.v[1] = 0xFF;
    emu.execute_add(0, 1);
    assert_eq!(emu.v[0], (0xFFu16 + 0xFF) as u8);
    assert_eq!(emu.v[0xF], 1);

    // add w/o carry
    emu.v[0] = 0xF;
    emu.v[1] = 0xA;
    emu.execute_add(0, 1);
    assert_eq!(emu.v[0], (0xFu16 + 0xA) as u8);
    assert_eq!(emu.v[0xF], 0);
}

#[test]
fn test_instr_sub() {
    let mut emu = Chip8::new();

    // subtract with borrow
    emu.v[0] = 0x1;
    emu.v[1] = 0xF;
    emu.execute_sub(0, 1);
    assert_eq!(emu.v[0], 0x1u8 - 0xF);
    assert_eq!(emu.v[0xF], 0);

    // subtract w/o borrow
    emu.v[0] = 0xF;
    emu.v[1] = 0x1;
    emu.execute_sub(0, 1);
    assert_eq!(emu.v[0], 0xFu8 - 0x1);
    assert_eq!(emu.v[0xF], 1);
}

#[test]
fn test_instr_shl() {
    let mut emu = Chip8::new();

    emu.v[0] = 0b10000000;
    emu.execute_shl(0);
    assert_eq!(emu.v[0], 0);
    assert_eq!(emu.v[0xF], 1);
}

#[test]
fn test_instr_shr() {
    let mut emu = Chip8::new();

    emu.v[0] = 0b1;
    emu.execute_shr(0);
    assert_eq!(emu.v[0], 0);
    assert_eq!(emu.v[0xF], 1);
}

#[test]
fn test_instr_sub_inverse() {
    let mut emu = Chip8::new();

    // subtract with borrow
    emu.v[0] = 0xF;
    emu.v[1] = 0x1;
    emu.execute_sub_inverse(0, 1);
    assert_eq!(emu.v[0], 0x1u8 - 0xF);
    assert_eq!(emu.v[0xF], 0);

    // subtract w/o borrow
    emu.v[0] = 0x1;
    emu.v[1] = 0xF;
    emu.execute_sub_inverse(0, 1);
    assert_eq!(emu.v[0], 0xFu8 - 0x1);
    assert_eq!(emu.v[0xF], 1);
}

#[test]
fn test_instr_skipifkeypress() {
    let mut emu = Chip8::new();

    // key is pressed
    let startpc = emu.pc;
    emu.v[0] = 0xA;
    emu.key[0xA] = 1;
    emu.execute_skipifkeypress(0);
    assert_eq!(emu.pc, startpc + 4);
}

#[test]
fn test_instr_skipifnkeypress() {
    let mut emu = Chip8::new();
}
