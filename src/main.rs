use fltk_theme::{color_themes, ColorTheme};
use rust_im::{backend::server::api::Server, log::init, string};
use std::env::args;

fn main() {
    ColorTheme::new(color_themes::BLACK_THEME).apply();
    init();

    let args = args().collect::<Vec<String>>();
    println!("{args:?}");
    if args.len() == 2 && args[1] == string!("s") {
        Server::new(3888);
        loop {}
    };
    rust_im::gui::run();
}
