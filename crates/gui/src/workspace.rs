use gpui::{actions, PathPromptOptions};

use crate::state::StateModel;

actions!(thunderstorm, [Open]);

pub struct Workspace {
    state: StateModel,
}

pub fn open(_: &Open, cx: &mut gpui::AppContext) {
    let paths = cx.prompt_for_paths(PathPromptOptions {
        files: true,
        directories: false,
        multiple: true,
    });

    cx.spawn(|cx| async move {
        let Ok(Some(paths)) = paths.await else {
            return;
        };

        println!("Opening {:?}", paths);

        cx.update(|cx| open_torrent_for_paths(paths, cx)).unwrap();
    })
    .detach()
}

pub fn open_torrent_for_paths(paths: Vec<std::path::PathBuf>, cx: &mut gpui::AppContext) {
    StateModel::update(
        |state, cx| {
            state.inner.update(cx, |s, _| {
                s.score += 10;
                s.pending_files = paths.clone()
            });
        },
        cx,
    )
}
