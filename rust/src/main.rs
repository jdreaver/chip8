use std::env;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

const MEMORY_BYTES: usize = 4096;

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
}

struct VM {
    memory: [u8; MEMORY_BYTES],
}

impl VM {
    fn new() -> VM {
        VM {
            memory: [0; MEMORY_BYTES],
        }
    }
}

fn load_rom_file(vm: &mut VM, path: &Path) -> io::Result<()> {
    let mut f = File::open(path)?;

    // Rom memory starts at 0x200
    f.read(&mut vm.memory[0x200..])?;

    Ok(())
}
