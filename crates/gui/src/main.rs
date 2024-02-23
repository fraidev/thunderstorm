use button::Button;
use gpui::*;
use state::State;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

mod button;
mod state;

fn footbar() -> Div {
    div()
        .flex()
        .bg(rgb(0x808080))
        .justify_center()
        .items_center()
        .text_xl()
        .text_color(rgb(0xffffff))
        .h(Pixels(25.0))
        .child("Footer")
}

fn headbar() -> Div {
    div()
        .flex()
        .bg(rgb(0x808080))
        .justify_center()
        .items_center()
        .text_xl()
        .text_color(rgb(0xffffff))
        .h(Pixels(50.0))
        .child("Header")
}

fn content(score: Arc<Mutex<i32>>) -> Div {
    let s = *score.lock().unwrap();
    let button = Button::new().on_click(move |_cx| {
        let mut score = score.lock().unwrap();
        *score += 1;
        println!("Score: {}", *score);
    });

    let children = div().child(button).child(format!("Score: {}", s));

    div()
        .flex()
        .bg(rgb(0xffffff))
        .h_auto()
        .flex_auto()
        .justify_center()
        .items_center()
        .text_xl()
        .text_color(rgb(0x000000))
        .child(children)
}

struct Root {
    score: Arc<Mutex<i32>>,
}

impl Render for Root {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .min_h(DefiniteLength::Fraction(1.0))
            .children(vec![headbar(), content(self.score.clone()), footbar()])
    }
}

fn main() {
    let app = App::new();
    app.run(|cx| {
        let window = build_window_options(None, None, cx);
        // cx.activate(false);
        cx.open_window(window, |cx| {
            State::init(cx);
            cx.new_view(|_| Root {
                score: Arc::new(Mutex::new(0)),
            })
        });
    });
}

pub fn build_window_options(
    bounds: Option<WindowBounds>,
    display_uuid: Option<Uuid>,
    cx: &mut AppContext,
) -> WindowOptions {
    let bounds = bounds.unwrap_or(WindowBounds::Maximized);
    let display = display_uuid.and_then(|uuid| {
        cx.displays()
            .into_iter()
            .find(|display| display.uuid().ok() == Some(uuid))
    });
    WindowOptions {
        bounds,
        titlebar: Some(TitlebarOptions {
            title: None,
            appears_transparent: true,
            traffic_light_position: Some(point(px(8.), px(8.))),
        }),
        center: false,
        focus: true,
        show: true,
        kind: WindowKind::Normal,
        is_movable: true,
        display_id: display.map(|display| display.id()),
    }
}
