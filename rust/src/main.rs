extern crate sdl2;

use std::env;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

const MEMORY_BYTES: usize = 4096;

const DISPLAY_WIDTH_PX: usize = 64;
const DISPLAY_HEIGHT_PX: usize = 32;
const PIXEL_SCALE_FACTOR: usize = 8;

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

    println!("Loaded ROM file {}", rom_path.display());
    println!("First 8 bytes: {:#04X?}", &vm.memory[0x200..(0x200 + 8)]);

    let mut canvas = create_sdl_window();

    // Draw test pixel
    vm.display[55][2] = true;
    draw_display(&mut canvas, vm.display);
    ::std::thread::sleep(::std::time::Duration::new(2, 0));

}

struct VM {
    memory: [u8; MEMORY_BYTES],
    display: [[bool; DISPLAY_HEIGHT_PX]; DISPLAY_WIDTH_PX],
}

impl VM {
    fn new() -> VM {
        VM {
            memory: [0; MEMORY_BYTES],
            display: [[false; DISPLAY_HEIGHT_PX]; DISPLAY_WIDTH_PX],
        }
    }
}

fn load_rom_file(vm: &mut VM, path: &Path) -> io::Result<()> {
    let mut f = File::open(path)?;

    // Rom memory starts at 0x200
    f.read(&mut vm.memory[0x200..])?;

    Ok(())
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
