mod app;

use vlife_physics::Vec2;

use app::Application;

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    eframe::run_native(
        "V-Life",
        eframe::NativeOptions {
            // initial_window_size: Some(egui::vec2(800.0, 480.0)),
            ..Default::default()
        },
        Box::new(|_cc| {
            let application = create_application();
            Box::new(application)
        }),
    )
}

fn create_application() -> Application {
    let world_size = Vec2::new(800.0, 480.0);
    Application::new(world_size)
}
