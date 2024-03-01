use gpui::{actions, impl_actions, AppContext, Menu, MenuItem};
use serde::Deserialize;

use crate::workspace;

#[derive(Clone, PartialEq, Deserialize)]
pub struct OpenBrowser {
    pub url: String,
}

#[derive(Clone, PartialEq, Deserialize)]
pub struct OpenThundestormUrl {
    pub url: String,
}

impl_actions!(thunderstorm, [OpenBrowser, OpenThundestormUrl]);
actions!(thunderstorm, [OpenSettings, Quit]);

pub fn app_menus() -> Vec<Menu<'static>> {
    vec![
        Menu {
            name: "Thundersword",
            items: vec![MenuItem::action("Quit", Quit)],
        },
        Menu {
            name: "File",
            items: vec![
                MenuItem::action("Open…", workspace::Open),
                MenuItem::separator(),
            ],
        },
    ]
}

pub fn init_actions(cx: &mut AppContext) {
    cx.on_action(quit);
    cx.on_action(workspace::open);
}
fn quit(_: &Quit, cx: &mut AppContext) {
    cx.quit();
}
