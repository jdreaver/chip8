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

## Resources

- https://tobiasvl.github.io/blog/write-a-chip-8-emulator/
- https://wiki.libsdl.org/ (for drawing to screen)
- https://www.reddit.com/r/C_Programming/comments/lcgwj3/a_simple_and_beginner_friendly_chip8_emulator/
  - https://github.com/f0lg0/CHIP-8


## TODO

- Add tests for C, just to practice testing C code.
- Terminal graphics instead of SDL
  - Make this an option?
- Move a lot of the #defines into chip8_state and make them configurable
- More accurate simulated clock speed
