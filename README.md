# Chip 8 Emulator

## Resources

- https://tobiasvl.github.io/blog/write-a-chip-8-emulator/
- https://wiki.libsdl.org/ (for drawing to screen)
- https://www.reddit.com/r/C_Programming/comments/lcgwj3/a_simple_and_beginner_friendly_chip8_emulator/
  - https://github.com/f0lg0/CHIP-8


## TODO

- Terminal graphics instead of SDL
  - Make this an option?
- Move stack to the end of memory, don't use a separate array
- Move a lot of the #defines into chip8_state and make them configurable
- More accurate simulated clock speed
