pub(crate) const DISPLAY_WIDTH_PX: usize = 64;
pub(crate) const DISPLAY_HEIGHT_PX: usize = 32;
pub(crate) const PIXEL_SCALE_FACTOR: usize = 8;

pub(crate) struct Display {
    pixels: [[bool; DISPLAY_HEIGHT_PX]; DISPLAY_WIDTH_PX],
    canvas: sdl2::render::Canvas<sdl2::video::Window>,

    /// Set to `true` when the display is modified and we need to
    /// repaint the canvas.
    needs_repaint: bool,
}

impl Display {
    pub(crate) fn new() -> Display {
        Display {
            pixels: [[false; DISPLAY_HEIGHT_PX]; DISPLAY_WIDTH_PX],
            needs_repaint: false,
            canvas: create_sdl_window(),
        }
    }

    pub(crate) fn clear(&mut self) {
        for i in 0..DISPLAY_WIDTH_PX {
            for j in 0..DISPLAY_HEIGHT_PX {
                self.pixels[i][j] = false;
            }
        }
        self.needs_repaint = true;
    }

    pub(crate) fn get_pixel(&self, x: usize, y: usize) -> bool {
        self.pixels[x][y]
    }

    pub(crate) fn set_pixel(&mut self, x: usize, y: usize, val: bool) {
        self.pixels[x][y] = val;
        self.needs_repaint = true;
    }

    pub(crate) fn paint(&mut self) {
        if !self.needs_repaint {
            return;
        }

        self.canvas
            .set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        self.canvas.clear();

        self.canvas
            .set_draw_color(sdl2::pixels::Color::RGB(255, 255, 255)); // White

        for i in 0..DISPLAY_WIDTH_PX {
            for j in 0..DISPLAY_HEIGHT_PX {
                if self.pixels[i][j] {
                    let rect = sdl2::rect::Rect::new(
                        (i * PIXEL_SCALE_FACTOR) as i32, // x
                        (j * PIXEL_SCALE_FACTOR) as i32, // y
                        PIXEL_SCALE_FACTOR as u32,       // width
                        PIXEL_SCALE_FACTOR as u32,       // height
                    );
                    if let Err(err) = self.canvas.fill_rect(rect) {
                        eprintln!("Error drawing rectangle {:?}: {}", rect, err);
                        std::process::exit(1);
                    }
                }
            }
        }

        self.canvas.present();
        self.needs_repaint = false;
    }
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
    window
        .into_canvas()
        .build()
        .expect("failed to create SDL canvas")
}
