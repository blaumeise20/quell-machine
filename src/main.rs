#![allow(non_upper_case_globals)]

mod game;

use game::rendering::WinHandler;
use speedy2d::{Window, window::{WindowCreationOptions, WindowSize}, dimen::Vector2};

fn main() {
    let window = Window::new_with_options(
        "Quell Machine",
        WindowCreationOptions::new_windowed(
            WindowSize::ScaledPixels(Vector2::new(800.0, 600.0)),
            // Some(WindowPosition::Center),
            None,
        )
            .with_resizable(false)
    ).unwrap();

    window.run_loop(WinHandler::new());
}
