use eframe::emath::RectTransform;
use eframe::{
    egui::{self, emath, Pos2, Rect, ScrollArea, Sense},
    epaint::{CircleShape, Rgba, Shape},
};
use std::time::Instant;

use vlife_physics::{Scalar, Vec2};
use vlife_simulator::{cell, CellId, Simulator};

const NUM_INITIAL_CELLS: usize = 500;

pub(crate) struct Application {
    world_size: Vec2,
    simulator: Simulator,
    last_update: Option<Instant>,
    selected_cell: Option<CellId>,
}

impl Application {
    pub fn new(world_size: Vec2) -> Self {
        let simulator = Self::create_simulator(world_size);
        Self {
            world_size,
            simulator,
            last_update: None,
            selected_cell: Some(0),
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
                let dt = 1.0 / 60.0;
                self.simulator.update(dt);
                dt
            }
        }
    }

    fn render_simulation(&mut self, ui: &mut egui::Ui, dt: Scalar) {
        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(ui.available_width(), ui.available_height()),
            Sense::hover().union(Sense::click()),
        );

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_max(
                Pos2::ZERO,
                Pos2::new(self.world_size.x as f32, self.world_size.y as f32),
            ),
            response.rect,
        );
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
                let object_position = object.position();
                let object_velocity = object.velocity();
                let object_velocity_p1 =
                    object_position + object_velocity.normalize() * object.radius();
                let object_velocity_p2 = object_velocity_p1 + object_velocity;
                let object_acceleration = cell.movement_velocity();
                let object_acceleration_p1 =
                    object_position + object_acceleration.normalize() * object.radius();
                let object_acceleration_p2 = object_acceleration_p1 + object_acceleration;
                let energy = (cell.energy() / cell::MAX_ENERGY).min(1.0) as f32;
                let energy_loss = ((-cell.energy_delta()).max(0.0) / dt).min(1.0) as f32;
                let energy_gain = (cell.energy_delta().max(0.0) / dt).min(1.0) as f32;
                let fill_color = Rgba::from_rgb(energy_loss, energy, energy_gain);
                let stroke_color = self
                    .selected_cell
                    .filter(|selected_id| *selected_id == cell_id)
                    .map_or(normal_color, |_| selected_color);

                painter.add(CircleShape {
                    center: object_position.transform_pos(&to_screen),
                    radius: cell.size() as f32,
                    fill: size_fill_color.into(),
                    stroke: (1.0, size_stroke_color).into(),
                });
                painter.add(CircleShape {
                    center: object_position.transform_pos(&to_screen),
                    radius: cell.contracted_size() as f32,
                    fill: fill_color.into(),
                    stroke: (1.0, stroke_color).into(),
                });
                painter.add(Shape::line_segment(
                    [
                        object_velocity_p1.transform_pos(&to_screen),
                        object_velocity_p2.transform_pos(&to_screen),
                    ],
                    (1.0, velocity_color),
                ));
                painter.add(Shape::line_segment(
                    [
                        object_acceleration_p1.transform_pos(&to_screen),
                        object_acceleration_p2.transform_pos(&to_screen),
                    ],
                    (1.0, acceleration_color),
                ));
            }
        }
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();

        egui::TopBottomPanel::top("top")
            .min_height(self.world_size.y as f32)
            .show(ctx, |ui| {
                egui::Frame::canvas(ui.style())
                    .inner_margin(0.0)
                    .outer_margin(0.0)
                    .fill(Rgba::from_rgb(0.0, 0.0, 0.0).into())
                    .show(ui, |ui| {
                        let dt = self.update_simulation();
                        self.render_simulation(ui, dt);
                    });
            });

        egui::TopBottomPanel::bottom("bottom")
            .min_height(400.0)
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

        // egui::CentralPanel::default()
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
