#include <SDL2/SDL.h>

#include <stdbool.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

#define PIXEL_SCALE_FACTOR 8

#define DISPLAY_HEIGHT 64
#define DISPLAY_WIDTH 32

#define MAX_STACK_SIZE 100

#define PROCESSOR_SPEED_HZ 700

typedef struct {
	uint8_t mem[4096];
	bool display[DISPLAY_HEIGHT][DISPLAY_WIDTH];
	uint16_t program_counter;
	uint16_t index_register;
	uint16_t stack[MAX_STACK_SIZE];

	/* Timers decremented at 60 Hz */
	uint8_t delay_timer;
	uint8_t sound_timer;

	/* General purpose registers */
	uint8_t V[16];
} chip8_state;

void init_state(chip8_state *state)
{
	memset(state->mem, 0, sizeof(state->mem));
	state->program_counter = 0x200;
	memset(state->display, 0, sizeof(state->display));
	memset(state->V, 0, sizeof(state->V));
}

typedef struct {
	SDL_Window *window;
	SDL_Renderer *renderer;
} chip8_screen;

void init_screen(chip8_screen *screen)
{
	SDL_Init(SDL_INIT_VIDEO);

	int window_width = DISPLAY_HEIGHT * PIXEL_SCALE_FACTOR;
	int window_height = DISPLAY_WIDTH * PIXEL_SCALE_FACTOR;
	screen->window = SDL_CreateWindow("CHIP-8", SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED, window_width, window_height, 0);
	screen->renderer = SDL_CreateRenderer(screen->window, -1, SDL_RENDERER_ACCELERATED);
}

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

void draw_display(chip8_screen *screen, bool display[DISPLAY_HEIGHT][DISPLAY_WIDTH])
{
	// Clear screen to black
	SDL_SetRenderDrawColor(screen->renderer, 0, 0, 0, 255);
	SDL_RenderClear(screen->renderer);

	// Set drawing color to white
	SDL_SetRenderDrawColor(screen->renderer, 255, 255, 255, 255);

	for (int i = 0; i < DISPLAY_HEIGHT; i++) {
		for (int j = 0; j < DISPLAY_WIDTH; j++) {
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
	size_t bytes_read = fread(mem + 0x200, 1, sizeof(uint8_t*) * 4096 - 0x200, fp);

	if (bytes_read != fsize) {
		fprintf(stderr, "failed loading ROM into memory. %ld != %ld\n", bytes_read, fsize);
		exit(EXIT_FAILURE);
	}

	fclose(fp);

        // Load font into 0x050–0x09F
	memcpy(mem + 0x050, font, sizeof(font));
}

void process_sdl_events(SDL_Window *window)
{
	SDL_Event event;
	while (SDL_PollEvent(&event)) {
		switch (event.type) {
		case SDL_QUIT:
			SDL_DestroyWindow(window);
			SDL_Quit();
			exit(EXIT_SUCCESS);
		default:
			break;
		}
	}
}

void processor_cycle(chip8_state *state)
{
	// Instructions are 2 bytes
	uint16_t instruction = state->mem[state->program_counter] << 8 | state->mem[state->program_counter + 1];
	state->program_counter += 2;

	// First byte of instruction stores op code
	switch (instruction & 0xF000) {
	case 0x0000:
		switch (instruction & 0x0FFF) {
		case 0x0E0: // Clear screen
			for (int i = 0; i < DISPLAY_HEIGHT; i++) {
				for (int j = 0; j < DISPLAY_WIDTH; j++) {
					state->display[i][j] = 0;
				}
			}
			// TODO: Set some bit here that says display was touched
			break;

		case 0x00E: // Return from subroutine
			break;
		}

		// TODO: Log unknown instruction
		break;
        case 0x1000: // Jump (0x1NNN) NNN is the new program counter
		state->program_counter = instruction & 0x0FFF;
		break;
	case 0x6000: // 0x6XNN: Set register VX to NN
		state->V[(uint8_t) (instruction & 0x0F00)] = (uint8_t) (instruction & 0x00FF);
		break;
	case 0x7000: // 0x7XNN: Add NN to register VX
		state->V[(uint8_t) (instruction & 0x0F00)] += (uint8_t) (instruction & 0x00FF);
		break;
	case 0xA000: // 0xANNN: Set index register to NNN
		state->index_register = instruction & 0x0FFF;
		break;
	case 0xD000: // 0xDXYN: Display
		/* TODO */
		printf("Would display: 0x%x (PC: 0x%x)\n", instruction, state->program_counter);
		break;
	default:
		printf("Unknown instruction: 0x%x (PC: 0x%x)\n", instruction, state->program_counter);
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

	// Draw simple rectangle for testing
	state.display[32][16] = 1;
	draw_display(&screen, state.display);

	while (true) {
		process_sdl_events(screen.window);

		processor_cycle(&state);

		// TODO: Perform more accurate clock speed emulation
		// by using clock_gettime(CLOCK_MONOTONIC, ...),
		// recording the nanosecond time of the last
		// instruction, and trying to sleep until the next
		// instruction execution time.
		usleep(1000000 / PROCESSOR_SPEED_HZ);
	}

	return 0;
}
