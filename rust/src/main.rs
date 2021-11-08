extern crate sdl2;

use std::cmp::min;
use std::env;
use std::fs::File;
use std::io;
use std::io::Read;
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

    if let Err(err) = load_rom_file(&mut vm, &rom_path) {
        eprintln!("Error loading ROM file {}: {}", &rom_path.display(), err);
        std::process::exit(1);
    }

    let mut canvas = create_sdl_window();

    // Draw test pixel
    vm.display[55][2] = true;

    loop {
        processor_cycle(&mut vm);

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
    memory: [u8; MEMORY_BYTES],
    display: [[bool; DISPLAY_HEIGHT_PX]; DISPLAY_WIDTH_PX],

    // Program counter
    pc: u16,

    // Index register
    ir: u16,

    // Stack is for subroutines
    sp: usize, // Points to element after top of stack (starts at 0 when stack empty)
    stack: [u16; MAX_STACK_SIZE],

    // General purpose registers
    v: [u8; 16],

    keys_pressed: [bool; 16],

    // Timers decremented at 60 Hz
    delay_timer: u8,
    sound_timer: u8,
}

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
	0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

fn load_rom_file(vm: &mut VM, path: &Path) -> io::Result<()> {
    let mut f = File::open(path)?;

    // Rom memory starts at 0x200
    f.read(&mut vm.memory[0x200..])?;

    // Load font into 0x050–0x09F
    vm.memory[0x050..=0x09F].copy_from_slice(&FONT_BYTES);

    Ok(())
}

fn processor_cycle(vm: &mut VM) {
    // Instructions are two bytes
    let instruction: u16 =
        (vm.memory[vm.pc as usize] as u16) << 8 | vm.memory[vm.pc as usize + 1] as u16;

    println!("instruction {:#04X?} (PC: {:#04X?})", instruction, vm.pc);

    // TODO: Parse instructions into an enum, and then process them in
    // a second stage.

    // Extract common parts of instruction here so we don't make mistakes later.
    // O (opcode): O___
    // X: _X__
    // Y: __Y_
    // NNN: _NNN
    // NN: __NN
    // N: ___N
    let op: u8 = (instruction >> 12) as u8;
    let x: usize = ((instruction & 0x0F00) >> 8) as usize;
    let y: usize = ((instruction & 0x00F0) >> 4) as usize;
    let nnn: u16 = instruction & 0x0FFF;
    let nn: u8 = (instruction & 0x00FF) as u8;
    let n: u8 = (instruction & 0x000F) as u8;

    match op {
        0x0 => {
            match nnn {
                // Clear screen
                0x00E0 => {
                    for i in 0..DISPLAY_WIDTH_PX {
                        for j in 0..DISPLAY_HEIGHT_PX {
                            vm.display[i][j] = false;
                        }
                    }
                }
                // Return from subroutine
                0x00EE => {
                    if vm.sp == 0 {
                        eprintln!("internal error: pop from empty stack! instruction {:#04X?} (PC: {:#04X?})", instruction, vm.pc);
                        std::process::exit(1);
                    }
                    vm.pc = vm.stack[vm.sp - 1];
                    vm.sp -= 1;
                }
                _ => exit_unknown_instruction(instruction, vm.pc),
            }
        }
        // 0x1NNN: Jump to NNN
        0x1 => vm.pc = nnn,
        // Subroutine call (0x2NNN) at location NNN
        0x2 => {
            // Add old PC to stack
            if vm.sp == MAX_STACK_SIZE {
                eprintln!(
                    "stack overflow! instruction {:#04X?} (PC: {:#04X?})",
                    instruction, vm.pc
                );
                std::process::exit(1);
            }
            vm.stack[vm.sp] = vm.pc;
            vm.sp += 1;

            // Jump to NNN
            vm.pc = nnn;
        }

        // All of the skip routines (including 9XY0, which is included here out of order)
        // 0x3XNN, skip if VX == NN
        0x3 => {
            if vm.v[x] == nn {
                vm.pc += 2
            }
        }
        // 0x4XNN, skip if VX != NN
        0x4 => {
            if vm.v[x] != nn {
                vm.pc += 2
            }
        }
        // 0x5XY0, skip if VX == VY
        0x5 => {
            if vm.v[x] == vm.v[y] {
                vm.pc += 2
            }
        }
        // 0x9XY0, skip if VX != VY
        0x9 => {
            if vm.v[x] != vm.v[y] {
                vm.pc += 2
            }
        }

        // 0x6NNN: Set register VX to NN
        0x6 => vm.v[x] = nn,
        // 0x7NNN: Add NN to VX, ignoring carry
        0x7 => vm.v[x] = vm.v[x].wrapping_add(nn),

        0x8 => match n {
            // 0x8XY0: Set VX to VY
            0x0 => vm.v[x] = vm.v[y],
            // 0x8XY1: Set VX to VX | VY
            0x1 => vm.v[x] |= vm.v[y],
            // 0x8XY2: Set VX to VX & VY
            0x2 => vm.v[x] &= vm.v[y],
            // 0x8XY3: Set VX to VX XOR VY
            0x3 => vm.v[x] ^= vm.v[y],
            // 0x8XY4: Set VX to VX + VY, accounting for carry
            0x4 => match vm.v[x].checked_add(vm.v[y]) {
                Some(sum) => vm.v[x] = sum,
                None => {
                    // Set overflow register
                    vm.v[0xF] = 1;
                    vm.v[x] = vm.v[x].wrapping_add(vm.v[y]);
                }
            },
            // 0x8XY5: Set VX to VX - VY, accounting for carry
            0x5 => {
                vm.v[0xF] = (vm.v[x] > vm.v[y]) as u8;
                vm.v[x] = vm.v[x].wrapping_sub(vm.v[y]);
            }
            // 0x8XY6: Store least significant bit of VX in VF and shift VX right by 1
            0x6 => {
                vm.v[0xF] = vm.v[x] & 0x1;
                vm.v[x] >>= 1;
            }
            // 0x8XY7: Set VX to VY - VX, accounting for carry
            0x7 => {
                vm.v[0xF] = (vm.v[y] > vm.v[x]) as u8;
                vm.v[x] = vm.v[y].wrapping_sub(vm.v[x]);
            }
            // 0x8XYE: Store most significant bit of VX in VF and shift VX left by 1
            0xE => {
                vm.v[0xF] = (vm.v[x] >> 7) & 0x1;
                vm.v[x] <<= 1;
            }
            _ => exit_unknown_instruction(instruction, vm.pc),
        },

        // 0xANNN: Set index register to NNN
        0xA => vm.ir = nnn,
        // 0xBNNN: Jump to VX + NNN
        0xB => vm.pc = vm.v[x] as u16 + nnn,
        // 0xCXNN: Set VX to a random number AND'ed with NN
        0xC => vm.v[x] = rand::random::<u8>() & nn,
        // 0xDXYN: Display
        0xD => {
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

        0xE => match nn {
            // 0xEX9E: skip instruction if key VX is being pressed
            0x9E => {
                if vm.keys_pressed[vm.v[x] as usize] {
                    vm.pc += 2;
                }
            }
            // 0xEXA1: skip instruction if key VX is not being pressed
            0xA1 => {
                if !vm.keys_pressed[vm.v[x] as usize] {
                    vm.pc += 2;
                }
            }
            _ => exit_unknown_instruction(instruction, vm.pc),
        }

	0xF => match nn {
	    // 0xFX07: set VX to the current value of the delay timer
	    0x07 => vm.v[x] = vm.delay_timer,
	    // 0xFX15: set the delay timer to the value in VX
	    0x15 => vm.delay_timer = vm.v[x],
	    // 0xFX18: set the sound timer to the value in VX
	    0x18 => vm.sound_timer = vm.v[x],
	    // 0xFX1E: Add VX to I
	    0x1E => match (vm.v[x] as u16).checked_add(vm.ir) {
		// Overflow behavior is non-standard, but assumed safe
                Some(sum) => vm.ir = sum,
                None => {
                    // Set overflow register
                    vm.v[0xF] = 1;
                    vm.ir = (vm.v[x] as u16).wrapping_add(vm.ir);
                }
	    }
	    // 0xFX0A: Block until any key is pressed, put key in VX
	    0x0A => {
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
	    // 0xFX29: Set I to font character in VX
	    0x29 => vm.ir = FONT_MEMORY_START as u16 + vm.v[x] as u16 * 5, // Fonts are 5 bytes wide
	    // 0xFX33: Store 3 decimal digits of VX in I, I+1, I+2
	    0x33 => {
		vm.memory[vm.ir as usize] = vm.v[x] / 100;
		vm.memory[vm.ir as usize + 1] = (vm.v[x] % 100) / 10;
		vm.memory[vm.ir as usize + 2] = vm.v[x] % 10;
	    }
	    // 0xFX55: Store all registers from V0 to VX in I, I+1, I+2, ... I+X
	    0x55 => {
		for i in 0..=x {
		    vm.memory[vm.ir as usize + i as usize] = vm.v[i];
		}
	    }
	    // 0xFX65: Store all memory from I, I+1, I+2, ... I+X in registers V0 to VX
	    0x65 => {
		for i in 0..=x {
		    vm.v[i] = vm.memory[vm.ir as usize + i as usize];
		}
	    }
            _ => exit_unknown_instruction(instruction, vm.pc),
	}

        _ => exit_unknown_instruction(instruction, vm.pc),
    }

    // Increment program counter here instead of in each instruction
    // so we don't forget.
    vm.pc += 2;
}

// TODO: This should be a pure error value, not an exit
fn exit_unknown_instruction(instruction: u16, pc: u16) {
    eprintln!(
        "Unknown instruction {:#04X?} (PC: {:#04X?})",
        instruction, pc
    );
    std::process::exit(1);
}

// enum Instruction {
//     ClearScreen,
// }

// fn parse_instruction(instruction: u16) -> Instruction {
//     let op: u8 = (instruction >> 12) as u8;
//     let x: usize = ((instruction & 0x0F00) >> 8) as usize;
//     let y: usize = ((instruction & 0x00F0) >> 4) as usize;
//     let nnn: u16 = instruction & 0x0FFF;
//     let nn: u8 = (instruction & 0x00FF) as u8;
//     let n: u8 = (instruction & 0x000F) as u8;

//     fn exit_unknown() {
//         eprintln!("Unknown instruction {:#04X?}", instruction);
//         std::process::exit(1);
//     }

//     match op {
//         0x0 => match nnn {
//             0x00E0 => Instruction::ClearScreen,
//             _ => exit_unknown(),
//         }
//         _ => exit_unknown(),
//     }
// }

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
    let canvas = window
        .into_canvas()
        .build()
        .expect("failed to create SDL canvas");
    canvas
}

fn draw_display(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    display: [[bool; DISPLAY_HEIGHT_PX]; DISPLAY_WIDTH_PX],
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
