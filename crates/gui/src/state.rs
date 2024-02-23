use std::sync::{Arc, Mutex};

use gpui::{AppContext, Global};

pub struct State {
    score: Arc<Mutex<i32>>,
}

impl State {
    pub fn init(cx: &mut AppContext) {
        let state = State {
            score: Arc::new(Mutex::new(0)),
        };
        cx.set_global(state);
    }
}

impl Global for State {}
