use gpui::{prelude::FluentBuilder, *};
use gpui_macros::IntoElement;

#[derive(IntoElement)]
pub struct Button {
    pub base: Div,
    pub(super) disabled: bool,
    on_click: Option<Box<dyn Fn(&mut WindowContext) + 'static>>,
}

impl Button {
    pub fn new() -> Self {
        Self {
            on_click: None,
            disabled: false,
            base: div(),
        }
    }
}

impl RenderOnce for Button {
    fn render(self, _cx: &mut gpui::WindowContext) -> impl gpui::prelude::IntoElement {
        self.base
            .flex()
            .bg(rgb(0x000000))
            .size_full()
            .text_xl()
            .text_color(rgb(0xffffff))
            .justify_center()
            .items_center()
            .child("Button")
            .on_mouse_move(|_cx, _event| {
                // println!("Mouse moved!");
            })
            .on_mouse_down(MouseButton::Left, move |_, cx| {
                cx.prevent_default();

                if let Some(on_click) = &self.on_click {
                    on_click(cx);
                }
            })
            .hover(|sr| sr.bg(rgb(0x0000ff)))
    }
}

impl Button {
    pub fn on_click(mut self, handler: impl Fn(&mut WindowContext) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}
