use eframe::egui;

use crate::app::Application;

pub struct TopBar;

impl TopBar {
    pub(crate) fn ui(ctx: &egui::Context, app: &mut Application) {
        egui::TopBottomPanel::top("top")
            // .min_height(self.world_size.y as f32)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let play_pause = if app.paused { "Play" } else { "Pause" };
                    if ui.button(play_pause).clicked() {
                        app.on_pause_play_button();
                    }

                    ui.add(
                        egui::Slider::new(&mut app.speed, 1.0..=5.0)
                            .fixed_decimals(1)
                            .text("Speed"),
                    );

                    ui.label(format!("{:3.1}x", app.time_ratio));
                    ui.separator();
                    ui.label(format!("{:3.0} SPS", app.steps_per_second));
                    ui.separator();
                    ui.label(format!("{:3.0} FPS", app.frames_per_second));
                    ui.separator();

                    if ui.button("Selection").clicked() {
                        println!("Selection")
                    }
                });
            });
    }
}
