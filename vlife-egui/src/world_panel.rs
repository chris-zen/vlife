use eframe::{
    egui::{self, Color32, Painter, Pos2, Rect, Response, Rgba, Rounding, Sense, Shape, Stroke},
    emath::{self, RectTransform},
    epaint::{CircleShape, RectShape},
};
use nalgebra::{Const, OPoint, UnitComplex};
use num_traits::float::FloatConst;

use vlife_simulator::{cell, Scalar, Vec2};

use crate::app::Application;

pub struct WorldPanel;

impl WorldPanel {
    pub(crate) fn ui(_ctx: &egui::Context, ui: &mut egui::Ui, app: &mut Application, dt: Scalar) {
        egui::Frame::canvas(ui.style())
            .inner_margin(0.0)
            .outer_margin(0.0)
            .fill(Rgba::from_rgb(0.0, 0.0, 0.0).into())
            .show(ui, |ui| {
                Self::render_simulation(ui, app, dt);
            });
    }

    fn render_simulation(ui: &mut egui::Ui, app: &mut Application, dt: Scalar) {
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

        Self::handle_interactions(app, response, from_screen);

        let size_fill_color = Rgba::from_white_alpha(0.0);
        let size_stroke_color = Rgba::from_white_alpha(0.1);
        let normal_color = Rgba::from_gray(1.0);
        let selected_color = Rgba::from_rgb(1.0, 0.7, 0.0);
        let velocity_color = Rgba::from_rgba_unmultiplied(0.0, 0.0, 1.0, 0.5);
        let acceleration_color = Rgba::from_rgba_unmultiplied(1.0, 0.0, 0.0, 0.3);

        for (cell_id, cell) in app.simulator.cells() {
            if let Some(object) = app.simulator.get_cell_object(cell_id) {
                let position = object.position();
                let velocity = object.velocity();
                let velocity_p1 = position + velocity.normalize() * object.radius();
                let velocity_p2 = velocity_p1 + velocity;
                let acceleration = cell.movement_velocity();
                let acceleration_p1 = position + acceleration.normalize() * object.radius();
                let acceleration_p2 = acceleration_p1 + acceleration;

                let energy = (cell.energy() / cell::MAX_ENERGY).min(1.0) as f32;
                let energy_loss = ((-cell.energy_delta()).max(0.0) / dt).min(1.0) as f32;
                let energy_gain = (cell.energy_delta().max(0.0) / dt).min(1.0) as f32;

                let fill_color = Rgba::from_rgb(energy_loss, energy, energy_gain);
                let stroke_color = app
                    .selected_cell
                    .filter(|selected_id| *selected_id == cell_id)
                    .map_or(normal_color, |_| selected_color);

                painter.add(RectShape::stroke(
                    screen_rect.expand(1.0),
                    Rounding::none(),
                    Stroke::new(1.0, Color32::from_gray(128)),
                ));
                painter.add(CircleShape {
                    center: position.transform_pos(&to_screen),
                    radius: cell.size() as f32,
                    fill: size_fill_color.into(),
                    stroke: (1.0, size_stroke_color).into(),
                });
                painter.add(CircleShape {
                    center: position.transform_pos(&to_screen),
                    radius: cell.contracted_size() as f32,
                    fill: fill_color.into(),
                    stroke: (1.0, stroke_color).into(),
                });

                Self::paint_eyes(
                    &painter,
                    &to_screen,
                    position,
                    cell.movement_direction(),
                    cell.contracted_size(),
                );

                painter.add(Shape::line_segment(
                    [
                        velocity_p1.transform_pos(&to_screen),
                        velocity_p2.transform_pos(&to_screen),
                    ],
                    (1.0, velocity_color),
                ));
                painter.add(Shape::line_segment(
                    [
                        acceleration_p1.transform_pos(&to_screen),
                        acceleration_p2.transform_pos(&to_screen),
                    ],
                    (1.0, acceleration_color),
                ));
            }
        }
    }

    fn handle_interactions(app: &mut Application, response: Response, from_screen: RectTransform) {
        // if let Some(pos) = response.hover_pos() {
        // println!("H: {:?} {}", pos, response.hovered());
        // }
        if let Some(pos) = response.interact_pointer_pos() {
            // println!("I: {:?} {} {}", pos, response.clicked(), response.dragged());
            if response.clicked() {
                let pos = from_screen.transform_pos(pos);
                if let Some(cell_id) = app
                    .simulator
                    .get_cell_id_closer_to(pos.x as Scalar, pos.y as Scalar)
                {
                    app.on_cell_selected(cell_id);
                }
            }
        }
    }

    fn paint_eyes(
        painter: &Painter,
        to_screen: &RectTransform,
        position: Vec2,
        direction: Scalar,
        contracted_size: Scalar,
    ) {
        let left_eye_angle = UnitComplex::new(-direction + 0.1 * Scalar::PI());
        let right_eye_angle = UnitComplex::new(-direction - 0.1 * Scalar::PI());
        let left_eye_vec = left_eye_angle.transform_vector(&Vec2::x_axis());
        let right_eye_vec = right_eye_angle.transform_vector(&&Vec2::x_axis());
        let white_eye_distance = 0.8 * contracted_size;
        let white_eye_size = 0.25 * contracted_size as f32;
        let pupil_eye_distance = 0.78 * contracted_size;
        let pupil_eye_size = 0.15 * contracted_size as f32;

        painter.add(CircleShape {
            center: (left_eye_vec * white_eye_distance + position).transform_pos(&to_screen),
            radius: white_eye_size,
            fill: Color32::WHITE,
            stroke: Stroke::NONE,
        });
        painter.add(CircleShape {
            center: (left_eye_vec * pupil_eye_distance + position).transform_pos(&to_screen),
            radius: pupil_eye_size,
            fill: Color32::BLACK,
            stroke: Stroke::NONE,
        });
        painter.add(CircleShape {
            center: (right_eye_vec * white_eye_distance + position).transform_pos(&to_screen),
            radius: white_eye_size,
            fill: Color32::WHITE,
            stroke: Stroke::NONE,
        });
        painter.add(CircleShape {
            center: (right_eye_vec * pupil_eye_distance + position).transform_pos(&to_screen),
            radius: pupil_eye_size,
            fill: Color32::BLACK,
            stroke: Stroke::NONE,
        });
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

impl TransformToScreen for OPoint<Scalar, Const<2>> {
    fn transform_pos(&self, transform: &RectTransform) -> Pos2 {
        transform.transform_pos(Pos2::new(self.x as f32, self.y as f32))
    }
}
