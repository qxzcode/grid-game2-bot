pub mod game;
mod util;

use eframe::egui;
use eframe::egui::{Color32, Pos2, Rect, Rounding, Stroke};
use egui::epaint::QuadraticBezierShape;
use egui::{pos2, vec2, Shape, Vec2};
use game::GRID_RADIUS;
use hex2d::{Coordinate, Direction, Spacing, Spin};
use util::transforms::Transform;

const SQRT_3: f32 = 1.7320508;
const GRID_WIDTH_IN_SIDE_LENGTHS: f32 = SQRT_3 * (GRID_RADIUS * 2 + 1) as f32;
const GRID_HEIGHT_IN_SIDE_LENGTHS: f32 = 1.5 * (GRID_RADIUS * 2 + 1) as f32 + 0.5;

// plus visual padding in the GUI:
const GRID_WIDTH: f32 = GRID_WIDTH_IN_SIDE_LENGTHS + 2.0;
const GRID_HEIGHT: f32 = GRID_HEIGHT_IN_SIDE_LENGTHS + 2.0;

/// The corners of a hexagon with side length 1 that is centered at the origin.
/// The first corner is repeated at the end.
const HEXAGON_CORNERS: [Vec2; 7] = [
    vec2(0.0, 1.0),
    vec2(SQRT_3 / 2.0, 0.5),
    vec2(SQRT_3 / 2.0, -0.5),
    vec2(0.0, -1.0),
    vec2(-SQRT_3 / 2.0, -0.5),
    vec2(-SQRT_3 / 2.0, 0.5),
    vec2(0.0, 1.0),
];

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Grid Game",
        native_options,
        Box::new(|cc| Box::new(GridGameViewer::new(cc))),
    )
    .expect("eframe failed to start");
}

struct GridGameViewer {
    frames: Vec<()>,
    current_frame: usize,
    pointer_pos: String,
    zoom: f32,
    camera: Pos2,
}

impl Default for GridGameViewer {
    fn default() -> Self {
        Self {
            frames: vec![()],
            current_frame: 0,
            pointer_pos: "".to_string(),
            zoom: 1.0,
            camera: pos2(0.0, 0.0),
        }
    }
}

impl GridGameViewer {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn paint_game(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let origin = Coordinate::new(0, 0);

        let rect = ui.max_rect();
        let world_to_screen = Transform::new_letterboxed(
            Pos2::new(
                -GRID_WIDTH / 2.0 / self.zoom + self.camera.x,
                GRID_HEIGHT / 2.0 / self.zoom + self.camera.y,
            ),
            Pos2::new(
                GRID_WIDTH / 2.0 / self.zoom + self.camera.x,
                -GRID_HEIGHT / 2.0 / self.zoom + self.camera.y,
            ),
            Pos2::new(rect.left(), rect.top()),
            Pos2::new(rect.right(), rect.bottom()),
        );
        let painter = ui.painter_at(rect);

        ctx.input(|i| {
            self.zoom *= (i.scroll_delta.y / 500.0).exp();
            if i.pointer.is_decidedly_dragging() {
                let px_scale = world_to_screen.map_dist(1.0);
                self.camera.x -= i.pointer.delta().x / px_scale;
                self.camera.y += i.pointer.delta().y / px_scale;
            }
        });

        self.pointer_pos = match ctx.pointer_latest_pos() {
            None => "".to_string(),
            Some(pos) => {
                let pos = world_to_screen.inverse().map_point(pos);
                let tile: Coordinate<i32> =
                    Coordinate::from_pixel(pos.x, pos.y, Spacing::PointyTop(1.0));
                format!(
                    "({:.1}, {:.1}) Hexagon: ({}, {}, r={})",
                    pos.x,
                    pos.y,
                    tile.x,
                    tile.y,
                    tile.distance(origin),
                )
            }
        };

        // background
        painter.rect(
            Rect::from_two_pos(
                world_to_screen.map_point(Pos2::new(-GRID_WIDTH, -GRID_HEIGHT)),
                world_to_screen.map_point(Pos2::new(GRID_WIDTH, GRID_HEIGHT)),
            ),
            Rounding::ZERO,
            Color32::from_gray(10),
            Stroke::NONE,
        );

        // let game = self.frames[self.current_frame];

        let get_hex_center_corners = |tile: Coordinate| {
            let tile_center: Pos2 = tile.to_pixel(Spacing::PointyTop(1.0)).into();
            (
                tile_center,
                HEXAGON_CORNERS.map(|p| world_to_screen.map_point(tile_center + p)),
            )
        };

        // TODO draw game
        for r in 0..=GRID_RADIUS {
            let ring = origin.ring_iter(r as i32, Spin::CW(Direction::XY));

            for tile in ring {
                let (_, tile_corners) = get_hex_center_corners(tile);
                painter.add(Shape::convex_polygon(
                    tile_corners.to_vec(),
                    match r {
                        0 => Color32::from_rgba_unmultiplied(255, 128, 0, 15),
                        GRID_RADIUS => Color32::from_white_alpha(5),
                        _ => Color32::TRANSPARENT,
                    },
                    Stroke::new(1.0, Color32::from_gray(100)),
                ));
            }
        }

        let tile = Coordinate::new(1, -3);
        let tile_center: Pos2 = tile.to_pixel(Spacing::PointyTop(1.0)).into();
        let edge_midpoint = |edge_index| {
            let c1 = tile_center + HEXAGON_CORNERS[edge_index];
            let c2 = tile_center + HEXAGON_CORNERS[edge_index + 1];
            world_to_screen.map_point(c1 + (c2 - c1) / 2.0)
        };
        painter.add(egui::Shape::QuadraticBezier(QuadraticBezierShape {
            points: [
                edge_midpoint(2),
                world_to_screen.map_point(tile_center),
                edge_midpoint(3),
            ],
            closed: false,
            fill: Color32::TRANSPARENT,
            stroke: Stroke::new(1.0, Color32::RED),
        }));
    }
}

impl eframe::App for GridGameViewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.label(format!("Turn: {}", self.current_frame));
                    if ui
                        .add_enabled(self.current_frame != 0, egui::Button::new("<<"))
                        .clicked()
                    {
                        self.current_frame = 0;
                    }
                    if ui
                        .add_enabled(self.current_frame != 0, egui::Button::new("<"))
                        .clicked()
                    {
                        self.current_frame -= 1;
                    }
                    if ui.button("Reset Game").clicked() {
                        self.frames = vec![()];
                        self.current_frame = 0;
                    }
                    if ui
                        .add_enabled(
                            self.current_frame != self.frames.len() - 1,
                            // || self.frames[self.current_frame].game_winner().is_none(),
                            egui::Button::new(">"),
                        )
                        .clicked()
                    {
                        if self.current_frame == self.frames.len() - 1 {
                            // let game = self.frames[self.current_frame];
                            // TODO
                            // self.frames.push(game);
                        }
                        self.current_frame += 1;
                    }
                    if ui
                        .add_enabled(
                            self.current_frame != self.frames.len() - 1,
                            egui::Button::new(">>"),
                        )
                        .clicked()
                    {
                        todo!();
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(&self.pointer_pos);
                });
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| self.paint_game(ctx, ui));
    }
}
