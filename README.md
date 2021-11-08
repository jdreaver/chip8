# CHIP-8 Emulator

This is my code for a [CHIP-8](https://en.wikipedia.org/wiki/CHIP-8)
emulator. Don't expect much. I'll probably complete this and then
abandon it, but it is a fun systems project.

## Running

### C Version

```sh
$ cd c/
$ make
$ ./bin/chip8 ../roms/ibm-logo.ch8
```

### Rust Version

```sh
$ cd rust/
$ cargo run -- ../roms/ibm-logo.ch8
```

## Resources

- https://tobiasvl.github.io/blog/write-a-chip-8-emulator/
- https://wiki.libsdl.org/ (for drawing to screen)

C:
- https://www.reddit.com/r/C_Programming/comments/lcgwj3/a_simple_and_beginner_friendly_chip8_emulator/
  - https://github.com/f0lg0/CHIP-8

Rust:
- https://dhole.github.io/post/chip8_emu_1/
- https://github.com/starrhorne/chip8-rust

## TODO

- Fix Rust version on the corex89 test. Figure out when memory
  location 248 is modified. Setting memory might be broken, or maybe
  setting ir is broken. (The AX test doesn't show OK either)
  - C version:
    ```
	...
	instruction: 0xd9b4
    instruction: 0x1248
    instruction: 0xa202
	...
    ```
  - Rust version:
    ```
	...
	instruction 0xD9B4 (PC: 0x3D8)
    instruction 0x1248 (PC: 0x3DA)
    instruction 0xDAB4 (PC: 0x24A) <--- wrong!
	...
    ```
- Add tests for C, just to practice testing C code.
- Terminal graphics instead of SDL
  - Make this an option?
- Move a lot of the #defines into chip8_state and make them configurable
- More accurate simulated clock speed
