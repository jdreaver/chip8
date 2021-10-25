#include <SDL2/SDL.h>

#include <stdbool.h>
#include <unistd.h>

#define PIXEL_SCALE_FACTOR 8

int main() {
	SDL_Init(SDL_INIT_VIDEO);

	int window_width = 64 * PIXEL_SCALE_FACTOR;
	int window_height = 32 * PIXEL_SCALE_FACTOR;
	SDL_Window *window = SDL_CreateWindow("CHIP-8", SDL_WINDOWPOS_CENTERED, SDL_WINDOWPOS_CENTERED, window_width, window_height, 0);
	SDL_Renderer *renderer = SDL_CreateRenderer(window, -1, SDL_RENDERER_ACCELERATED);

	// Clear screen to black
	SDL_SetRenderDrawColor(renderer, 0, 0, 0, 255);
	SDL_RenderClear(renderer);

	// Set drawing color to white
	SDL_SetRenderDrawColor(renderer, 255, 255, 255, 255);

	// Draw a white rectangle
	SDL_Rect rect = {
		.x = 32 * PIXEL_SCALE_FACTOR,
		.y = 16 * PIXEL_SCALE_FACTOR,
		.w = 4 * PIXEL_SCALE_FACTOR,
		.h = 4 * PIXEL_SCALE_FACTOR,
	};
	SDL_RenderFillRect(renderer, &rect);

	SDL_RenderPresent(renderer);

	while (true) {
		SDL_Event event;

		// check for event
		while (SDL_PollEvent(&event)) {
			// get snapshot of current state of the keyboard
			const Uint8* state = SDL_GetKeyboardState(NULL);

			switch (event.type) {
			case SDL_QUIT:
				SDL_DestroyWindow(window);
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
