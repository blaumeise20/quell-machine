extern crate clipboard;

use std::{time::Instant, collections::{HashMap, HashSet}, rc::Rc, path::PathBuf};
use clipboard::{ClipboardContext, ClipboardProvider};
use image::{imageops::{rotate90, rotate180, rotate270}, ImageBuffer, Rgba};
use speedy2d::{window::{WindowHandler, WindowHelper, VirtualKeyCode, KeyScancode, MouseButton, MouseScrollDistance}, Graphics2D, color::Color, image::{ImageDataType, ImageFileFormat, ImageSmoothingMode, ImageHandle}, dimen::Vector2, shape::Rectangle, font::{Font, TextLayout, TextOptions, FormattedTextBlock, TextAlignment}};

use crate::game::{cells::{DEFAULT_GRID_HEIGHT, DEFAULT_GRID_WIDTH, CellType, Cell, Grid}, direction::Direction, update::{update, run_update_loop}, codes::{import, export_q1, export_q2}, cell_data::{CELL_DATA, HOTBAR_ITEMS, MAILBOX}};

use super::update::UpdateState;

pub static mut grid: Grid = Grid::new_const(DEFAULT_GRID_WIDTH, DEFAULT_GRID_HEIGHT);
pub static mut initial: Grid = Grid::new_const(DEFAULT_GRID_WIDTH, DEFAULT_GRID_HEIGHT);

pub static mut screen_x: f32 = DEFAULT_GRID_WIDTH as f32 / 2.0;
pub static mut screen_y: f32 = DEFAULT_GRID_HEIGHT as f32 / 2.0;
pub static mut screen_zoom: f32 = 1.0;

pub static mut SCREEN_WIDTH: f32 = 800.0;
pub static mut SCREEN_HEIGHT: f32 = 600.0;

const CELL_SIZE: f32 = 40.0;
const CELL_SPEED: f32 = 10.0;

const HOTBAR_HEIGHT: f32 = 90.0;
const HOTBAR_CELL_SIZE: f32 = HOTBAR_HEIGHT * 0.6;
const HOTBAR_CELL_SPACING: f32 = (HOTBAR_HEIGHT - HOTBAR_CELL_SIZE) / 2.0;

const TOOLTIP_WIDTH: f32 = 400.0;
const TOOLTIP_HEIGHT: f32 = 200.0;
const TOOLTIP_PADDING: f32 = 20.0;

#[cfg(target_os = "macos")]
const COMMAND_KEY: VirtualKeyCode = VirtualKeyCode::LWin;
#[cfg(not(target_os = "macos"))]
const COMMAND_KEY: VirtualKeyCode = VirtualKeyCode::LControl;

type Text = Rc<FormattedTextBlock>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tool {
    Place,
    Rect(isize),
    Circle(isize),
}

pub struct WinHandler {
    resource_path: PathBuf,
    assets: Option<Assets>,
    prev_time: Instant,
    keys: HashSet<VirtualKeyCode>,
    mouse: Option<MouseButton>,
    mouse_pos: Vector2<f32>,
    undo_stack: CellUndoStack,

    help_text: Option<Text>,
    hotbar_item_text: Option<HashMap<CellType, Tooltip>>,

    active_item: usize,
    hotbar_state: Vec<usize>,
    open_item_menu: Option<usize>,
    direction: Direction,
    place: bool,
    placement_tool: Tool,

    running: bool,
    running_state: Option<UpdateState>,
    show_help: bool,
    tick_times: [f32; 10],
    is_initial: bool,
    threaded: bool,
}

impl WinHandler {
    #[inline(always)]
    pub fn new(resource_path: PathBuf) -> Self {
        WinHandler {
            resource_path,
            assets: None,
            prev_time: Instant::now(),
            keys: HashSet::new(),
            mouse: None,
            mouse_pos: Vector2::new(0.0, 0.0),
            undo_stack: CellUndoStack::new(),

            help_text: None,
            hotbar_item_text: None,

            active_item: 0,
            hotbar_state: vec![0; HOTBAR_ITEMS.len()],
            open_item_menu: None,
            direction: Direction::Right,
            place: true,
            placement_tool: Tool::Place,

            running: false,
            running_state: None,
            show_help: true,
            tick_times: [0.0; 10],
            is_initial: true,
            threaded: false,
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

    fn on_resize(&mut self, _: &mut WindowHelper<()>, size: Vector2<u32>) {
        unsafe {
            SCREEN_WIDTH = size.x as f32;
            SCREEN_HEIGHT = size.y as f32;
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
                        self.resource_path.join($path)
                    ).unwrap()
                }
            }

            let font = Font::new(include_bytes!("../../assets/font.ttf")).unwrap();

            unsafe {
                self.help_text = Some(font.layout_text(
                    "WASD to move\nR+F to zoom\nLeft click to place\nRight click to delete\nAlt+R/F to change cursor size\nI+O to import/export\nSpace to start\nG to step\nT to reset\n\nPress ESC to hide this message",
                    25.0,
                    TextOptions::new()
                        .with_wrap_to_width(SCREEN_WIDTH, TextAlignment::Center)
                ));

                // self.hotbar_item_text = Some(HOTBAR_ITEMS.iter().flat_map(|a| {
                //     a.iter().map(|cell_type| {
                //         (cell_type.id, (
                //             font.layout_text(
                //                 cell_type.name,
                //                 HOTBAR_CELL_SIZE / 1.5,
                //                 TextOptions::new()
                //                     .with_wrap_to_width(TOOLTIP_WIDTH - TOOLTIP_PADDING * 2.0, TextAlignment::Left)
                //             ),
                //             font.layout_text(
                //                 cell_type.description,
                //                 HOTBAR_CELL_SIZE / 2.0,
                //                 TextOptions::new()
                //                     .with_wrap_to_width(TOOLTIP_WIDTH - TOOLTIP_PADDING * 2.0, TextAlignment::Left)
                //             ),
                //         ))
                //     })
                // }).collect());
                self.hotbar_item_text = Some(HOTBAR_ITEMS.iter().flat_map(|a| {
                    a.iter().map(|cell_type| {
                        (cell_type.id, Tooltip::new(
                            &font,
                            cell_type.name,
                            Some(format!("ID: {}", cell_type.id)),
                            Some(cell_type.description),
                        ))
                    })
                }).collect());
            }

            let assets = Assets {
                cell_bg: img!("assets/background.png"),
                cells: {
                    let mut map = HashMap::new();
                    for cell in CELL_DATA {
                        let [
                            tex0,
                            tex1,
                            tex2,
                            tex3,
                        ] = create_rotated_textures(cell.sides, self.resource_path.join("assets/cells/".to_string() + cell.texture_name + ".png"));
                        map.insert(cell.id, [
                            g.create_image_from_raw_pixels(ImageDataType::RGBA, ImageSmoothingMode::NearestNeighbor, Vector2::new(tex0.width(), tex0.height()), &tex0.into_raw()).unwrap(),
                            g.create_image_from_raw_pixels(ImageDataType::RGBA, ImageSmoothingMode::NearestNeighbor, Vector2::new(tex1.width(), tex1.height()), &tex1.into_raw()).unwrap(),
                            g.create_image_from_raw_pixels(ImageDataType::RGBA, ImageSmoothingMode::NearestNeighbor, Vector2::new(tex2.width(), tex2.height()), &tex2.into_raw()).unwrap(),
                            g.create_image_from_raw_pixels(ImageDataType::RGBA, ImageSmoothingMode::NearestNeighbor, Vector2::new(tex3.width(), tex3.height()), &tex3.into_raw()).unwrap(),
                        ]);
                    }
                    map
                },

                tool_place: img!("assets/tool_place.png"),
                tool_rect: img!("assets/tool_rect.png"),
                tool_circle: img!("assets/tool_circle.png"),

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

        if self.running && self.running_state.is_none() {
            let start_time = Instant::now();
            unsafe { do_tick(&mut self.is_initial); }
            self.tick_times.rotate_left(1);
            self.tick_times[9] = start_time.elapsed().as_secs_f32() * 1000.0;
        }
        if let Some(state) = &self.running_state {
            unsafe {
                let state = state.lock().unwrap();
                grid = state.1.clone();
            }
        }

        unsafe {
            let hotbar_rect = Rectangle::new(
                Vector2::new(0.0, SCREEN_HEIGHT as f32 - HOTBAR_HEIGHT),
                Vector2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32),
            );
            if self.keys.contains(&VirtualKeyCode::W) { screen_y += delta_secs * CELL_SPEED / screen_zoom; }
            if self.keys.contains(&VirtualKeyCode::S) { screen_y -= delta_secs * CELL_SPEED / screen_zoom; }
            if self.keys.contains(&VirtualKeyCode::A) { screen_x -= delta_secs * CELL_SPEED / screen_zoom; }
            if self.keys.contains(&VirtualKeyCode::D) { screen_x += delta_secs * CELL_SPEED / screen_zoom; }

        // grid
            draw_grid(assets, g);

        // placing
            if self.place && !is_inside(hotbar_rect.clone(), self.mouse_pos) {
                let screen_w_half = SCREEN_WIDTH / 2.0;
                let screen_h_half = SCREEN_HEIGHT / 2.0;
                let x = (self.mouse_pos.x - screen_w_half) / CELL_SIZE / screen_zoom + screen_x;
                let y = screen_y - (self.mouse_pos.y - screen_h_half) / CELL_SIZE / screen_zoom;
                let cell = Cell::new(HOTBAR_ITEMS[self.active_item][self.hotbar_state[self.active_item]].id, self.direction);

                let x = x.floor() as isize;
                let y = y.floor() as isize;

                let dia = match self.placement_tool {
                    Tool::Place => 1,
                    Tool::Rect(d) => d,
                    Tool::Circle(d) => d,
                };
                let half_dia = dia / 2;
                let x = x - half_dia;
                let y = y - half_dia;

                let place_cell;
                let do_place;
                if let Some(MouseButton::Left) = self.mouse {
                    place_cell = Some(cell.clone());
                    do_place = true;
                }
                else if let Some(MouseButton::Right) = self.mouse {
                    place_cell = None;
                    do_place = true;
                }
                else {
                    place_cell = None;
                    do_place = false;
                }

                if let Tool::Circle(_) = self.placement_tool {
                    let real_half_dia = half_dia as f32 + 0.5;
                    let sqrad = real_half_dia * real_half_dia;
                    for oy in 0..dia {
                        for ox in 0..dia {
                            let x_dist = ox as f32 + 0.5 - real_half_dia;
                            let y_dist = oy as f32 + 0.5 - real_half_dia;
                            if x_dist * x_dist + y_dist * y_dist <= sqrad {
                                let x = x + ox;
                                let y = y + oy;
                                draw_ghost_cell(assets, g, x, y, &cell);
                                if do_place {
                                    let mut place_cell = place_cell.clone();
                                    let cell = grid.get_mut(x, y);
                                    if let Some(cell) = cell {
                                        if cell.id == MAILBOX {
                                            if let Some(ref mut place_cell) = place_cell {
                                                if place_cell.id != MAILBOX {
                                                    let contained = (place_cell.id, place_cell.direction - cell.direction);
                                                    *place_cell = cell.copy();
                                                    place_cell.contained_cell = Some(contained);
                                                }
                                            }
                                        }
                                    }
                                    if place_cell != *cell {
                                        self.undo_stack.insert(x, y, cell.clone());
                                        *cell = place_cell;
                                    }
                                }
                            }
                        }
                    }
                }
                else {
                    for oy in 0..dia {
                        for ox in 0..dia {
                            let x = x + ox;
                            let y = y + oy;
                            draw_ghost_cell(assets, g, x, y, &cell);
                            if do_place {
                                let mut place_cell = place_cell.clone();
                                let cell = grid.get_mut(x, y);
                                if let Some(cell) = cell {
                                    if cell.id == MAILBOX {
                                        if let Some(ref mut place_cell) = place_cell {
                                            if place_cell.id != MAILBOX {
                                                let contained = (place_cell.id, place_cell.direction - cell.direction);
                                                *place_cell = cell.copy();
                                                place_cell.contained_cell = Some(contained);
                                            }
                                        }
                                    }
                                }
                                if place_cell != *cell {
                                    self.undo_stack.insert(x, y, cell.clone());
                                    *cell = place_cell;
                                }
                            }
                        }
                    }
                }
            }

        // hotbar
            // background
            g.draw_rectangle(
                hotbar_rect,
                Color::from_hex_argb(0xcfaaaaaa),
            );

            // cells
            #[allow(clippy::needless_range_loop)]
            for i in 0..HOTBAR_ITEMS.len() {
                let item = HOTBAR_ITEMS[i];
                let active_cell = item[self.hotbar_state[i]];
                let cell_img = &assets.cells.get(&active_cell.id).unwrap()[usize::from(self.direction)];
                let rect = Rectangle::new(
                    Vector2::new(
                        i as f32 * (HOTBAR_CELL_SIZE + HOTBAR_CELL_SPACING) + HOTBAR_CELL_SPACING,
                        SCREEN_HEIGHT as f32 - HOTBAR_HEIGHT + HOTBAR_CELL_SPACING,
                    ),
                    Vector2::new(
                        i as f32 * (HOTBAR_CELL_SIZE + HOTBAR_CELL_SPACING) + HOTBAR_CELL_SIZE + HOTBAR_CELL_SPACING,
                        SCREEN_HEIGHT as f32 - HOTBAR_CELL_SPACING,
                    ),
                );
                g.draw_rectangle_image_tinted(
                    rect,
                    Color::from_hex_argb(if self.active_item == i { 0xffffffff } else { 0x70ffffff }),
                    cell_img,
                );
            }

            // active tool
            let tool_img = match self.placement_tool {
                Tool::Place => &assets.tool_place,
                Tool::Rect(_) => &assets.tool_rect,
                Tool::Circle(_) => &assets.tool_circle,
            };
            let tool_rect = Rectangle::new(
                Vector2::new(
                    SCREEN_WIDTH as f32 - HOTBAR_CELL_SIZE - HOTBAR_CELL_SPACING,
                    SCREEN_HEIGHT as f32 - HOTBAR_CELL_SIZE - HOTBAR_CELL_SPACING,
                ),
                Vector2::new(
                    SCREEN_WIDTH as f32 - HOTBAR_CELL_SPACING,
                    SCREEN_HEIGHT as f32 - HOTBAR_CELL_SPACING,
                ),
            );
            g.draw_rectangle_image(
                tool_rect,
                tool_img,
            );

            // top border
            g.draw_line(
                Vector2::new(0.0, SCREEN_HEIGHT as f32 - HOTBAR_HEIGHT),
                Vector2::new(SCREEN_WIDTH, SCREEN_HEIGHT as f32 - HOTBAR_HEIGHT),
                2.0,
                Color::DARK_GRAY,
            );

            // open item menu
            if let Some(i1) = self.open_item_menu {
                if i1 < HOTBAR_ITEMS.len() {
                    let img_x = i1 as f32 * (HOTBAR_CELL_SIZE + HOTBAR_CELL_SPACING) + HOTBAR_CELL_SPACING;
                    for i2 in 0..HOTBAR_ITEMS[i1].len() {
                        let id = HOTBAR_ITEMS[i1][i2].id;
                        let cell_img = &assets.cells.get(&id).unwrap()[usize::from(self.direction)];
                        let rect = Rectangle::new(
                            Vector2::new(
                                img_x,
                                SCREEN_HEIGHT as f32 - HOTBAR_HEIGHT - HOTBAR_CELL_SIZE - HOTBAR_CELL_SPACING - (i2 as f32 * (HOTBAR_CELL_SPACING + HOTBAR_CELL_SIZE)),
                            ),
                            Vector2::new(
                                img_x + HOTBAR_CELL_SIZE,
                                SCREEN_HEIGHT as f32 - HOTBAR_HEIGHT - HOTBAR_CELL_SPACING - (i2 as f32 * (HOTBAR_CELL_SPACING + HOTBAR_CELL_SIZE)),
                            ),
                        );
                        g.draw_rectangle_image_tinted(
                            rect.clone(),
                            Color::from_hex_argb(if self.hotbar_state[i1] == i2 { 0xffffffff } else { 0x7fffffff }),
                            cell_img,
                        );
                        if is_inside(rect.clone(), self.mouse_pos) {
                            let position = rect.top_right() + Vector2::new(HOTBAR_CELL_SPACING, 0.0);
                            self.hotbar_item_text.as_ref().unwrap().get(&id).unwrap().draw(g, position);
                        }
                    }
                }
                else {
                    let img_x = SCREEN_WIDTH as f32 - HOTBAR_CELL_SIZE - HOTBAR_CELL_SPACING;
                    for i2 in 0..3 {
                        let img = match i2 {
                            0 => &assets.tool_place,
                            1 => &assets.tool_rect,
                            2 => &assets.tool_circle,
                            _ => unreachable!(),
                        };
                        let rect = Rectangle::new(
                            Vector2::new(
                                img_x,
                                SCREEN_HEIGHT as f32 - HOTBAR_HEIGHT - HOTBAR_CELL_SIZE - HOTBAR_CELL_SPACING - (i2 as f32 * (HOTBAR_CELL_SPACING + HOTBAR_CELL_SIZE)),
                            ),
                            Vector2::new(
                                img_x + HOTBAR_CELL_SIZE,
                                SCREEN_HEIGHT as f32 - HOTBAR_HEIGHT - HOTBAR_CELL_SPACING - (i2 as f32 * (HOTBAR_CELL_SPACING + HOTBAR_CELL_SIZE)),
                            ),
                        );
                        g.draw_rectangle_image_tinted(
                            rect.clone(),
                            Color::from_hex_argb(if tool_to_index(self.placement_tool) == i2 { 0xffffffff } else { 0x7fffffff }),
                            img,
                        );
                        // if is_inside(rect.clone(), self.mouse_pos) {
                        //     let position = rect.top_right() + Vector2::new(HOTBAR_CELL_SPACING, 0.0);
                        //     draw_tooltip(g, position, self.hotbar_item_text.as_ref().unwrap().get(&id).unwrap());
                        // }
                    }
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
        
        // ticks per second
        g.draw_text(
            Vector2::new(10.0, 50.0),
            Color::WHITE,
            &assets.font.layout_text(&format!("Ticks per second: {}", 1000.0/self.tick_times.iter().sum::<f32>()), 17.0, TextOptions::new()),
        );

        // separate thread updating
        if self.threaded {
            g.draw_text(
                Vector2::new(10.0, 50.0),
                Color::WHITE,
                &assets.font.layout_text("Separate thread updating enabled", 17.0, TextOptions::new()),
            );
        }

        helper.request_redraw();
	}

    fn on_key_down(&mut self, window: &mut WindowHelper<()>, virtual_key_code: Option<VirtualKeyCode>, _: KeyScancode) {
        fn set_running(this: &mut WinHandler, running: bool) {
            if running && this.is_initial {
                this.is_initial = false;
                unsafe { initial = grid.clone(); }
            }

            if this.threaded {
                if !running {
                    if let Some(state) = &this.running_state {
                        state.lock().unwrap().0 = false;
                        this.running_state = None;
                    }
                }
                this.running = running;
                if this.running && this.running_state.is_none() {
                    this.running_state = Some(run_update_loop(unsafe { initial.clone() }, unsafe { grid.clone() }));
                }
            }
            else {
                this.running = running;
            }
        }

        if let Some(key) = virtual_key_code {
            println!("{:?}", key);
            self.keys.insert(key);
            match key {
                VirtualKeyCode::Q if self.keys.contains(&COMMAND_KEY) => {
                    window.terminate_loop();
                },

                VirtualKeyCode::Z if self.keys.contains(&COMMAND_KEY) => {
                    if let Some(action) = self.undo_stack.pop() {
                        action.undo_on(unsafe { &mut grid });
                    }
                },

                VirtualKeyCode::Escape => self.show_help = !self.show_help,

                VirtualKeyCode::Space => { set_running(self, !self.running) },
                VirtualKeyCode::G => { if !self.running { unsafe { do_tick(&mut self.is_initial); } } },
                VirtualKeyCode::T => {
                    if !self.is_initial {
                        set_running(self, false);
                        unsafe { grid = initial.clone(); }
                        self.is_initial = true;
                    }
                },

                VirtualKeyCode::Q => self.direction -= 1,
                VirtualKeyCode::E => self.direction += 1,

                VirtualKeyCode::R => unsafe { if self.keys.contains(&VirtualKeyCode::LAlt) { scale_tool(&mut self.placement_tool, -2) } else { screen_zoom /= 1.2 } },
                VirtualKeyCode::F => unsafe { if self.keys.contains(&VirtualKeyCode::LAlt) { scale_tool(&mut self.placement_tool,  2) } else { screen_zoom *= 1.2 } },

                VirtualKeyCode::I => {
                    let mut clip: ClipboardContext = ClipboardProvider::new().unwrap();
                    let text = clip.get_contents().unwrap();
                    unsafe {
                        grid = import(text.as_str()).unwrap();
                        screen_x = grid.width as f32 / 2.0;
                        screen_y = grid.height as f32 / 2.0;
                        screen_zoom = 1.0;
                    }
                },
                VirtualKeyCode::O => {
                    let mut clip: ClipboardContext = ClipboardProvider::new().unwrap();
                    let text = unsafe { export_q1(&grid) };
                    clip.set_contents(text).unwrap();
                },
                VirtualKeyCode::P => {
                    let mut clip: ClipboardContext = ClipboardProvider::new().unwrap();
                    let text = unsafe { export_q2(&grid) };
                    clip.set_contents(text).unwrap();
                },

                VirtualKeyCode::M => {
                    if !self.running {
                        self.threaded = !self.threaded;
                    }
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

        unsafe {
            let len = HOTBAR_ITEMS.len();

            if let Some(i1) = self.open_item_menu {
                if i1 < len {
                    let img_x = i1 as f32 * (HOTBAR_CELL_SIZE + HOTBAR_CELL_SPACING) + HOTBAR_CELL_SPACING;
                    for i2 in 0..HOTBAR_ITEMS[i1].len() {
                        let rect = Rectangle::new(
                            Vector2::new(
                                img_x,
                                SCREEN_HEIGHT as f32 - HOTBAR_HEIGHT - HOTBAR_CELL_SIZE - HOTBAR_CELL_SPACING - (i2 as f32 * (HOTBAR_CELL_SPACING + HOTBAR_CELL_SIZE)),
                            ),
                            Vector2::new(
                                img_x + HOTBAR_CELL_SIZE,
                                SCREEN_HEIGHT as f32 - HOTBAR_HEIGHT - HOTBAR_CELL_SPACING - (i2 as f32 * (HOTBAR_CELL_SPACING + HOTBAR_CELL_SIZE)),
                            ),
                        );
                        if is_inside(rect, self.mouse_pos) && button == MouseButton::Left {
                            self.hotbar_state[i1] = i2;
                            self.place = false;
                        }
                    }
                }
                else {
                    let img_x = SCREEN_WIDTH as f32 - HOTBAR_CELL_SPACING - HOTBAR_CELL_SIZE;
                    for i2 in 0..3 {
                        let rect = Rectangle::new(
                            Vector2::new(
                                img_x,
                                SCREEN_HEIGHT as f32 - HOTBAR_HEIGHT - HOTBAR_CELL_SIZE - HOTBAR_CELL_SPACING - (i2 as f32 * (HOTBAR_CELL_SPACING + HOTBAR_CELL_SIZE)),
                            ),
                            Vector2::new(
                                img_x + HOTBAR_CELL_SIZE,
                                SCREEN_HEIGHT as f32 - HOTBAR_HEIGHT - HOTBAR_CELL_SPACING - (i2 as f32 * (HOTBAR_CELL_SPACING + HOTBAR_CELL_SIZE)),
                            ),
                        );
                        if is_inside(rect, self.mouse_pos) && button == MouseButton::Left {
                            self.placement_tool = match i2 {
                                0 => Tool::Place,
                                1 => Tool::Rect(5),
                                2 => Tool::Circle(5),
                                _ => unreachable!(),
                            };
                            self.place = false;
                        }
                    }
                }
                self.open_item_menu = None;
            }

            #[allow(clippy::needless_range_loop)]
            for i in 0..len {
                let rect = Rectangle::new(
                    Vector2::new(
                        i as f32 * (HOTBAR_CELL_SIZE + HOTBAR_CELL_SPACING) + HOTBAR_CELL_SPACING,
                        SCREEN_HEIGHT as f32 - HOTBAR_HEIGHT + HOTBAR_CELL_SPACING,
                    ),
                    Vector2::new(
                        i as f32 * (HOTBAR_CELL_SIZE + HOTBAR_CELL_SPACING) + HOTBAR_CELL_SIZE + HOTBAR_CELL_SPACING,
                        SCREEN_HEIGHT as f32 - HOTBAR_CELL_SPACING,
                    ),
                );
                if is_inside(rect, self.mouse_pos) {
                    if button == MouseButton::Left {
                        self.active_item = i;
                        self.open_item_menu = None;
                    }
                    else if button == MouseButton::Right {
                        self.active_item = i;
                        self.open_item_menu = Some(i);
                    }
                }
            }

            let tools_rect = Rectangle::new(
                Vector2::new(
                    SCREEN_WIDTH as f32 - HOTBAR_CELL_SIZE - HOTBAR_CELL_SPACING,
                    SCREEN_HEIGHT as f32 - HOTBAR_CELL_SIZE - HOTBAR_CELL_SPACING,
                ),
                Vector2::new(
                    SCREEN_WIDTH as f32 - HOTBAR_CELL_SPACING,
                    SCREEN_HEIGHT as f32 - HOTBAR_CELL_SPACING,
                ),
            );
            if is_inside(tools_rect, self.mouse_pos) {
                self.open_item_menu = Some(len);
                self.place = false;
            }

            if self.place {
                self.undo_stack.start();
            }
        }

    }
    fn on_mouse_button_up(&mut self, _: &mut WindowHelper<()>, _: MouseButton) {
        self.place = true;
        self.mouse = None;
    }
    fn on_mouse_move(&mut self, _: &mut WindowHelper<()>, position: Vector2<f32>) {
        self.mouse_pos = position;
    }

    fn on_mouse_wheel_scroll(&mut self, _: &mut WindowHelper<()>, distance: MouseScrollDistance) {
        unsafe {
            match distance {
                MouseScrollDistance::Lines { x: _, y, z: _ } => {
                    if self.keys.contains(&VirtualKeyCode::LAlt) {
                        if y > 1.0 {
                            scale_tool(&mut self.placement_tool, -2);
                        }
                        else if y < -1.0 {
                            scale_tool(&mut self.placement_tool, 2);
                        }
                    }
                    else if y > 0.0 {
                        screen_zoom *= 1.2f64.powf(y.abs()) as f32;
                    }
                    else {
                        screen_zoom /= 1.2f64.powf(y.abs()) as f32;
                    }
                },
                MouseScrollDistance::Pixels { x: _, y, z: _ } => {
                    if self.keys.contains(&VirtualKeyCode::LAlt) {
                        if y > 8.0 {
                            scale_tool(&mut self.placement_tool, -2);
                        }
                        else if y < -8.0 {
                            scale_tool(&mut self.placement_tool, 2);
                        }
                    }
                    else if y > 0.0 {
                        screen_zoom *= 1.0 + (y.abs() as f32 / 100.0);
                    }
                    else {
                        screen_zoom /= 1.0 + (y.abs() as f32 / 100.0);
                    }
                },
                _ => {}
            }
        }
    }
}

unsafe fn do_tick(is_initial: &mut bool) {

    update(&mut grid);
}

fn scale_tool(tool: &mut Tool, change: isize) {
    let value = match *tool {
        Tool::Place => 1,
        Tool::Rect(v) => v,
        Tool::Circle(v) => v,
    } + change;
    if value < 1 {
        *tool = Tool::Place;
    }
    else {
        *tool = match (value, *tool) {
            (1, _) => Tool::Place,
            (_, Tool::Place) => Tool::Rect(value),
            (value, Tool::Rect(_)) => Tool::Rect(value),
            (value, Tool::Circle(_)) => Tool::Circle(value),
        }
    }
}

fn tool_to_index(tool: Tool) -> usize {
    match tool {
        Tool::Place => 0,
        Tool::Rect(_) => 1,
        Tool::Circle(_) => 2,
    }
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

            if let Some(cell) = grid.get_unchecked(x as isize, y as isize) {
                // draw cell
                g.draw_rectangle_image(cell_rect, &assets.cells.get(&cell.id).unwrap()[usize::from(cell.direction)]);
                if cell.id == MAILBOX {
                    if let Some((id, dir)) = cell.contained_cell {
                        let cell_rect = Rectangle::new(
                            Vector2::new(
                                ((x as f32 - screen_x) * CELL_SIZE + CELL_SIZE / 4.0) * screen_zoom + screen_w_half,
                                ((screen_y - y as f32 - 1.0) * CELL_SIZE + CELL_SIZE / 4.0) * screen_zoom + screen_h_half,
                            ),
                            Vector2::new(
                                ((x as f32 - screen_x + 1.0) * CELL_SIZE - CELL_SIZE / 4.0) * screen_zoom + screen_w_half,
                                ((screen_y - y as f32) * CELL_SIZE - CELL_SIZE / 4.0) * screen_zoom + screen_h_half,
                            )
                        );
                        g.draw_rectangle_image(cell_rect, &assets.cells.get(&id).unwrap()[usize::from(cell.direction + dir)]);
                    }
                }
            }
            else {
                // draw background
                g.draw_rectangle_image(cell_rect.clone(), &assets.cell_bg);
            }
        }
    }
}

unsafe fn draw_ghost_cell(assets: &Assets, g: &mut Graphics2D, x: isize, y: isize, cell: &Cell) {
    let screen_w_half = SCREEN_WIDTH / 2.0;
    let screen_h_half = SCREEN_HEIGHT / 2.0;
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
    g.draw_rectangle_image_tinted(
        cell_rect,
        Color::from_hex_argb(0x70ffffff),
        &assets.cells.get(&cell.id).unwrap()[usize::from(cell.direction)]
    );
}

struct Tooltip {
    title: Text,
    data: Option<Text>,
    description: Option<Text>,
}

const TITLE_SIZE: f32 = 30.0;
const DATA_SIZE: f32 = 17.0;
const DESCRIPTION_SIZE: f32 = 23.0;
impl Tooltip {
    pub fn new(font: &Font, title: impl ToString, data: Option<impl ToString>, description: Option<impl ToString>) -> Tooltip {
        Tooltip {
            title: font.layout_text(&title.to_string(), TITLE_SIZE, TextOptions::new().with_wrap_to_width(TOOLTIP_WIDTH - TOOLTIP_PADDING * 2.0, TextAlignment::Left)),
            data: data.map(|d| font.layout_text(&d.to_string(), DATA_SIZE, TextOptions::new().with_wrap_to_width(TOOLTIP_WIDTH - TOOLTIP_PADDING * 2.0, TextAlignment::Left))),
            description: description.map(|d| font.layout_text(&d.to_string(), DESCRIPTION_SIZE, TextOptions::new().with_wrap_to_width(TOOLTIP_WIDTH - TOOLTIP_PADDING * 2.0, TextAlignment::Left))),
        }
    }

    pub fn draw(&self, g: &mut Graphics2D, draw_position: Vector2<f32>) {
        let rect = Rectangle::new(
            draw_position,
            draw_position + Vector2::new(TOOLTIP_WIDTH, TOOLTIP_HEIGHT)
        );
        g.draw_rectangle(
            rect.clone(),
            Color::from_hex_argb(0xcc555555),
        );
        draw_stroke_rect(g, rect, Color::from_hex_argb(0xff111111), 2.0);

        g.draw_text(
            draw_position + Vector2::new(TOOLTIP_PADDING, TOOLTIP_PADDING),
            Color::WHITE,
            &self.title,
        );

        let mut text_position = draw_position + Vector2::new(TOOLTIP_PADDING, TOOLTIP_PADDING + self.title.height() + 5.0);

        if let Some(ref data) = self.data {
            g.draw_text(
                text_position,
                Color::WHITE,
                data,
            );
            text_position.y += data.height() + 5.0;
        }

        text_position.y += 5.0;

        if let Some(ref description) = self.description {
            g.draw_text(
                text_position,
                Color::WHITE,
                description,
            );
        }
    }
}

fn draw_stroke_rect(g: &mut Graphics2D, rect: Rectangle, color: Color, stroke_width: f32) {
    g.draw_line(
        *rect.top_left(),
        rect.top_right(),
        stroke_width,
        color,
    );
    g.draw_line(
        rect.top_right(),
        *rect.bottom_right(),
        stroke_width,
        color,
    );
    g.draw_line(
        *rect.bottom_right(),
        rect.bottom_left(),
        stroke_width,
        color,
    );
    g.draw_line(
        rect.bottom_left(),
        *rect.top_left(),
        stroke_width,
        color,
    );
}

struct Assets {
    cell_bg: ImageHandle,
    cells: HashMap<CellType, [ImageHandle; 4]>,

    tool_place: ImageHandle,
    tool_rect: ImageHandle,
    tool_circle: ImageHandle,

    font: Font,
}

fn create_rotated_textures(amount: usize, path: PathBuf) -> [ImageBuffer<Rgba<u8>, Vec<u8>>; 4] {
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

const UNDO_STACK_SIZE: usize = 20;
struct CellUndoStack(Vec<UndoAction>);

impl CellUndoStack {
    fn new() -> Self {
        CellUndoStack(Vec::new())
    }

    fn start(&mut self) {
        self.0.push(UndoAction::new());
        if self.0.len() > UNDO_STACK_SIZE {
            self.0.remove(0);
        }
    }

    fn insert(&mut self, x: isize, y: isize, cell: Option<Cell>) {
        self.0.last_mut().unwrap().insert((x, y), cell);
    }

    fn pop(&mut self) -> Option<UndoAction> {
        self.0.pop()
    }
}

struct UndoAction(HashMap<(isize, isize), Option<Cell>>);

impl UndoAction {
    fn new() -> Self {
        UndoAction(HashMap::new())
    }

    fn insert(&mut self, pos: (isize, isize), cell: Option<Cell>) {
        self.0.insert(pos, cell);
    }

    fn undo_on(&self, action_grid: &mut Grid) {
        for (pos, cell) in self.0.iter() {
            action_grid.set_cell(pos.0, pos.1, cell.clone());
        }
    }
}
