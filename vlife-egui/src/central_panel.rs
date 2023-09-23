use eframe::egui;

use vlife_simulator::Scalar;

use crate::app::Application;
use crate::world_panel::WorldPanel;

pub struct CentralPanel;

impl CentralPanel {
    pub(crate) fn ui(ctx: &egui::Context, app: &mut Application, dt: Scalar) {
        egui::CentralPanel::default().show(ctx, |ui| {
            WorldPanel::ui(ctx, ui, app, dt);
        });
    }
}
