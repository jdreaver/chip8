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
    sp: u8, // Points to element after top of stack (starts at 0 when stack empty)
    stack: [u16; MAX_STACK_SIZE],

    // General purpose registers
    v: [u8; 16],
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
        }
    }
}

fn load_rom_file(vm: &mut VM, path: &Path) -> io::Result<()> {
    let mut f = File::open(path)?;

    // Rom memory starts at 0x200
    f.read(&mut vm.memory[0x200..])?;

    Ok(())
}

fn processor_cycle(vm: &mut VM) {
    // Instructions are two bytes
    let instruction: u16 =
        (vm.memory[vm.pc as usize] as u16) << 8 | vm.memory[vm.pc as usize + 1] as u16;

    // Increment program counter here so we don't forget later
    vm.pc += 2;

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
        0x0 => match nnn {
            0x00E0 => {
                // Clear screen
                for i in 0..DISPLAY_WIDTH_PX {
                    for j in 0..DISPLAY_HEIGHT_PX {
                        vm.display[i][j] = false;
                    }
                }
            }
            _ => exit_unknown_instruction(instruction, vm.pc),
        },
        // 0x1NNN: Jump to NNN
        0x1 => vm.pc = nnn,
        // 0x6NNN: Set register VX to NN
        0x6 => vm.v[x] = nn,
        // 0x7NNN: Add NN to VX, ignoring carry
        0x7 => vm.v[x] += nn,
        // 0xANNN: Set index register to NNN
        0xA => vm.ir = nnn,
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
        _ => exit_unknown_instruction(instruction, vm.pc),
    }
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
