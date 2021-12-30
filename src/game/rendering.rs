extern crate clipboard;

use std::{time::Instant, collections::HashMap, rc::Rc};
use clipboard::{ClipboardContext, ClipboardProvider};
use image::{imageops::{rotate90, rotate180, rotate270}, ImageBuffer, Rgba};
use speedy2d::{window::{WindowHandler, WindowHelper, VirtualKeyCode, KeyScancode, MouseButton}, Graphics2D, color::Color, image::{ImageDataType, ImageFileFormat, ImageSmoothingMode, ImageHandle}, dimen::Vector2, shape::Rectangle, font::{Font, TextLayout, TextOptions, FormattedTextBlock, TextAlignment}};

use crate::game::{cells::{DEFAULT_GRID_HEIGHT, DEFAULT_GRID_WIDTH, grid, CellType, Cell, initial}, direction::Direction, update::update, codes::{import, export}, cell_data::{ROTATOR_CW, ROTATOR_CCW, ORIENTATOR, TRASH, PULLSHER, MOVER, GENERATOR, WALL, PUSH, SLIDE, ENEMY, PULLER, MIRROR, HOTBAR_CELLS}};

pub static mut screen_x: f32 = DEFAULT_GRID_WIDTH as f32 / 2.0;
pub static mut screen_y: f32 = DEFAULT_GRID_HEIGHT as f32 / 2.0;
pub static mut screen_zoom: f32 = 1.0;

pub static mut SCREEN_WIDTH: f32 = 800.0;
pub static mut SCREEN_HEIGHT: f32 = 600.0;

pub const CELL_SIZE: f32 = 40.0;
const CELL_SPEED: f32 = 10.0;
const HOTBAR_HEIGHT: f32 = 80.0;
const HOTBAR_CELL_SIZE: f32 = 50.0;

macro_rules! collection {
    ($($k:expr => $v:expr),* $(,)?) => {{
        HashMap::from_iter([$(($k, $v),)*].into_iter())
    }};
}

pub struct WinHandler {
    assets: Option<Assets>,
    prev_time: Instant,
    keys: HashMap<VirtualKeyCode, bool>,
    mouse: Option<MouseButton>,
    mouse_pos: Vector2<f32>,

    help_text: Option<Rc<FormattedTextBlock>>,

    active_item: usize,
    direction: Direction,

    running: bool,
    show_help: bool,
    tick_times: [f32; 10],
    is_initial: bool,
}

impl WinHandler {
    #[inline(always)]
    pub fn new() -> Self {
        WinHandler {
            assets: None,
            prev_time: Instant::now(),
            keys: HashMap::new(),
            mouse: None,
            mouse_pos: Vector2::new(0.0, 0.0),

            help_text: None,

            active_item: 0,
            direction: Direction::Right,

            running: false,
            show_help: true,
            tick_times: [0.0; 10],
            is_initial: true,
        }
    }
}

impl WindowHandler for WinHandler {
    fn on_start(&mut self, _: &mut WindowHelper<()>, info: speedy2d::window::WindowStartupInfo) {
        unsafe {
            let pixels = info.viewport_size_pixels();
            // let scale = info.scale_factor();
            SCREEN_WIDTH = pixels.x as f32;
            SCREEN_HEIGHT = pixels.y as f32;
            grid.init();
        }
    }

	fn on_draw(&mut self, helper: &mut WindowHelper, g: &mut Graphics2D) {
        // setup and helper stuff
        if self.assets.is_none() {
            macro_rules! img {
                ($path:expr) => {
                    g.create_image_from_file_path(
                        Some(ImageFileFormat::PNG),
                        ImageSmoothingMode::NearestNeighbor,
                        $path
                    ).unwrap()
                }
            }
            macro_rules! cell_img {
                ($path:literal $amount:literal) => {{
                    let [tex0, tex1, tex2, tex3] = create_rotated_textures($amount, concat!("assets/cells/", $path, ".png"));
                    [
                        g.create_image_from_raw_pixels(ImageDataType::RGBA, ImageSmoothingMode::NearestNeighbor, Vector2::new(tex0.width(), tex0.height()), &tex0.into_raw()).unwrap(),
                        g.create_image_from_raw_pixels(ImageDataType::RGBA, ImageSmoothingMode::NearestNeighbor, Vector2::new(tex1.width(), tex1.height()), &tex1.into_raw()).unwrap(),
                        g.create_image_from_raw_pixels(ImageDataType::RGBA, ImageSmoothingMode::NearestNeighbor, Vector2::new(tex2.width(), tex2.height()), &tex2.into_raw()).unwrap(),
                        g.create_image_from_raw_pixels(ImageDataType::RGBA, ImageSmoothingMode::NearestNeighbor, Vector2::new(tex3.width(), tex3.height()), &tex3.into_raw()).unwrap(),
                    ]
                }}
            }

            let font = Font::new(include_bytes!("../../assets/font.ttf")).unwrap();

            unsafe {
                self.help_text = Some(font.layout_text(
                    "WASD to move, R+F to zoom, Left click to place, Right click to delete, I+O to import/export, Space to start, G to step, T to reset, Press ESC to hide this message",
                    25.0,
                    TextOptions::new()
                        .with_wrap_to_width(SCREEN_WIDTH, TextAlignment::Center)
                ));
            }

            let assets = Assets {
                cell_bg: img!("assets/background.png"),
                cells: collection![
                    WALL => cell_img!("wall" 1),
                    MOVER => cell_img!("mover" 4),
                    PULLER => cell_img!("puller" 4),
                    PULLSHER => cell_img!("pullsher" 4),
                    GENERATOR => cell_img!("generator" 4),
                    ROTATOR_CW => cell_img!("rotator_cw" 1),
                    ROTATOR_CCW => cell_img!("rotator_ccw" 1),
                    ORIENTATOR => cell_img!("orientator" 4),
                    PUSH => cell_img!("push" 1),
                    SLIDE => cell_img!("slide" 2),
                    TRASH => cell_img!("trash" 1),
                    ENEMY => cell_img!("enemy" 1),
                    MIRROR => cell_img!("mirror" 2),
                ],
                font,
            };

            self.assets = Some(assets);
        }
        let delta = self.prev_time.elapsed();
        let delta_secs = delta.as_secs_f32();
        self.prev_time = Instant::now();

        // draw stuff

        let assets = self.assets.as_ref().unwrap();
		g.clear_screen(Color::from_hex_rgb(0x000000));

        if self.running {
            let start_time = Instant::now();
            unsafe { do_tick(&mut self.is_initial); }
            self.tick_times.rotate_left(1);
            self.tick_times[9] = start_time.elapsed().as_secs_f32() / 1000.0;
        }

        unsafe {
        // grid
            if *self.keys.get(&VirtualKeyCode::W).unwrap_or(&false) { screen_y += delta_secs * CELL_SPEED / screen_zoom; }
            if *self.keys.get(&VirtualKeyCode::S).unwrap_or(&false) { screen_y -= delta_secs * CELL_SPEED / screen_zoom; }
            if *self.keys.get(&VirtualKeyCode::A).unwrap_or(&false) { screen_x -= delta_secs * CELL_SPEED / screen_zoom; }
            if *self.keys.get(&VirtualKeyCode::D).unwrap_or(&false) { screen_x += delta_secs * CELL_SPEED / screen_zoom; }

            draw_grid(assets, g);

        // hotbar
            let hotbar_rect = Rectangle::new(
                Vector2::new(0.0, SCREEN_HEIGHT as f32 - HOTBAR_HEIGHT),
                Vector2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32),
            );

            // background
            g.draw_rectangle(
                hotbar_rect.clone(),
                Color::from_hex_argb(0x99aaaaaa),
            );

            // cells
            #[allow(clippy::needless_range_loop)]
            for i in 0..HOTBAR_CELLS.len() {
                let cell_type = HOTBAR_CELLS[i];
                let cell_img = &assets.cells.get(&cell_type).unwrap()[usize::from(self.direction)];
                let rect = Rectangle::new(
                    Vector2::new(
                        (i as f32 * HOTBAR_CELL_SIZE * 1.5) + (HOTBAR_CELL_SIZE / 2.0),
                        SCREEN_HEIGHT as f32 - HOTBAR_HEIGHT + (HOTBAR_HEIGHT - HOTBAR_CELL_SIZE) / 2.0,
                    ),
                    Vector2::new(
                        (i as f32 * HOTBAR_CELL_SIZE * 1.5) + (HOTBAR_CELL_SIZE / 2.0) + HOTBAR_CELL_SIZE,
                        SCREEN_HEIGHT as f32 - HOTBAR_HEIGHT + (HOTBAR_HEIGHT + HOTBAR_CELL_SIZE) / 2.0,
                    ),
                );
                g.draw_rectangle_image(
                    rect.clone(),
                    cell_img,
                );
                if is_inside(rect, self.mouse_pos) {
                    if let Some(MouseButton::Left) = self.mouse {
                        self.active_item = i;
                    }
                }
            }

            // make cells lighter
            g.draw_rectangle(
                hotbar_rect.clone(),
                Color::from_hex_argb(0x88aaaaaa),
            );

            // active item
            let cell_img = &assets.cells.get(&HOTBAR_CELLS[self.active_item]).unwrap()[usize::from(self.direction)];
            g.draw_rectangle_image(
                Rectangle::new(
                    Vector2::new(
                        (self.active_item as f32 * HOTBAR_CELL_SIZE * 1.5) + (HOTBAR_CELL_SIZE / 2.0),
                        SCREEN_HEIGHT as f32 - HOTBAR_HEIGHT + (HOTBAR_HEIGHT - HOTBAR_CELL_SIZE) / 2.0,
                    ),
                    Vector2::new(
                        (self.active_item as f32 * HOTBAR_CELL_SIZE * 1.5) + (HOTBAR_CELL_SIZE / 2.0) + HOTBAR_CELL_SIZE,
                        SCREEN_HEIGHT as f32 - HOTBAR_HEIGHT + (HOTBAR_HEIGHT + HOTBAR_CELL_SIZE) / 2.0,
                    ),
                ),
                cell_img,
            );

            // top border
            g.draw_line(
                Vector2::new(0.0, SCREEN_HEIGHT as f32 - HOTBAR_HEIGHT),
                Vector2::new(SCREEN_WIDTH, SCREEN_HEIGHT as f32 - HOTBAR_HEIGHT),
                2.0,
                Color::DARK_GRAY,
            );

        // placing
            if !is_inside(hotbar_rect, self.mouse_pos) {
                let screen_w_half = SCREEN_WIDTH / 2.0;
                let screen_h_half = SCREEN_HEIGHT / 2.0;
                let x = (self.mouse_pos.x - screen_w_half) / CELL_SIZE / screen_zoom + screen_x;
                let y = screen_y - (self.mouse_pos.y - screen_h_half) / CELL_SIZE / screen_zoom;
                let cell = Cell::new(HOTBAR_CELLS[self.active_item], self.direction);
                if let Some(MouseButton::Left) = self.mouse {
                    grid.set(x.floor() as isize, y.floor() as isize, cell);
                }
                else if let Some(MouseButton::Right) = self.mouse {
                    grid.delete(x.floor() as isize, y.floor() as isize);
                }
            }
        }

        // help
        if self.show_help {
            unsafe {
                let help_rect = Rectangle::new(
                    Vector2::new(0.0, 0.0),
                    Vector2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32),
                );
                g.draw_rectangle(
                    help_rect,
                    Color::from_hex_argb(0xee444444),
                );
                g.draw_text(
                    Vector2::new(0.0, SCREEN_HEIGHT as f32 / 2.0),
                    Color::WHITE,
                    self.help_text.as_ref().unwrap(),
                );
            }
        }

        // fps
        g.draw_text(
            Vector2::new(10.0, 10.0),
            Color::WHITE,
            &assets.font.layout_text(&format!("FPS: {}", 1.0/delta_secs), 17.0, TextOptions::new()),
        );

        // tick time
        g.draw_text(
            Vector2::new(10.0, 30.0),
            Color::WHITE,
            &assets.font.layout_text(&format!("Tick time: {}", self.tick_times.iter().sum::<f32>() / 10.0), 17.0, TextOptions::new()),
        );

        helper.request_redraw();
	}

    fn on_key_down(&mut self, _: &mut WindowHelper<()>, virtual_key_code: Option<VirtualKeyCode>, _: KeyScancode) {
        if let Some(key) = virtual_key_code {
            self.keys.insert(key, true);
            match key {
                VirtualKeyCode::Escape => self.show_help = !self.show_help,

                VirtualKeyCode::Space => { self.running = !self.running; },
                VirtualKeyCode::G => { self.running = false; unsafe { do_tick(&mut self.is_initial); } },
                VirtualKeyCode::T => { if !self.is_initial { unsafe { self.running = false; grid = initial.clone(); } } },

                VirtualKeyCode::Q => self.direction -= 1,
                VirtualKeyCode::E => self.direction += 1,

                VirtualKeyCode::R => unsafe { screen_zoom /= 1.2 },
                VirtualKeyCode::F => unsafe { screen_zoom *= 1.2 },

                VirtualKeyCode::I => {
                    let mut clip: ClipboardContext = ClipboardProvider::new().unwrap();
                    let text = clip.get_contents().unwrap();
                    let _ = import(text.as_str());
                    unsafe {
                        screen_x = grid.width as f32 / 2.0;
                        screen_y = grid.height as f32 / 2.0;
                        screen_zoom = 1.0;
                    }
                },
                VirtualKeyCode::O => {
                    let mut clip: ClipboardContext = ClipboardProvider::new().unwrap();
                    let text = export();
                    clip.set_contents(text).unwrap();
                },
                _ => {},
            }
        }
    }

    fn on_key_up(&mut self, _: &mut WindowHelper<()>, virtual_key_code: Option<VirtualKeyCode>, _: KeyScancode) {
        if let Some(key) = virtual_key_code {
            self.keys.remove(&key);
        }
    }

    fn on_mouse_button_down(&mut self, _: &mut WindowHelper<()>, button: MouseButton) {
        self.mouse = Some(button);
    }
    fn on_mouse_button_up(&mut self, _: &mut WindowHelper<()>, _: MouseButton) {
        self.mouse = None;
    }
    fn on_mouse_move(&mut self, _: &mut WindowHelper<()>, position: Vector2<f32>) {
        self.mouse_pos = position;
    }
}

unsafe fn do_tick(is_initial: &mut bool) {
    if *is_initial {
        *is_initial = false;
        initial = grid.clone();
    }
    update();
}

unsafe fn draw_grid(assets: &Assets, g: &mut Graphics2D) {

    // calculate visible cells
    let screen_w_half = SCREEN_WIDTH / 2.0;
    let screen_h_half = SCREEN_HEIGHT / 2.0;
    let sx = (-screen_w_half) / CELL_SIZE / screen_zoom + screen_x;
    let sy = screen_y - screen_h_half / CELL_SIZE / screen_zoom;
    let ex = screen_w_half / CELL_SIZE / screen_zoom + screen_x;
    let ey = screen_y - (-screen_h_half) / CELL_SIZE / screen_zoom;

    let sx = (sx.floor() as isize).max(0).min(grid.width as isize);
    let sy = (sy.floor() as isize).max(0).min(grid.height as isize);
    let ex = (ex.ceil() as isize).max(0).min(grid.width as isize);
    let ey = (ey.ceil() as isize).max(0).min(grid.height as isize);

    for y in sy..ey {
        for x in sx..ex {
            let cell_rect = Rectangle::new(
                Vector2::new(
                    (x as f32 - screen_x) * CELL_SIZE * screen_zoom + screen_w_half,
                    (screen_y - y as f32 - 1.0) * CELL_SIZE * screen_zoom + screen_h_half,
                ),
                Vector2::new(
                    (x as f32 - screen_x + 1.0) * CELL_SIZE * screen_zoom + screen_w_half,
                    (screen_y - y as f32) * CELL_SIZE * screen_zoom + screen_h_half,
                )
            );

            // draw background
            g.draw_rectangle_image(cell_rect.clone(), &assets.cell_bg);

            if let Some(cell) = grid.get(x as isize, y as isize) {
                // draw cell
                g.draw_rectangle_image(cell_rect.clone(), &assets.cells.get(&cell.id).unwrap()[usize::from(cell.direction)]);
            }
        }
    }
}

struct Assets {
    cell_bg: ImageHandle,
    cells: HashMap<CellType, [ImageHandle; 4]>,

    font: Font,
}

fn create_rotated_textures(amount: usize, path: &str) -> [ImageBuffer<Rgba<u8>, Vec<u8>>; 4] {
    let first_texture = image::open(path).unwrap().to_rgba8();
    let mut textures = [first_texture.clone(), first_texture.clone(), first_texture.clone(), first_texture];
    for (i, img) in textures.iter_mut().enumerate() {
        match i % amount {
            0 => {},
            1 => *img = rotate90(img),
            2 => *img = rotate180(img),
            3 => *img = rotate270(img),
            _ => unreachable!(),
        }
    }
    textures
}

fn is_inside<T: PartialOrd>(rect: Rectangle<T>, point: Vector2<T>) -> bool {
    rect.top_left().x <= point.x && rect.top_left().y <= point.y &&
        rect.bottom_right().x >= point.x && rect.bottom_right().y >= point.y
}
