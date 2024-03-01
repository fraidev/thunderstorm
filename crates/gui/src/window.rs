use gpui::*;
use uuid::Uuid;

pub static WIDTH: f64 = 800.0;
pub static HEIGHT: f64 = 450.0;

pub fn build_window_options(display_uuid: Option<Uuid>, cx: &mut AppContext) -> WindowOptions {
    let bounds = cx.displays().first().map(|d| d.bounds()).unwrap_or(Bounds {
        origin: Point::new(GlobalPixels::from(0.0), GlobalPixels::from(0.0)),
        size: Size {
            width: GlobalPixels::from(1920.0),
            height: GlobalPixels::from(1080.0),
        },
    });
    let center = bounds.center();
    let width = GlobalPixels::from(WIDTH);
    let height = GlobalPixels::from(HEIGHT);
    let x: GlobalPixels = center.x - width / 2.0;
    let y: GlobalPixels = center.y - height / 2.0;
    let bounds: Bounds<GlobalPixels> = Bounds::new(Point { x, y }, Size { width, height });
    let display = display_uuid.and_then(|uuid| {
        cx.displays()
            .into_iter()
            .find(|display| display.uuid().ok() == Some(uuid))
    });

    WindowOptions {
        bounds: WindowBounds::Fixed(bounds),
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
