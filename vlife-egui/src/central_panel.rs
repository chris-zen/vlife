use eframe::egui;

use vlife_simulator::Real;

use crate::app::Application;
use crate::world_panel::WorldPanel;

pub struct CentralPanel;

impl CentralPanel {
    pub(crate) fn ui(ctx: &egui::Context, app: &mut Application, dt: Real) {
        egui::CentralPanel::default().show(ctx, |ui| {
            WorldPanel::ui(ctx, ui, app, dt);
        });
    }
}
