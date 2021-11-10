#[derive(Debug, PartialEq)]
pub(crate) enum Instruction {
    /// 0x00E0: Clear screen
    ClearScreen,
    /// 0x00EE: Return from subroutine
    SubroutineReturn,
    /// 0x1NNN: Jump to NNN
    Jump { nnn: u16 },
    /// 0x2NNN: Subroutine call at NNN
    SubroutineCall { nnn: u16 },
    /// 0x3XNN: Skip if VX == NN
    SkipVxEqNn { x: usize, nn: u8 },
    /// 0x4XNN: Skip if VX != NN
    SkipVxNeqNn { x: usize, nn: u8 },
    /// 0x5XY0: Skip if VX == VY
    SkipVxEqVy { x: usize, y: usize },
    /// 0x6NNN: Set register VX to NN
    SetVxNn { x: usize, nn: u8 },
    /// 0x7XNN: Add NN to VX, ignoring carry
    AddNnVx { x: usize, nn: u8 },
    /// 0x8XY0: Set VX to VY
    SetVxVy { x: usize, y: usize },
    /// 0x8XY1: Set VX to VX | VY
    SetVxOrVy { x: usize, y: usize },
    /// 0x8XY2: Set VX to VX & VY
    SetVxAndVy { x: usize, y: usize },
    /// 0x8XY3: Set VX to VX XOR VY
    SetVxXorVy { x: usize, y: usize },
    /// 0x8XY4: Set VX to VX + VY, accounting for carry
    SetVxPlusVy { x: usize, y: usize },
    /// 0x8XY5: Set VX to VX - VY, accounting for carry
    SetVxMinusVy { x: usize, y: usize },
    /// 0x8XY6: Store least significant bit of VX in VF and shift VX right by 1
    ShiftVxRight { x: usize },
    /// 0x8XY7: Set VX to VY - VX, accounting for carry
    SetVyMinusVx { x: usize, y: usize },
    /// 0x8XYE: Store most significant bit of VX in VF and shift VX left by 1
    ShiftVxLeft { x: usize },
    /// 0x9XY0: Skip if VX != VY
    SkipVxNeqVy { x: usize, y: usize },
    /// 0xANNN: Set index register to NNN
    SetIndexNnn { nnn: u16 },
    /// 0xBNNN: Jump to V0 + NNN
    JumpV0Nnn { nnn: u16 },
    /// 0xCXNN: Set VX to a random number AND'ed with NN
    SetVxRandNn { x: usize, nn: u8 },
    /// 0xDXYN: Display
    Display { x: usize, y: usize, n: u8 },
    /// 0xEX9E: Skip instruction if key VX is being pressed
    SkipIfVxPressed { x: usize },
    /// 0xEXA1: Skip instruction if key VX is not being pressed
    SkipIfVxNotPressed { x: usize },
    /// 0xFX07: Set VX to the current value of the delay timer
    SetVxDelay { x: usize },
    /// 0xFX15: Set the delay timer to the value in VX
    SetDelayVx { x: usize },
    /// 0xFX18: Set the sound timer to the value in VX
    SetSoundVx { x: usize },
    /// 0xFX1E: Add VX to I
    AddVxI { x: usize },
    /// 0xFX0A: Block until any key is pressed, put key in VX
    BlockUntilAnyKey { x: usize },
    /// 0xFX29: Set I to font character in VX
    SetIFontVx { x: usize },
    /// 0xFX33: Store 3 decimal digits of VX in I, I+1, I+2
    StoreVxDigitsI { x: usize },
    /// 0xFX55: Store all registers from V0 to VX in I, I+1, I+2, ... I+X
    StoreVxI { x: usize },
    /// 0xFX65: Store all memory from I, I+1, I+2, ... I+X in registers V0 to VX
    StoreIVx { x: usize },
}

pub(crate) fn parse_instruction(instruction: u16) -> Result<Instruction, String> {
    let op: u8 = (instruction >> 12) as u8;
    let x: usize = ((instruction & 0x0F00) >> 8) as usize;
    let y: usize = ((instruction & 0x00F0) >> 4) as usize;
    let nnn: u16 = instruction & 0x0FFF;
    let nn: u8 = (instruction & 0x00FF) as u8;
    let n: u8 = (instruction & 0x000F) as u8;

    match (op, x, y, n) {
        (0, 0, 0xE, 0) => Ok(Instruction::ClearScreen),
        (0, 0, 0xE, 0xE) => Ok(Instruction::SubroutineReturn),
        (1, _, _, _) => Ok(Instruction::Jump { nnn }),
        (2, _, _, _) => Ok(Instruction::SubroutineCall { nnn }),
        (3, _, _, _) => Ok(Instruction::SkipVxEqNn { x, nn }),
        (4, _, _, _) => Ok(Instruction::SkipVxNeqNn { x, nn }),
        (5, _, _, _) => Ok(Instruction::SkipVxEqVy { x, y }),
        (6, _, _, _) => Ok(Instruction::SetVxNn { x, nn }),
        (7, _, _, _) => Ok(Instruction::AddNnVx { x, nn }),
        (8, _, _, 0) => Ok(Instruction::SetVxVy { x, y }),
        (8, _, _, 1) => Ok(Instruction::SetVxOrVy { x, y }),
        (8, _, _, 2) => Ok(Instruction::SetVxAndVy { x, y }),
        (8, _, _, 3) => Ok(Instruction::SetVxXorVy { x, y }),
        (8, _, _, 4) => Ok(Instruction::SetVxPlusVy { x, y }),
        (8, _, _, 5) => Ok(Instruction::SetVxMinusVy { x, y }),
        (8, _, _, 6) => Ok(Instruction::ShiftVxRight { x }),
        (8, _, _, 7) => Ok(Instruction::SetVyMinusVx { x, y }),
        (8, _, _, 0xE) => Ok(Instruction::ShiftVxLeft { x }),
        (9, _, _, _) => Ok(Instruction::SkipVxNeqVy { x, y }),
        (0xA, _, _, _) => Ok(Instruction::SetIndexNnn { nnn }),
        (0xB, _, _, _) => Ok(Instruction::JumpV0Nnn { nnn }),
        (0xC, _, _, _) => Ok(Instruction::SetVxRandNn { x, nn }),
        (0xD, _, _, _) => Ok(Instruction::Display { x, y, n }),
        (0xE, _, 9, 0xE) => Ok(Instruction::SkipIfVxPressed { x }),
        (0xE, _, 0xA, 1) => Ok(Instruction::SkipIfVxNotPressed { x }),
        (0xF, _, 0, 7) => Ok(Instruction::SetVxDelay { x }),
        (0xF, _, 1, 5) => Ok(Instruction::SetDelayVx { x }),
        (0xF, _, 1, 8) => Ok(Instruction::SetSoundVx { x }),
        (0xF, _, 1, 0xE) => Ok(Instruction::AddVxI { x }),
        (0xF, _, 0, 0xA) => Ok(Instruction::BlockUntilAnyKey { x }),
        (0xF, _, 2, 9) => Ok(Instruction::SetIFontVx { x }),
        (0xF, _, 3, 3) => Ok(Instruction::StoreVxDigitsI { x }),
        (0xF, _, 5, 5) => Ok(Instruction::StoreVxI { x }),
        (0xF, _, 6, 5) => Ok(Instruction::StoreIVx { x }),
        _ => Err(format!("Unknown instruction {:#04X?}", instruction)),
    }
}

#[test]
fn test_parse_instruction() {
    let assert_parse = |raw: u16, instruction: Instruction| {
	assert_eq!(parse_instruction(raw), Ok(instruction));
    };
    assert_parse(0x00E0, Instruction::ClearScreen);
    assert_parse(0x00EE, Instruction::SubroutineReturn);
    assert_parse(0x1ABC, Instruction::Jump { nnn: 0xABC });
    assert_parse(0x2ABC, Instruction::SubroutineCall { nnn: 0xABC });
    assert_parse(0x3ABC, Instruction::SkipVxEqNn { x: 0xA, nn: 0xBC });
    assert_parse(0x4ABC, Instruction::SkipVxNeqNn { x: 0xA, nn: 0xBC });
    assert_parse(0x5ABC, Instruction::SkipVxEqVy { x: 0xA, y: 0xB });
    assert_parse(0x6ABC, Instruction::SetVxNn { x: 0xA, nn: 0xBC });
    assert_parse(0x7ABC, Instruction::AddNnVx { x: 0xA, nn: 0xBC });
    assert_parse(0x8AB0, Instruction::SetVxVy { x: 0xA, y: 0xB });
    assert_parse(0x8AB1, Instruction::SetVxOrVy { x: 0xA, y: 0xB });
    assert_parse(0x8AB2, Instruction::SetVxAndVy { x: 0xA, y: 0xB });
    assert_parse(0x8AB3, Instruction::SetVxXorVy { x: 0xA, y: 0xB });
    assert_parse(0x8AB4, Instruction::SetVxPlusVy { x: 0xA, y: 0xB });
    assert_parse(0x8AB5, Instruction::SetVxMinusVy { x: 0xA, y: 0xB });
    assert_parse(0x8AB6, Instruction::ShiftVxRight { x: 0xA });
    assert_parse(0x8AB7, Instruction::SetVyMinusVx { x: 0xA, y: 0xB });
    assert_parse(0x8ABE, Instruction::ShiftVxLeft { x: 0xA });
    assert_parse(0x9ABC, Instruction::SkipVxNeqVy { x: 0xA, y: 0xB });
    assert_parse(0xAABC, Instruction::SetIndexNnn { nnn: 0xABC });
    assert_parse(0xBABC, Instruction::JumpV0Nnn { nnn: 0xABC });
    assert_parse(0xCABC, Instruction::SetVxRandNn { x: 0xA, nn: 0xBC });
    assert_parse(0xDABC, Instruction::Display { x: 0xA, y: 0xB, n: 0xC });
    assert_parse(0xE19E, Instruction::SkipIfVxPressed { x: 1 });
    assert_parse(0xE2A1, Instruction::SkipIfVxNotPressed { x: 2 });
    assert_parse(0xF307, Instruction::SetVxDelay { x: 3 });
    assert_parse(0xF415, Instruction::SetDelayVx { x: 4 });
    assert_parse(0xF518, Instruction::SetSoundVx { x: 5 });
    assert_parse(0xF61E, Instruction::AddVxI { x: 6 });
    assert_parse(0xF70A, Instruction::BlockUntilAnyKey { x: 7 });
    assert_parse(0xF829, Instruction::SetIFontVx { x: 8 });
    assert_parse(0xF933, Instruction::StoreVxDigitsI { x: 9 });
    assert_parse(0xFA55, Instruction::StoreVxI { x: 0xA });
    assert_parse(0xFB65, Instruction::StoreIVx { x: 0xB });
}
