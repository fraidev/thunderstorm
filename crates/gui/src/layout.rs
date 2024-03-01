use gpui::*;

use crate::{
    button::Button,
    state::StateModel,
    workspace::{self, Open},
};

fn footer(cx: &mut ViewContext<Root>) -> Div {
    let stateModel = cx.global::<StateModel>();
    let s = stateModel.inner.read(cx);

    div()
        .flex()
        .bg(rgb(0xeeeeee))
        .justify_center()
        .items_center()
        .text_xs()
        .text_color(rgb(0x000000))
        .shadow_2xl()
        .border_1()
        .h(Pixels(25.0))
        .child(format!("Transfers: {:?}", s.transfers.len()))
}

fn navbar() -> Div {
    let button = Button::new("File".to_string()).on_click(move |cx| {
        workspace::open(&Open, cx);

        StateModel::update(
            |state, cx| {
                state.inner.update(cx, |s, _| {
                    s.score += 10;
                    s.file_path = Some("test".to_string());
                });
            },
            cx,
        )
    });

    let children = div().pl_20().child(button).child("Nav Bar");

    div()
        .flex()
        .bg(rgb(0xeeeeee))
        .shadow_2xl()
        .items_center()
        .text_xl()
        .text_color(rgb(0x000000))
        .h(Pixels(50.0))
        .child(children)
}

fn content(cx: &mut ViewContext<Root>) -> Div {
    let state = cx.global::<StateModel>().clone();
    let s = state.inner.read(cx);
    let score = s.score;
    let button = Button::new("Button".to_string()).on_click(move |cx| {
        state.inner.update(cx, |s, _| {
            s.score += 1;
        });
    });

    let children = div().child(button).child(format!("Score: {}", score));

    div()
        .flex()
        .bg(rgb(0xffffff))
        .h_auto()
        .flex_auto()
        .justify_center()
        .items_center()
        .text_xl()
        .text_color(rgb(0x000000))
        .child(div().child(format!("FilePending: {:?}", s.pending_files)))
        .child(children)
}

pub struct Root {}

impl Render for Root {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .min_h(DefiniteLength::Fraction(1.0))
            .children(vec![navbar(), content(cx), footer(cx)])
    }
}
