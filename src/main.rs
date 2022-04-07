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
    ).unwrap();

    let mut path = std::env::current_exe().unwrap();
    path.pop();
    if path.file_name().to_owned().unwrap() == "MacOS" {
        path.pop();
        path.push("Resources");
        let path = path.iter().filter(|&p| p != ".").collect::<std::path::PathBuf>();
        window.run_loop(WinHandler::new(path));
    }
    else {
        window.run_loop(WinHandler::new(std::env::current_dir().unwrap()));
    }
}
