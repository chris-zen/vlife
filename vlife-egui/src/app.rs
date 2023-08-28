use eframe::egui::{Color32, Painter, Rounding, Stroke};
use eframe::epaint::RectShape;
use eframe::{
    egui::{self, emath, Pos2, Rect, ScrollArea, Sense},
    emath::RectTransform,
    epaint::{CircleShape, Rgba, Shape},
};
use nalgebra::{Const, OPoint, UnitComplex};
use num_traits::float::FloatConst;
use std::time::Instant;

use vlife_physics::{Scalar, Vec2};
use vlife_simulator::{cell, CellId, Simulator};

const NUM_INITIAL_CELLS: usize = 500;

const DEFAULT_DELTA: Scalar = 1.0 / 60.0; // 60 Hz

pub(crate) struct Application {
    world_size: Vec2,
    simulator: Simulator,
    last_update: Option<Instant>,
    selected_cell: Option<CellId>,
    paused: bool,
    speed: f32,
}

impl Application {
    pub fn new(world_size: Vec2) -> Self {
        let simulator = Self::create_simulator(world_size);
        Self {
            world_size,
            simulator,
            last_update: None,
            selected_cell: Some(0),
            paused: false,
            speed: 1.0,
        }
    }

    fn create_simulator(world_size: Vec2) -> Simulator {
        let mut simulator = Simulator::new(world_size);
        for _ in 0..NUM_INITIAL_CELLS {
            simulator.add_random_cell();
        }
        simulator
    }

    fn update_simulation(&mut self) -> Scalar {
        match self.last_update {
            None => {
                self.last_update = Some(Instant::now());
                0.0
            }
            Some(_last_update) => {
                self.last_update = Some(Instant::now());
                // let dt = Instant::now().duration_since(last_update).as_secs_f64();
                if !self.paused {
                    let mut time = 0.0;
                    let total_time = self.speed as Scalar * DEFAULT_DELTA;
                    while time <= total_time {
                        self.simulator.update(DEFAULT_DELTA);
                        time += DEFAULT_DELTA;
                    }
                }
                DEFAULT_DELTA
            }
        }
    }

    fn render_simulation(&mut self, ui: &mut egui::Ui, dt: Scalar) {
        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(ui.available_width(), ui.available_height()),
            Sense::hover().union(Sense::click()),
        );

        let world_rect = Rect::from_min_max(
            Pos2::ZERO,
            Pos2::new(self.world_size.x as f32, self.world_size.y as f32),
        );
        let margin = 0.5 * (response.rect.size() - world_rect.size());
        let screen_rect = world_rect.translate(response.rect.min.to_vec2() + margin);
        let to_screen = emath::RectTransform::from_to(world_rect, screen_rect);
        let from_screen = to_screen.inverse();

        // if let Some(pos) = response.hover_pos() {
        // println!("H: {:?} {}", pos, response.hovered());
        // }
        if let Some(pos) = response.interact_pointer_pos() {
            // println!("I: {:?} {} {}", pos, response.clicked(), response.dragged());
            if response.clicked() {
                let pos = from_screen.transform_pos(pos);
                if let Some(cell_id) = self
                    .simulator
                    .get_cell_id_closer_to(pos.x as Scalar, pos.y as Scalar)
                {
                    self.selected_cell = Some(cell_id);
                }
            }
        }

        let size_fill_color = Rgba::from_white_alpha(0.0);
        let size_stroke_color = Rgba::from_white_alpha(0.1);
        let normal_color = Rgba::from_gray(1.0);
        let selected_color = Rgba::from_rgb(1.0, 0.7, 0.0);
        let velocity_color = Rgba::from_rgba_unmultiplied(0.0, 0.0, 1.0, 0.5);
        let acceleration_color = Rgba::from_rgba_unmultiplied(1.0, 0.0, 0.0, 0.3);
        for (cell_id, cell) in self.simulator.cells() {
            if let Some(object) = self.simulator.get_cell_object(cell_id) {
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
                let stroke_color = self
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

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        egui::TopBottomPanel::top("top")
            // .min_height(self.world_size.y as f32)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let play_pause = if self.paused { "Play" } else { "Pause" };
                    if ui.button(play_pause).clicked() {
                        self.paused = !self.paused;
                    }
                    ui.add(
                        egui::Slider::new(&mut self.speed, 1.0..=5.0)
                            .fixed_decimals(1)
                            .text("Speed"),
                    );
                    ui.separator();
                    if ui.button("Selection").clicked() {
                        println!("Selection")
                    }
                });
            });

        egui::TopBottomPanel::bottom("bottom")
            .min_height(200.0)
            .show(ctx, |ui| {
                // ui.horizontal(|ui| {
                //     let dt = self
                //         .last_update
                //         .map(|last_update| Instant::now().duration_since(last_update).as_secs_f64())
                //         .unwrap_or(0.0);
                //     ui.label("dt:");
                //     ui.label(format!("{:06}", dt));
                // });

                let scroll_area = ScrollArea::vertical().auto_shrink([false; 2]);
                scroll_area.show(ui, |ui| {
                    if let Some(cell_id) = self.selected_cell {
                        if let Some(cell) = self.simulator.get_cell_view(cell_id) {
                            ui.monospace(format!("{cell}"));
                        }
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Frame::canvas(ui.style())
                .inner_margin(0.0)
                .outer_margin(0.0)
                .fill(Rgba::from_rgb(0.0, 0.0, 0.0).into())
                .show(ui, |ui| {
                    let dt = self.update_simulation();
                    self.render_simulation(ui, dt);
                });
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
