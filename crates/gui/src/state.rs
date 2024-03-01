use std::path::PathBuf;

use gpui::{AppContext, Context, Global, Model};

pub struct State {
    pub score: i32,
    pub file_path: Option<String>,
    pub pending_files: Vec<PathBuf>,
    pub transfers: Vec<String>,
}

#[derive(Clone)]
pub struct StateModel {
    pub inner: Model<State>,
}

impl Global for StateModel {}

impl StateModel {
    pub fn init(cx: &mut AppContext) -> Self {
        let this = Self {
            inner: cx.new_model(|_| State {
                score: 0,
                file_path: None,
                pending_files: vec![],
                transfers: vec![],
            }),
        };
        // this.push(RootListBuilder {}, cx);
        cx.set_global(this.clone());
        this
    }
    pub fn update(f: impl FnOnce(&mut Self, &mut AppContext), cx: &mut AppContext) {
        cx.update_global::<Self, _>(|mut this, cx| {
            f(&mut this, cx);
        });
    }
}
impl Global for State {}
