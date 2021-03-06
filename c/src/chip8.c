#include <SDL2/SDL.h>

#include <stdbool.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

#define PIXEL_SCALE_FACTOR 8

#define DISPLAY_WIDTH 64
#define DISPLAY_HEIGHT 32

#define MAX_STACK_SIZE 100

#define PROCESSOR_SPEED_HZ 700

typedef struct {
	uint8_t mem[4096];
	bool display[DISPLAY_WIDTH][DISPLAY_HEIGHT];
	uint16_t program_counter;
	uint16_t index_register;

	/* Stack is for subroutines */
	uint8_t stack_ptr; // Points to element after top of stack (starts at 0 when stack empty)
	uint16_t stack[MAX_STACK_SIZE];

	/* Timers decremented at 60 Hz */
	uint8_t delay_timer;
	uint8_t sound_timer;

	/* General purpose registers */
	uint8_t V[16];

	bool keys_pressed[16];
} chip8_state;

void init_state(chip8_state *state)
{
	memset(state->mem, 0, sizeof(state->mem));
	state->program_counter = 0x200;
	memset(state->display, 0, sizeof(state->display));
	memset(state->V, 0, sizeof(state->V));
	state->stack_ptr = 0;
}

typedef struct {
	SDL_Window *window;
	SDL_Renderer *renderer;
} chip8_screen;

void init_screen(chip8_screen *screen)
{
	SDL_Init(SDL_INIT_VIDEO);

	int window_width = DISPLAY_WIDTH * PIXEL_SCALE_FACTOR;
	int window_height = DISPLAY_HEIGHT * PIXEL_SCALE_FACTOR;
	screen->window = SDL_CreateWindow("CHIP-8", SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED, window_width, window_height, 0);
	screen->renderer = SDL_CreateRenderer(screen->window, -1, SDL_RENDERER_ACCELERATED);
}

#define FONT_MEMORY_START 0x050

uint8_t font[] = {
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
};

void draw_display(chip8_screen *screen, bool display[DISPLAY_WIDTH][DISPLAY_HEIGHT])
{
	// Clear screen to black
	SDL_SetRenderDrawColor(screen->renderer, 0, 0, 0, 255);
	SDL_RenderClear(screen->renderer);

	// Set drawing color to white
	SDL_SetRenderDrawColor(screen->renderer, 255, 255, 255, 255);

	for (int i = 0; i < DISPLAY_WIDTH; i++) {
		for (int j = 0; j < DISPLAY_HEIGHT; j++) {
			if (display[i][j]) {
				SDL_Rect rect = {
					.x = i * PIXEL_SCALE_FACTOR,
					.y = j * PIXEL_SCALE_FACTOR,
					.w = PIXEL_SCALE_FACTOR,
					.h = PIXEL_SCALE_FACTOR,
				};
				SDL_RenderFillRect(screen->renderer, &rect);
			}
		}
	}

	// Draw frame
	SDL_RenderPresent(screen->renderer);
}

void load_rom(char* filename, uint8_t mem[4096])
{
	FILE* fp = fopen(filename, "rb");

	if (fp == NULL) {
		perror("loading ROM file");
		exit(EXIT_FAILURE);
	}

	struct stat st;
	stat(filename, &st);
	size_t fsize = st.st_size;

	// Memory up to 0x200 is reserved for internal use
	size_t bytes_read = fread(mem + 0x200, 1, 4096 - 0x200, fp);

	if (bytes_read != fsize) {
		fprintf(stderr, "failed loading ROM into memory. %ld != %ld\n", bytes_read, fsize);
		if (feof(fp)) {
			fprintf(stderr, "ROM file hit EOF\n");
		}
		if (ferror(fp)) {
			fprintf(stderr, "ROM file had an error\n");
		}
		exit(EXIT_FAILURE);
	}

	fclose(fp);

        // Load font into 0x050???0x09F
	memcpy(mem + FONT_MEMORY_START, font, sizeof(font));
}

// Keyboard looks like this:
// 1 2 3 4
// Q W E R
// A S D F
// Z X C V
SDL_Scancode keymappings[16] = {
    SDL_SCANCODE_1, SDL_SCANCODE_2, SDL_SCANCODE_3, SDL_SCANCODE_4,
    SDL_SCANCODE_Q, SDL_SCANCODE_W, SDL_SCANCODE_E, SDL_SCANCODE_R,
    SDL_SCANCODE_A, SDL_SCANCODE_S, SDL_SCANCODE_D, SDL_SCANCODE_F,
    SDL_SCANCODE_Z, SDL_SCANCODE_X, SDL_SCANCODE_C, SDL_SCANCODE_V};


void process_sdl_events(SDL_Window *window, chip8_state *state)
{
	const Uint8* sdl_state = SDL_GetKeyboardState(NULL);
	SDL_Event event;
	while (SDL_PollEvent(&event)) {
		switch (event.type) {
		case SDL_QUIT:
			SDL_DestroyWindow(window);
			SDL_Quit();
			exit(EXIT_SUCCESS);
		default:
			if (sdl_state[SDL_SCANCODE_ESCAPE]) {
				// TODO: DRY this with above
				SDL_DestroyWindow(window);
				SDL_Quit();
				exit(EXIT_SUCCESS);
			}

			for (int i = 0; i <= 0xF; i++) {
				state->keys_pressed[i] = sdl_state[keymappings[i]];
			}
			break;
		}
	}
}

void exit_unknown_instruction(uint16_t instruction, uint16_t program_counter)
{
	fprintf(stderr, "Unknown instruction: 0x%04x (PC: 0x%x)\n", instruction, program_counter);
	exit(EXIT_FAILURE);
}

void processor_cycle(chip8_state *state)
{
	// Instructions are 2 bytes
	uint16_t instruction = state->mem[state->program_counter] << 8 | state->mem[state->program_counter + 1];
	state->program_counter += 2;

	//printf("instruction: 0x%04x\n", instruction);

	// Extract common parts of instruction here so we don't make mistakes later.
	// O (opcode): O___
        // X: _X__
        // Y: __Y_
	// NNN: _NNN
	// NN: __NN
	// N: ___N
	uint8_t op = instruction >> 12;
	uint8_t x = (instruction & 0x0F00) >> 8;
	uint8_t y = (instruction & 0x00F0) >> 4;
	uint16_t nnn = instruction & 0x0FFF;
	uint8_t nn = instruction & 0x00FF;
	uint8_t n = instruction & 0x000F;

	// First byte of instruction stores op code
	switch (op) {
	case 0x0:
		switch (instruction & 0x0FFF) {
		case 0x00E0: // Clear screen
			for (int i = 0; i < DISPLAY_WIDTH; i++) {
				for (int j = 0; j < DISPLAY_HEIGHT; j++) {
					state->display[i][j] = 0;
				}
			}
			// TODO: Set some bit here that says display was touched
			break;

		case 0x00EE: // Return from subroutine
			if (state->stack_ptr == 0) {
				fprintf(stderr, "internal error: pop from empty stack! instruction: %d (PC: %d)\n", instruction, state->program_counter);
				exit(EXIT_FAILURE);
			}
			state->program_counter = state->stack[state->stack_ptr - 1];
			state->stack_ptr--;
			break;
		default:
			exit_unknown_instruction(instruction, state->program_counter);
		}
		break;
        case 0x1: // Jump (0x1NNN) NNN is the new program counter
		state->program_counter = nnn;
		break;
        case 0x2: // Subroutine call (0x2NNN) at location NNN
		// Add old PC to stack
		if (state->stack_ptr == MAX_STACK_SIZE) {
			fprintf(stderr, "stack overflow! instruction: %d (PC: %d)\n", instruction, state->program_counter);
			exit(EXIT_FAILURE);
		}
		state->stack[state->stack_ptr] = state->program_counter;
		state->stack_ptr++;

		// Jump to NNN
		state->program_counter = nnn;
		break;

	// All of the skip routines (including 9XY0, which is included here out of order)
	case 0x3: // 0x3XNN, skip if VX == NN
		if (state->V[x] == nn) {
			state->program_counter += 2;
		}
		break;
	case 0x4: // 0x4XNN, skip if VX != NN
		if (state->V[x] != nn) {
			state->program_counter += 2;
		}
		break;
	case 0x5: // 0x5XY0, skip if VX == VY
		if (state->V[x] == state->V[y]) {
			state->program_counter += 2;
		}
		break;
	case 0x9: // 0x9XY0, skip if VX != VY
		if (state->V[x] != state->V[y]) {
			state->program_counter += 2;
		}
		break;

	case 0x6: // 0x6XNN: Set register VX to NN
		state->V[x] = nn;
		break;
	case 0x7: // 0x7XNN: Add NN to register VX, ignoring carry
		state->V[x] += nn;
		break;
	case 0x8:
		switch (n) {
		case 0x0: // 0x8XY0: Set VX to VY
			state->V[x] = state->V[y];
			break;
		case 0x1: // 0x8XY1: Set VX to VX | VY
			state->V[x] |= state->V[y];
			break;
		case 0x2: // 0x8XY2: Set VX to VX & VY
			state->V[x] &= state->V[y];
			break;
		case 0x3: // 0x8XY3: Set VX to VX XOR VY
			state->V[x] ^= state->V[y];
			break;
		case 0x4: // 0x8XY4: Set VX to VX + VY, accounting for carry
			// If the sum of vx and vx is less than one of
			// the operands (we pick vx arbitrarily), then
			// we saw overflow.
			state->V[0xF] = (state->V[x] + state->V[y]) < state->V[x];
			state->V[x] += state->V[y];
			break;
		case 0x5: // 0x8XY5: Set VX to VX - VY, accounting for carry
			state->V[0xF] = state->V[x] > state->V[y];
			state->V[x] = state->V[x] - state->V[y];
			break;
		case 0x6: // 0x8XY6: Store least significant bit of VX in VF and shift VX right by 1
			state->V[0xF] = state->V[x] & 0x1;
			state->V[x] >>= 1;
			break;
		case 0x7: // 0x8XY7: Set VX to VY - VX, accounting for carry
			state->V[0xF] = state->V[y] > state->V[x];
			state->V[x] = state->V[y] - state->V[x];
			break;
		case 0xE: // 0x8XYE: Store most significant bit of VX in VF and shift VX left by 1
			state->V[0xF] = (state->V[x] >> 7) & 0x1;
			state->V[x] <<= 1;
			break;

		default:
			exit_unknown_instruction(instruction, state->program_counter);
		}
		break;
	case 0xA: // 0xANNN: Set index register to NNN
		state->index_register = nnn;
		break;
	case 0xB: // 0xBNNN: Jump to VX + NNN
		state->program_counter = state->V[x] + nnn;
		break;
	case 0xC: // 0xCXNN: Set VX to a random number AND'ed with NN
		state->V[x] = rand() & nn;
		break;
	case 0xD: ;// 0xDXYN: Display
		/* Display n-byte sprite starting at memory location I
		 * at (Vx, Vy), set VF = collision. */

		uint8_t dx = state->V[x] % DISPLAY_WIDTH;
		uint8_t dy = state->V[y] % DISPLAY_HEIGHT;

		// Reset collision flag
		state->V[0xF] = 0;

		// Read n bytes from memory. j is the y value
		for (uint8_t j = 0; j < n && dy + j < DISPLAY_HEIGHT; j++) {
			uint8_t sprite_row = state->mem[state->index_register + j];

			// i is the x value we use to iterate over bits
			for (uint8_t i = 0; i < 8 && dx + i < DISPLAY_WIDTH; i++) {
				// Bit shift to get the current row bit
				uint8_t sprite_bit = (sprite_row >> (7 - i)) & 1;

				if (state->display[dx+i][dy+j] == 1 && sprite_bit == 1) {
					// Set collision register
					state->V[0xF] = 1;
				}

				// XOR with the current bit
				state->display[dx+i][dy+j] ^= sprite_bit;
			}
		}
		break;
	case 0xE:
		switch (nn) {
		case 0x9E: // 0xEX9E: skip instruction if key VX is being pressed
			if (state->keys_pressed[state->V[x]] == 1) {
				state->program_counter += 2;
			}
			break;
		case 0xA1: // 0xEXA1: skip instruction if key VX is not being pressed
			if (state->keys_pressed[state->V[x]] == 0) {
				state->program_counter += 2;
			}
			break;
		default:
			exit_unknown_instruction(instruction, state->program_counter);
		}
		break;

	case 0xF:
		switch (nn) {
		case 0x07: // 0xFX07: set VX to the current value of the delay timer
			state->V[x] = state->delay_timer;
			break;
		case 0x15: // 0xFX15: set the delay timer to the value in VX
			state->delay_timer = state->V[x];
			break;
		case 0x18: // 0xFX18: set the sound timer to the value in VX
			state->sound_timer = state->V[x];
			break;
		case 0x1E: // 0xFX1E: Add VX to I
			// Overflow behavior is non-standard, but assumed safe
			state->V[0xF] = (state->V[x] + state->index_register) < state->V[x];
			state->index_register += state->V[x];
			break;
		case 0x0A: ;// 0xFX0A: Block until any key is pressed, put key in VX
			// Decrement program counter to repeat this
			// instruction in case a key isn't pressed
			state->program_counter -= 2;

			for (int i = 0; i <= 0xF; i++) {
				if (state->keys_pressed[i] == 1) {
					state->V[x] = i;
					state->program_counter += 2;
					break;
				}
			}
			break;
		case 0x29: ;// 0xFX29: Set I to font character in VX
			// Fonts are 5 bytes wide
			state->index_register = FONT_MEMORY_START + state->V[x] * 5;
			break;
		case 0x33: ;// 0xFX33: Store 3 decimal digits of VX in I, I+1, I+2
			state->mem[state->index_register]     = (state->V[x] % 1000) / 100;
			state->mem[state->index_register + 1] = (state->V[x] % 100) / 10;
			state->mem[state->index_register + 2] = (state->V[x] % 10);
			break;
		case 0x55: // 0xFX55: Store all registers from V0 to VX in I, I+1, I+2, ... I+X
			for (int i = 0; i <= x; i++) {
				state->mem[state->index_register + i] = state->V[i];
			}
			break;
		case 0x65: // 0xFX65: Store all memory from I, I+1, I+2, ... I+X in registers V0 to VX
			for (int i = 0; i <= x; i++) {
				state->V[i] = state->mem[state->index_register + i];
			}
			break;
		default:
			exit_unknown_instruction(instruction, state->program_counter);
		}
		break;
	default:
		exit_unknown_instruction(instruction, state->program_counter);
	}
}

int main(int argc, char *argv[])
{
	if (argc != 2) {
		fprintf(stderr, "Usage: %s <rom-file>\n", argv[0]);
		return 1;
	}

	chip8_state state;
	init_state(&state);

	chip8_screen screen;
	init_screen(&screen);

	// Load ROM into memory
	load_rom(argv[1], state.mem);

	while (true) {
		process_sdl_events(screen.window, &state);

		processor_cycle(&state);

		// TODO: Only draw display when display is updated
		// (set a bit on instructions in processor_cycle that
		// update the screen)
		draw_display(&screen, state.display);

		// TODO: Perform more accurate clock speed emulation
		// by using clock_gettime(CLOCK_MONOTONIC, ...),
		// recording the nanosecond time of the last
		// instruction, and trying to sleep until the next
		// instruction execution time.
		usleep(1000000 / PROCESSOR_SPEED_HZ);
	}

	return 0;
}
