mod display;
mod instruction;

use std::cmp::min;
use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io;
use std::io::Read;
use std::os::unix::prelude::MetadataExt;
use std::path::Path;

use instruction::{Instruction, parse_instruction};

const MEMORY_BYTES: usize = 4096;

const PROCESSOR_SPEED_HZ: u64 = 700;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 1 {
        eprintln!("Usage: chip8 ROM-FILE");
        std::process::exit(1);
    }

    let rom_path = Path::new(&args[0]);
    let mut vm = VM::new();

    if let Err(err) = load_rom_file(&mut vm.memory, rom_path) {
        eprintln!("Error loading ROM file {}: {}", &rom_path.display(), err);
        std::process::exit(1);
    }

    loop {
        // TODO: Process SDL events for keypresses

        if let Err(err) = processor_cycle(&mut vm) {
            eprintln!("Error in processor cycle: {}", err);
            std::process::exit(1);
        }

        vm.display.paint();

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

struct VM {
    memory: Memory,
    display: display::Display,

    // Program counter
    pc: u16,

    // Index register
    ir: u16,

    // Stack is for subroutines
    stack: VecDeque<u16>,

    // General purpose registers
    v: [u8; 16],

    keys_pressed: [bool; 16],

    // Timers decremented at 60 Hz
    delay_timer: u8,
    sound_timer: u8,
}

type Memory = [u8; MEMORY_BYTES];

impl VM {
    fn new() -> VM {
        VM {
            memory: [0; MEMORY_BYTES],
            display: display::Display::new(),
            pc: 0x200,
            ir: 0,
            stack: VecDeque::new(),
            v: [0; 16],
            keys_pressed: [false; 16],
            delay_timer: 0,
            sound_timer: 0,
        }
    }
}

const FONT_MEMORY_START: usize = 0x050;

static FONT_BYTES: [u8; 80] = [
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
        eprintln!(
            "Read {} bytes of ROM file, but file size is {}",
            amount_read,
            metadata.size()
        );
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
        Instruction::ClearScreen => vm.display.clear(),
        Instruction::SubroutineReturn => {
	    match vm.stack.pop_back() {
		None => {
                    eprintln!(
			"internal error: pop from empty stack! instruction {:#04X?} (PC: {:#04X?})",
			raw_instruction, vm.pc
                    );
                    std::process::exit(1);
		}
		Some(pc) => vm.pc = pc,
	    }
        }
        Instruction::Jump { nnn } => vm.pc = nnn,
        Instruction::SubroutineCall { nnn } => {
            vm.stack.push_back(vm.pc);
            vm.pc = nnn; // Jump to NNN
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
        },
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
            let dx: u16 = vm.v[x] as u16 % display::DISPLAY_WIDTH_PX as u16;
            let dy: u16 = vm.v[y] as u16 % display::DISPLAY_HEIGHT_PX as u16;

            // Reset collision flag
            vm.v[0xF] = 0;

            // Read n bytes from memory. j is the y value
            for j in 0..min(n as u16, display::DISPLAY_HEIGHT_PX as u16 - dy) {
                let sprite_row: u8 = vm.memory[(vm.ir + j) as usize];

                // i is the x value we use to iterate over bits
                for i in 0..min(8, display::DISPLAY_WIDTH_PX as u16 - dx) {
                    // Bit shift to get the current row bit
                    let sprite_bit: bool = ((sprite_row >> (7 - i)) & 0b1) == 1;

		    let pixel = vm.display.get_pixel((dx + i) as usize, (dy + j) as usize);
                    if pixel && sprite_bit {
                        // Set collision register
                        vm.v[0xF] = 1;
                    }

                    // XOR with current bit
                    vm.display.set_pixel((dx + i) as usize, (dy + j) as usize, pixel ^ sprite_bit);
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
        },
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
