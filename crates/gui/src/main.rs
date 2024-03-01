#![allow(non_snake_case)]

use gpui::*;
use state::StateModel;

mod app_menu;
mod button;
mod layout;
mod state;
mod utils;
mod window;
mod workspace;

fn main() {
    let app = App::new();
    app.run(|cx| {
        let window = window::build_window_options(None, cx);

        check_mode(cx);
        app_menu::init_actions(cx);
        cx.open_window(window, |cx| {
            StateModel::init(cx);
            cx.set_menus(app_menu::app_menus());
            cx.new_view(|_| layout::Root {})
        });
    });
}

#[cfg(debug_assertions)]
fn check_mode(_cx: &mut AppContext) {
    println!("Debug mode");
}

#[cfg(not(debug_assertions))]
fn check_mode(cx: &mut AppContext) {
    cx.activate(false);
}
