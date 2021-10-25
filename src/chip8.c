#include <SDL2/SDL.h>

#include <stdbool.h>
#include <string.h>
#include <unistd.h>

#define PIXEL_SCALE_FACTOR 8

#define DISPLAY_HEIGHT 64
#define DISPLAY_WIDTH 32

#define MAX_STACK_SIZE 100

typedef struct {
	uint8_t mem[4096];
	bool display[DISPLAY_HEIGHT][DISPLAY_WIDTH];
	uint8_t program_counter;
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
	memset(state->display, 0, sizeof(state->display));
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

int main()
{
	chip8_state state;
	init_state(&state);

	chip8_screen screen;
	init_screen(&screen);

	// Draw simple rectangle for testing
	state.display[32][16] = 1;
	draw_display(&screen, state.display);

	while (true) {
		SDL_Event event;

		// check for event
		while (SDL_PollEvent(&event)) {
			// get snapshot of current state of the keyboard
			const Uint8* state = SDL_GetKeyboardState(NULL);

			switch (event.type) {
			case SDL_QUIT:
				SDL_DestroyWindow(screen.window);
				SDL_Quit();
				return 0;
			default:
				break;
			}
		}
		usleep(1000);
	}

	return 0;
}
