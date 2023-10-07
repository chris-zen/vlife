use eframe::egui::{self, ScrollArea};
use std::time::Instant;

use vlife_simulator::{CellHandle, Real};
use vlife_simulator::{Simulator, Vec2};

use crate::central_panel::CentralPanel;
use crate::top_bar::TopBar;

const NUM_INITIAL_CELLS: usize = 500;

const DEFAULT_DELTA: Real = 1.0 / 60.0; // 60 Hz

pub(crate) struct Application {
    last_update: Option<Instant>,
    frame_time: f64,
    frame_count: usize,
    step_count: usize,
    pub(crate) frames_per_second: f64,
    pub(crate) steps_per_second: f64,
    pub(crate) time_ratio: f64,
    pub(crate) world_size: Vec2,
    pub(crate) simulator: Simulator,
    pub(crate) selected_cell: Option<CellHandle>,
    pub(crate) paused: bool,
    pub(crate) speed: f32,
}

impl Application {
    pub fn new(world_size: Vec2) -> Self {
        let simulator = Self::create_simulator(world_size);
        Self {
            last_update: None,
            frame_time: 0.0,
            frame_count: 0,
            step_count: 0,
            frames_per_second: 0.0,
            steps_per_second: 0.0,
            time_ratio: 1.0,
            world_size,
            simulator,
            selected_cell: None,
            paused: false,
            speed: 1.0,
        }
    }

    fn ui(&mut self, ctx: &egui::Context) {
        TopBar::ui(ctx, self);

        egui::TopBottomPanel::bottom("bottom")
            .min_height(300.0)
            .show(ctx, |ui| {
                let scroll_area = ScrollArea::vertical().auto_shrink([false; 2]);
                scroll_area.show(ui, |ui| {
                    if let Some(cell_id) = self.selected_cell {
                        // if let Some(cell) = self.simulator.get_cell_view(cell_id) {
                        //     ui.monospace(format!("{cell}"));
                        // }
                    }
                });
            });

        let dt = self.update_simulation();

        CentralPanel::ui(ctx, self, dt);
    }

    fn create_simulator(world_size: Vec2) -> Simulator {
        let mut simulator = Simulator::new(world_size);
        for _ in 0..1 {
            simulator.create_random_cell();
        }
        simulator
    }

    fn update_simulation(&mut self) -> Real {
        self.update_frames_per_second();
        if !self.paused {
            self.advance_simulation()
        } else {
            0.0
        }
    }

    fn update_frames_per_second(&mut self) {
        match self.last_update {
            None => {
                self.last_update = Some(Instant::now());
                self.frame_count = 0;
                self.step_count = 0;
            }
            Some(last_update) => {
                self.last_update = Some(Instant::now());
                self.frame_time += Instant::now().duration_since(last_update).as_secs_f64();
                self.frame_count += 1;
                if self.frame_time >= 1.0 {
                    let frame_time_recip = self.frame_time.recip();
                    self.frames_per_second = self.frame_count as f64 * frame_time_recip;
                    self.steps_per_second = self.step_count as f64 * frame_time_recip;
                    self.time_ratio = self.step_count as f64 * DEFAULT_DELTA * frame_time_recip;
                    self.frame_time = 0.0;
                    self.frame_count = 0;
                    self.step_count = 0;
                }
            }
        }
    }

    fn advance_simulation(&mut self) -> Real {
        let mut time = 0.0;
        let dt = self.simulator.step_time();
        let total_time = self.speed as Real * dt;
        while time <= total_time {
            self.step_count += 1;
            self.simulator.update();
            time += dt;
        }
        total_time
    }

    pub(crate) fn on_cell_selected(&mut self, cell_handle: CellHandle) {
        self.selected_cell = Some(cell_handle)
    }

    pub(crate) fn on_pause_play_button(&mut self) {
        self.paused = !self.paused;
        self.steps_per_second = 0.0;
        self.step_count = 0;
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        self.ui(ctx);
    }
}
