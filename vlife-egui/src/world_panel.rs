use eframe::{
    egui::{self, Color32, Pos2, Rect, Rgba, Rounding, Sense, Stroke},
    emath::{self, RectTransform},
    epaint::RectShape,
};
use eframe::epaint::PathShape;
use nalgebra::{Const, OPoint};

use vlife_simulator::{Real, Vec2};

use crate::app::Application;

pub struct WorldPanel;

impl WorldPanel {
    pub(crate) fn ui(_ctx: &egui::Context, ui: &mut egui::Ui, app: &mut Application, dt: Real) {
        egui::Frame::canvas(ui.style())
            .inner_margin(0.0)
            .outer_margin(0.0)
            .fill(Rgba::from_rgb(0.0, 0.0, 0.0).into())
            .show(ui, |ui| {
                Self::render_simulation(ui, app, dt);
            });
    }

    fn render_simulation(ui: &mut egui::Ui, app: &mut Application, dt: Real) {
        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(ui.available_width(), ui.available_height()),
            Sense::hover().union(Sense::click()),
        );

        let world_rect = Rect::from_min_max(
            Pos2::ZERO,
            Pos2::new(app.world_size.x as f32, app.world_size.y as f32),
        );
        let margin = 0.5 * (response.rect.size() - world_rect.size());
        let screen_rect = world_rect.translate(response.rect.min.to_vec2() + margin);
        let to_screen = emath::RectTransform::from_to(world_rect, screen_rect);
        let from_screen = to_screen.inverse();

        painter.add(RectShape::stroke(
            screen_rect.expand(1.0),
            Rounding::none(),
            Stroke::new(1.0, Color32::from_gray(128)),
        ));
        for cell_view in app.simulator.cells() {
            let membrane = cell_view.membrane().into_iter().map(|pos| pos.transform_pos(&to_screen)).collect::<Vec<_>>();
            for pos in &membrane {
                painter.circle(
                    *pos,
                    6.0,
                    Color32::LIGHT_BLUE,
                    Stroke::new(1.0, Color32::LIGHT_BLUE),
                );
            }
            // painter.add(PathShape::closed_line(membrane, Stroke::new(1.0, Color32::WHITE)));

            let center_pos = cell_view.position().transform_pos(&to_screen);
            painter.circle(
                center_pos,
                1.0,
                Color32::WHITE,
                Stroke::new(1.0, Color32::WHITE),
            );
            // painter.line_segment([center_pos, membrane[0]], (1.0, Color32::GOLD));

            // let pos = cell_body.ball().position();
            // let rotation = cell_body.ball().rotation_vector();
            // let radius = cell_body.cell().radius();
            // let head = rotation * radius + pos;
            // let pos = Vec2::new(pos.x, world_rect.max.y as Real - pos.y).transform_pos(&to_screen);
            // let head =
            //     Vec2::new(head.x, world_rect.max.y as Real - head.y).transform_pos(&to_screen);
            // painter.circle(
            //     pos,
            //     radius as f32,
            //     Color32::DARK_GRAY,
            //     Stroke::new(1.0, Color32::WHITE),
            // );
            // painter.line_segment([pos, head], Stroke::new(1.0, Color32::WHITE));
        }
    }
}

trait TransformToScreen {
    fn transform_pos(&self, transform: &RectTransform) -> Pos2;
}

impl TransformToScreen for Vec2 {
    fn transform_pos(&self, transform: &RectTransform) -> Pos2 {
        transform.transform_pos(Pos2::new(self.x as f32, self.y as f32))
    }
}

impl TransformToScreen for OPoint<Real, Const<2>> {
    fn transform_pos(&self, transform: &RectTransform) -> Pos2 {
        transform.transform_pos(Pos2::new(self.x as f32, self.y as f32))
    }
}
