mod app;
mod components;
mod cpu_stress;
mod gpu_stress;
mod overclock;
mod ram_stress;
mod sensors;
mod theme;

use gpui::{
    prelude::*, px, size, App, Application, Bounds, TitlebarOptions, WindowBounds, WindowOptions,
};

use app::StressApp;

fn main() {
    Application::new().run(|cx: &mut App| {
        let (width, height) = theme::START_SIZE;
        let (min_width, min_height) = theme::MIN_SIZE;
        let bounds = Bounds::centered(None, size(px(width), px(height)), cx);

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            window_min_size: Some(size(px(min_width), px(min_height))),
            // Must match the .desktop file name, so the taskbar shows the right icon.
            app_id: Some("linstress".to_string()),
            titlebar: Some(TitlebarOptions {
                title: Some("LinStress".into()),
                ..Default::default()
            }),
            ..Default::default()
        };

        cx.open_window(options, |_window, cx| cx.new(StressApp::new))
            .expect("failed to open the window");
        cx.activate(true);
    });
}
