pub mod game;
mod util;

use std::collections::HashSet;

use eframe::egui;
use eframe::egui::{Color32, Pos2, Rounding, Stroke};
use egui::epaint::QuadraticBezierShape;
use egui::{pos2, vec2, Frame, Shape, Vec2};
use game::GRID_RADIUS;
use hex2d::{Coordinate, Direction, Spacing, Spin};
use rand::seq::SliceRandom;
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
    draw_lines: bool,
    highlight_edges: bool,
}

impl Default for GridGameViewer {
    fn default() -> Self {
        Self {
            frames: vec![()],
            current_frame: 0,
            pointer_pos: "".to_string(),
            zoom: 1.0,
            camera: pos2(0.0, 0.0),
            draw_lines: true,
            highlight_edges: true,
        }
    }
}

impl GridGameViewer {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    /// Makes the letterboxing transform given the current camera parameters.
    fn make_transform(&self, ui: &egui::Ui) -> Transform {
        let ui_rect = ui.max_rect();
        Transform::new_letterboxed(
            Pos2::new(
                -GRID_WIDTH / 2.0 / self.zoom + self.camera.x,
                GRID_HEIGHT / 2.0 / self.zoom + self.camera.y,
            ),
            Pos2::new(
                GRID_WIDTH / 2.0 / self.zoom + self.camera.x,
                -GRID_HEIGHT / 2.0 / self.zoom + self.camera.y,
            ),
            Pos2::new(ui_rect.left(), ui_rect.top()),
            Pos2::new(ui_rect.right(), ui_rect.bottom()),
        )
    }

    fn paint_game(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        let origin = Coordinate::new(0, 0);

        ctx.input(|i| {
            self.zoom *= (i.scroll_delta.y / 500.0).exp();
            self.zoom = self.zoom.clamp(1.0, 1.0e4);
            if i.pointer.is_decidedly_dragging() {
                let px_scale = self.make_transform(ui).map_dist(1.0);
                self.camera.x -= i.pointer.delta().x / px_scale;
                self.camera.y += i.pointer.delta().y / px_scale;
                self.camera = self.camera.clamp(
                    pos2(-GRID_WIDTH / 2.0, -GRID_HEIGHT / 2.0),
                    pos2(GRID_WIDTH / 2.0, GRID_HEIGHT / 2.0),
                );
            }
        });

        let ui_rect = ui.max_rect();
        let world_to_screen = self.make_transform(ui);
        let painter = ui.painter_at(ui_rect);

        self.pointer_pos = match ctx.pointer_latest_pos() {
            None => "".to_string(),
            Some(pos) => {
                let pos = world_to_screen.inverse().map_point(pos);
                let tile: Coordinate<i32> =
                    Coordinate::from_pixel(pos.x, pos.y, Spacing::PointyTop(1.0));
                format!(
                    "({:.1}, {:.1}) Hexagon: (x={}, y={}, z={}, r={})",
                    pos.x,
                    pos.y,
                    tile.x,
                    tile.y,
                    tile.z(),
                    tile.distance(origin),
                )
            }
        };

        // background
        painter.rect(
            ui_rect,
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
                        GRID_RADIUS => Color32::WHITE.gamma_multiply(0.05),
                        _ => Color32::TRANSPARENT,
                    },
                    Stroke::new(0.5, Color32::from_gray(50)),
                ));
            }
        }

        let mut occupied_edges = HashSet::new();
        for color in [Color32::RED, Color32::GREEN, Color32::YELLOW] {
            let mut last_tile = Coordinate::new(10, -3);
            let mut last_edge = None;
            for _ in 0..100 {
                // Get the next tile in the path.
                let tile = *last_tile
                    .neighbors()
                    .choose(&mut rand::thread_rng())
                    .unwrap();

                let tile_center = |tile: Coordinate| tile.to_pixel(Spacing::PointyTop(1.0)).into();
                let edge_endpoints = |edge_index| {
                    let e1 = tile_center(tile) + HEXAGON_CORNERS[edge_index];
                    let e2 = tile_center(tile) + HEXAGON_CORNERS[edge_index + 1];
                    [e1, e2].map(|e| world_to_screen.map_point(e))
                };

                let [e1, e2] = edge_endpoints(match tile.direction_to_cw(last_tile).unwrap() {
                    Direction::ZY => 0,
                    Direction::XY => 1,
                    Direction::XZ => 2,
                    Direction::YZ => 3,
                    Direction::YX => 4,
                    Direction::ZX => 5,
                });
                let edge = e1 + (e2 - e1) / 2.0;
                if last_edge == Some(edge)
                    || !occupied_edges.insert((tile.min(last_tile), tile.max(last_tile)))
                {
                    // for debug, to prevent illegal random moves
                    continue;
                }
                assert_ne!(last_edge, Some(edge)); // Verify that this isn't a 180deg turn.

                if self.highlight_edges {
                    // Redraw the hexagon edge to show that it is now off-limits.
                    painter.line_segment([e1, e2], Stroke::new(1.0, Color32::WHITE));
                }

                if self.draw_lines {
                    // Draw the curved segment of the player's line.
                    if let Some(last_edge) = last_edge {
                        painter.add(egui::Shape::QuadraticBezier(QuadraticBezierShape {
                            points: [
                                last_edge,
                                world_to_screen.map_point(tile_center(last_tile)),
                                edge,
                            ],
                            closed: false,
                            fill: Color32::TRANSPARENT,
                            stroke: Stroke::new(1.5, color.gamma_multiply(0.4)),
                        }));
                    }
                }

                last_tile = tile;
                last_edge = Some(edge);
            }

            // Draw the end-of-line marker.
            if let Some(last_edge) = last_edge {
                painter.circle_filled(last_edge, world_to_screen.map_dist(0.25), color);
            }
        }
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

                    ui.separator();

                    ui.checkbox(&mut self.draw_lines, "Draw lines");
                    ui.checkbox(&mut self.highlight_edges, "Highlight edges");
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(&self.pointer_pos);
                });
            });
        });
        egui::CentralPanel::default()
            .frame(Frame::default().inner_margin(0.0))
            .show(ctx, |ui| self.paint_game(ctx, ui));
    }
}
