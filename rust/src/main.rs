extern crate sdl2;

use std::cmp::min;
use std::env;
use std::fs::File;
use std::io;
use std::io::Read;
use std::os::unix::prelude::MetadataExt;
use std::path::Path;

const MEMORY_BYTES: usize = 4096;

const DISPLAY_WIDTH_PX: usize = 64;
const DISPLAY_HEIGHT_PX: usize = 32;
const PIXEL_SCALE_FACTOR: usize = 8;

const PROCESSOR_SPEED_HZ: u64 = 700;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 1 {
        eprintln!("Usage: chip8 ROM-FILE");
        std::process::exit(1);
    }

    let rom_path = Path::new(&args[0]);
    let mut vm = VM::new();

    if let Err(err) = load_rom_file(&mut vm.memory, &rom_path) {
        eprintln!("Error loading ROM file {}: {}", &rom_path.display(), err);
        std::process::exit(1);
    }

    let mut canvas = create_sdl_window();

    // Draw test pixel
    vm.display[55][2] = true;

    loop {
	// TODO: Process SDL events for keypresses

        if let Err(err) = processor_cycle(&mut vm) {
            eprintln!("Error in processor cycle: {}", err);
            std::process::exit(1);
        }

        // TODO: Only draw display when display is updated
        // (set a bit on instructions in processor_cycle that
        // update the screen)
        draw_display(&mut canvas, vm.display);

        // TODO: Perform more accurate clock speed emulation
        // by using clock_gettime(CLOCK_MONOTONIC, ...),
        // recording the nanosecond time of the last
        // instruction, and trying to sleep until the next
        // instruction execution time.
        std::thread::sleep(std::time::Duration::from_micros(
            1000000 / PROCESSOR_SPEED_HZ,
        ));
    }
}

const MAX_STACK_SIZE: usize = 100;

struct VM {
    memory: Memory,
    display: Display,

    // Program counter
    pc: u16,

    // Index register
    ir: u16,

    // Stack is for subroutines
    sp: usize, // Points to element after top of stack (starts at 0 when stack empty)
    stack: CallStack,

    // General purpose registers
    v: [u8; 16],

    keys_pressed: [bool; 16],

    // Timers decremented at 60 Hz
    delay_timer: u8,
    sound_timer: u8,
}

type Memory = [u8; MEMORY_BYTES];
type Display = [[bool; DISPLAY_HEIGHT_PX]; DISPLAY_WIDTH_PX];
type CallStack = [u16; MAX_STACK_SIZE];

impl VM {
    fn new() -> VM {
        VM {
            memory: [0; MEMORY_BYTES],
            display: [[false; DISPLAY_HEIGHT_PX]; DISPLAY_WIDTH_PX],
            pc: 0x200,
            ir: 0,
            sp: 0,
            stack: [0; MAX_STACK_SIZE],
            v: [0; 16],
            keys_pressed: [false; 16],
            delay_timer: 0,
            sound_timer: 0,
        }
    }
}

const FONT_MEMORY_START: usize = 0x050;

const FONT_BYTES: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

fn load_rom_file(memory: &mut Memory, path: &Path) -> io::Result<()> {
    let mut f = File::open(path)?;
    let metadata = f.metadata()?;

    // Rom memory starts at 0x200
    let amount_read = f.read(&mut memory[0x200..])?;
    if amount_read as u64 != metadata.size() {
        eprintln!("Read {} bytes of ROM file, but file size is {}", amount_read, metadata.size());
        std::process::exit(1);
    }

    // Load font into 0x050–0x09F
    memory[0x050..=0x09F].copy_from_slice(&FONT_BYTES);

    Ok(())
}

fn processor_cycle(vm: &mut VM) -> Result<(), String> {
    // Instructions are two bytes
    let raw_instruction: u16 =
        (vm.memory[vm.pc as usize] as u16) << 8 | vm.memory[vm.pc as usize + 1] as u16;

    // println!("instruction {:#04X?} (PC: {:#04X?})", instruction, &current_pc);

    // Increment program counter here instead of in each instruction
    // so we don't forget.
    vm.pc += 2;

    match parse_instruction(raw_instruction)? {
        Instruction::ClearScreen => {
            for i in 0..DISPLAY_WIDTH_PX {
                for j in 0..DISPLAY_HEIGHT_PX {
                    vm.display[i][j] = false;
                }
            }
        }
        Instruction::SubroutineReturn => {
            if vm.sp == 0 {
                eprintln!("internal error: pop from empty stack! instruction {:#04X?} (PC: {:#04X?})", raw_instruction, vm.pc);
                std::process::exit(1);
            }
            vm.pc = vm.stack[vm.sp - 1];
            vm.sp -= 1;
        }
        Instruction::Jump { nnn } => vm.pc = nnn,
        Instruction::SubroutineCall { nnn } => {
            // Add old PC to stack
            if vm.sp == MAX_STACK_SIZE {
                eprintln!(
                    "stack overflow! instruction {:#04X?} (PC: {:#04X?})",
                    raw_instruction, vm.pc
                );
                std::process::exit(1);
            }
            vm.stack[vm.sp] = vm.pc;
            vm.sp += 1;

            // Jump to NNN
            vm.pc = nnn;
        }
        Instruction::SkipVxEqNn { x, nn } => {
            if vm.v[x] == nn {
                vm.pc += 2
            }
        }
        Instruction::SkipVxNeqNn { x, nn } => {
            if vm.v[x] != nn {
                vm.pc += 2
            }
        }
        Instruction::SkipVxEqVy { x, y } => {
            if vm.v[x] == vm.v[y] {
                vm.pc += 2
            }
        }
        Instruction::SkipVxNeqVy { x, y } => {
            if vm.v[x] != vm.v[y] {
                vm.pc += 2
            }
        }
        Instruction::SetVxNn { x, nn } => vm.v[x] = nn,
        Instruction::AddNnVx { x, nn } => vm.v[x] = vm.v[x].wrapping_add(nn),
        Instruction::SetVxVy { x, y } => vm.v[x] = vm.v[y],
        Instruction::SetVxOrVy { x, y } => vm.v[x] |= vm.v[y],
        Instruction::SetVxAndVy { x, y } => vm.v[x] &= vm.v[y],
        Instruction::SetVxXorVy { x, y } => vm.v[x] ^= vm.v[y],
        Instruction::SetVxPlusVy { x, y } => match vm.v[x].checked_add(vm.v[y]) {
            Some(sum) => vm.v[x] = sum,
            None => {
                // Set overflow register
                vm.v[0xF] = 1;
                vm.v[x] = vm.v[x].wrapping_add(vm.v[y]);
            }
        }
        Instruction::SetVxMinusVy { x, y } => {
            vm.v[0xF] = (vm.v[x] > vm.v[y]) as u8;
            vm.v[x] = vm.v[x].wrapping_sub(vm.v[y]);
        }
	Instruction::ShiftVxRight { x } => {
            vm.v[0xF] = vm.v[x] & 0x1;
            vm.v[x] >>= 1;
        }
        Instruction::SetVyMinusVx { x, y } => {
            vm.v[0xF] = (vm.v[y] > vm.v[x]) as u8;
            vm.v[x] = vm.v[y].wrapping_sub(vm.v[x]);
        }
	Instruction::ShiftVxLeft { x } => {
            vm.v[0xF] = (vm.v[x] >> 7) & 0x1;
            vm.v[x] <<= 1;
        }
        Instruction::SetIndexNnn { nnn } => vm.ir = nnn,
        Instruction::JumpV0Nnn { nnn } => vm.pc = vm.v[0] as u16 + nnn,
        Instruction::SetVxRandNn { x, nn } => vm.v[x] = rand::random::<u8>() & nn,
        Instruction::Display { x, y, n } => {
            // Display n-byte sprite starting at memory location I at
            // (Vx, Vy), set VF = collision.
            let dx: u16 = vm.v[x] as u16 % DISPLAY_WIDTH_PX as u16;
            let dy: u16 = vm.v[y] as u16 % DISPLAY_HEIGHT_PX as u16;

            // Reset collision flag
            vm.v[0xF] = 0;

            // Read n bytes from memory. j is the y value
            for j in 0..min(n as u16, DISPLAY_HEIGHT_PX as u16 - dy) {
                let sprite_row: u8 = vm.memory[(vm.ir + j) as usize];

                // i is the x value we use to iterate over bits
                for i in 0..min(8, DISPLAY_WIDTH_PX as u16 - dx) {
                    // Bit shift to get the current row bit
                    let sprite_bit: bool = ((sprite_row >> (7 - i)) & 0b1) == 1;

                    if vm.display[(dx + i) as usize][(dy + j) as usize] && sprite_bit {
                        // Set collision register
                        vm.v[0xF] = 1;
                    }

                    // XOR with current bit
                    vm.display[(dx + i) as usize][(dy + j) as usize] ^= sprite_bit;
                }
            }
        }
        Instruction::SkipIfVxPressed { x } => {
            if vm.keys_pressed[vm.v[x] as usize] {
                vm.pc += 2;
            }
	}
        Instruction::SkipIfVxNotPressed { x } => {
            if !vm.keys_pressed[vm.v[x] as usize] {
                vm.pc += 2;
            }
        }
        Instruction::SetVxDelay { x } => vm.v[x] = vm.delay_timer,
        Instruction::SetDelayVx { x } => vm.delay_timer = vm.v[x],
        Instruction::SetSoundVx { x } => vm.sound_timer = vm.v[x],
        Instruction::AddVxI { x } => match (vm.v[x] as u16).checked_add(vm.ir) {
            // Overflow behavior is non-standard, but assumed safe
            Some(sum) => vm.ir = sum,
            None => {
                // Set overflow register
                vm.v[0xF] = 1;
                vm.ir = (vm.v[x] as u16).wrapping_add(vm.ir);
            }
        }
        Instruction::BlockUntilAnyKey { x } => {
            // Decrement program counter to repeat this
            // instruction in case a key isn't pressed
            vm.pc -= 2;
            for i in 0..0xF {
                if vm.keys_pressed[i] {
                    vm.v[x] = i as u8;
                    vm.pc += 2;
                }
            }
        }
        Instruction::SetIFontVx { x } => vm.ir = FONT_MEMORY_START as u16 + vm.v[x] as u16 * 5, // Fonts are 5 bytes wide
        Instruction::StoreVxDigitsI { x } => {
            vm.memory[vm.ir as usize] = vm.v[x] / 100;
            vm.memory[vm.ir as usize + 1] = (vm.v[x] % 100) / 10;
            vm.memory[vm.ir as usize + 2] = vm.v[x] % 10;
        }
        Instruction::StoreVxI { x } => {
            for i in 0..=x {
                vm.memory[vm.ir as usize + i as usize] = vm.v[i];
            }
        }
        Instruction::StoreIVx { x } => {
            for i in 0..=x {
                vm.v[i] = vm.memory[vm.ir as usize + i as usize];
            }
        }
    }

    Ok(())
}

#[derive(Debug, PartialEq)]
enum Instruction {
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

fn parse_instruction(instruction: u16) -> Result<Instruction, String> {
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
    assert_eq!(parse_instruction(0x00E0), Ok(Instruction::ClearScreen));
    assert_eq!(parse_instruction(0x00EE), Ok(Instruction::SubroutineReturn));
    assert_eq!(parse_instruction(0x1ABC), Ok(Instruction::Jump { nnn: 0xABC }));
    assert_eq!(parse_instruction(0x2ABC), Ok(Instruction::SubroutineCall { nnn: 0xABC }));
    assert_eq!(parse_instruction(0x3ABC), Ok(Instruction::SkipVxEqNn { x: 0xA, nn: 0xBC }));
    assert_eq!(parse_instruction(0x4ABC), Ok(Instruction::SkipVxNeqNn { x: 0xA, nn: 0xBC }));
    assert_eq!(parse_instruction(0x5ABC), Ok(Instruction::SkipVxEqVy { x: 0xA, y: 0xB }));
    assert_eq!(parse_instruction(0x6ABC), Ok(Instruction::SetVxNn { x: 0xA, nn: 0xBC }));
    assert_eq!(parse_instruction(0x7ABC), Ok(Instruction::AddNnVx { x: 0xA, nn: 0xBC }));
    assert_eq!(parse_instruction(0x8AB0), Ok(Instruction::SetVxVy { x: 0xA, y: 0xB }));
    assert_eq!(parse_instruction(0x8AB1), Ok(Instruction::SetVxOrVy { x: 0xA, y: 0xB }));
    assert_eq!(parse_instruction(0x8AB2), Ok(Instruction::SetVxAndVy { x: 0xA, y: 0xB }));
    assert_eq!(parse_instruction(0x8AB3), Ok(Instruction::SetVxXorVy { x: 0xA, y: 0xB }));
    assert_eq!(parse_instruction(0x8AB4), Ok(Instruction::SetVxPlusVy { x: 0xA, y: 0xB }));
    assert_eq!(parse_instruction(0x8AB5), Ok(Instruction::SetVxMinusVy { x: 0xA, y: 0xB }));
    assert_eq!(parse_instruction(0x8AB6), Ok(Instruction::ShiftVxRight { x: 0xA }));
    assert_eq!(parse_instruction(0x8AB7), Ok(Instruction::SetVyMinusVx { x: 0xA, y: 0xB }));
    assert_eq!(parse_instruction(0x8ABE), Ok(Instruction::ShiftVxLeft { x: 0xA }));
    assert_eq!(parse_instruction(0x9ABC), Ok(Instruction::SkipVxNeqVy { x: 0xA, y: 0xB }));
    assert_eq!(parse_instruction(0xAABC), Ok(Instruction::SetIndexNnn { nnn: 0xABC }));
    assert_eq!(parse_instruction(0xBABC), Ok(Instruction::JumpV0Nnn { nnn: 0xABC }));
    assert_eq!(parse_instruction(0xCABC), Ok(Instruction::SetVxRandNn { x: 0xA, nn: 0xBC }));
    assert_eq!(parse_instruction(0xDABC), Ok(Instruction::Display { x: 0xA, y: 0xB, n: 0xC }));
    assert_eq!(parse_instruction(0xE19E), Ok(Instruction::SkipIfVxPressed { x: 1 }));
    assert_eq!(parse_instruction(0xE2A1), Ok(Instruction::SkipIfVxNotPressed { x: 2 }));
    assert_eq!(parse_instruction(0xF307), Ok(Instruction::SetVxDelay { x: 3 }));
    assert_eq!(parse_instruction(0xF415), Ok(Instruction::SetDelayVx { x: 4 }));
    assert_eq!(parse_instruction(0xF518), Ok(Instruction::SetSoundVx { x: 5 }));
    assert_eq!(parse_instruction(0xF61E), Ok(Instruction::AddVxI { x: 6 }));
    assert_eq!(parse_instruction(0xF70A), Ok(Instruction::BlockUntilAnyKey { x: 7 }));
    assert_eq!(parse_instruction(0xF829), Ok(Instruction::SetIFontVx { x: 8 }));
    assert_eq!(parse_instruction(0xF933), Ok(Instruction::StoreVxDigitsI { x: 9 }));
    assert_eq!(parse_instruction(0xFA55), Ok(Instruction::StoreVxI { x: 0xA }));
    assert_eq!(parse_instruction(0xFB65), Ok(Instruction::StoreIVx { x: 0xB }));
}

fn create_sdl_window() -> sdl2::render::Canvas<sdl2::video::Window> {
    let sdl_context = sdl2::init().expect("failed to init SDL context");
    let video_subsystem = sdl_context
        .video()
        .expect("failed to init SDL video subsystem");

    let window_width = (DISPLAY_WIDTH_PX * PIXEL_SCALE_FACTOR) as u32;
    let window_height = (DISPLAY_HEIGHT_PX * PIXEL_SCALE_FACTOR) as u32;
    let window = video_subsystem
        .window("CHIP-8", window_width, window_height)
        .position_centered()
        .opengl()
        .build()
        .expect("failed to create SDL window");
    window
        .into_canvas()
        .build()
        .expect("failed to create SDL canvas")
}

fn draw_display(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    display: Display,
) {
    canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
    canvas.clear();

    canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 255, 255)); // White

    for i in 0..DISPLAY_WIDTH_PX {
        for j in 0..DISPLAY_HEIGHT_PX {
            if display[i][j] {
                let rect = sdl2::rect::Rect::new(
                    (i * PIXEL_SCALE_FACTOR) as i32, // x
                    (j * PIXEL_SCALE_FACTOR) as i32, // y
                    PIXEL_SCALE_FACTOR as u32,       // width
                    PIXEL_SCALE_FACTOR as u32,       // height
                );
                if let Err(err) = canvas.fill_rect(rect) {
                    eprintln!("Error drawing rectangle {:?}: {}", rect, err);
                    std::process::exit(1);
                }
            }
        }
    }

    canvas.present();
}
