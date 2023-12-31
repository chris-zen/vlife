mod app;
mod central_panel;
mod top_bar;
mod world_panel;

use eframe::egui;
use vlife_simulator::Vec2;

use app::Application;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    eframe::run_native(
        "V-Life",
        eframe::NativeOptions {
            initial_window_size: Some(egui::vec2(1000.0, 700.0)),
            ..Default::default()
        },
        Box::new(|_cc| {
            let application = create_application();
            Box::new(application)
        }),
    )
}

fn create_application() -> Application {
    let world_size = Vec2::new(700.0, 300.0);
    Application::new(world_size)
}
