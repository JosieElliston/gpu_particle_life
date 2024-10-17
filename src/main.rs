// mod boids_example_gfx;

mod color_drag_value;
mod gfx;
mod my_utils;

use color_drag_value::ColorDragValue;
use eframe::egui::{self, Rect, Vec2};
use gfx::GfxData;
use rand::prelude::*;

fn main() -> eframe::Result {
    // std::env::set_var("RUST_BACKTRACE", "1");
    // env_logger::init();

    let native_options = eframe::NativeOptions::default();

    eframe::run_native(
        "particle life",
        native_options,
        // Box::new(|cc| Ok(Box::new(App::new(cc, 6, 100)))),
        Box::new(|cc| Ok(Box::new(App::new(cc, 6, 5000)))),
    )
}

// TODO: not square simulation window
struct ViewSettings {
    particle_radius: f32,
    texture_size: u32,
    zoom_scale: f32,
    zoom_center: Vec2,
    specie_colors: Vec<egui::Rgba>,
}
impl ViewSettings {
    fn new(specie_n: usize, _particle_n: usize) -> Self {
        const INITIAL_TEXTURE_SIZE: u32 = 100;
        Self {
            particle_radius: 0.002, // TODO: this should vary with particle n
            // particle_radius: 0.05,
            texture_size: INITIAL_TEXTURE_SIZE,
            zoom_scale: 1.0,
            zoom_center: Vec2::new(0.5, 0.5),
            specie_colors: [
                egui::Color32::RED,
                egui::Color32::GREEN,
                egui::Color32::BLUE,
                egui::Color32::YELLOW,
                // egui::Color32::LIGHT_RED,
                egui::Color32::ORANGE,
                egui::Color32::LIGHT_GREEN,
                egui::Color32::LIGHT_BLUE,
                egui::Color32::LIGHT_YELLOW,
                egui::Color32::DARK_RED,
                egui::Color32::DARK_GREEN,
                egui::Color32::DARK_BLUE,
                egui::Color32::GRAY,
            ][..specie_n]
                .iter()
                .copied()
                .map(egui::Rgba::from)
                .collect::<Vec<_>>(),
            // specie_colors: (0..specie_n)
            //     .map(|specie_i| color_interpolation::get_color(specie_n, specie_i))
            //     .collect(),
        }
    }

    // fn get_color(specie_n: usize, specie_i: usize) -> egui::Rgba {
    //     let x = 2.0 * std::f32::consts::PI * specie_i as f32 / specie_n as f32;
    //     egui::Rgba::from_rgb(x.cos(), (x - 2.0).cos(), (x - 4.0).cos())
    //     egui::ecolor::Hsva::new(specie_i as f32 / specie_n as f32, 1.0, 0.9, 1.0).into()
    // }
}

struct SimSettings {
    substep_n: usize,
    specie_n: usize,
    particle_n: usize,
    local_radius: f32,
    friction_half_life: f32,
    time_scale: f32,
    attractions: Vec<Vec<f32>>,
    dt: f32,
}
impl SimSettings {
    fn new(specie_n: usize, particle_n: usize) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            substep_n: 8,
            specie_n,
            particle_n,
            // TODO: vary with particle_n
            // TODO: if i use grid optimization, should be enforced that this is of the form 1/n for some n, it's probably incorrect near the right/bottom if it's not
            local_radius: 0.1,
            friction_half_life: 0.04,
            time_scale: 1.0,
            attractions: (0..specie_n)
                .map(|_| (0..specie_n).map(|_| rng.gen_range(-1.0..=1.0)).collect())
                .collect(),
            dt: 0.01,
        }
    }
}

// stuff that should live on the gpu in the future
// stuff that changes each tick
// stuff that of dynamic size
// TODO: wgsl likes "normalized device coordinates" which are in [-1.0, 1.0] instead of [0, 1.0], but also it's cool how its ambiguous over whether it's y-down
// stuff that gets sent to the gpu on initialization but never anytime else
#[derive(Debug)]
struct SimData {
    poses: Vec<Vec2>,
    vels: Vec<Vec2>,
    species: Vec<u32>,
}
impl SimData {
    fn new(specie_n: usize, particle_n: usize) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            poses: (0..particle_n)
                .map(|_| Vec2 {
                    x: rng.gen_range(0.0..1.0),
                    y: rng.gen_range(0.0..1.0),
                })
                .collect(),
            vels: (0..particle_n)
                .map(|_| Vec2 {
                    x: rng.gen_range(-0.1..=0.1),
                    y: rng.gen_range(-0.1..=0.1),
                })
                .collect(),
            species: (0..particle_n)
                .map(|_| rng.gen_range(0..specie_n as _))
                .collect(),
        }
    }
}

struct App {
    view_settings: ViewSettings,
    sim_settings: SimSettings,
    gfx_data: GfxData,
}
impl App {
    fn new(cc: &eframe::CreationContext<'_>, specie_n: usize, particle_n: usize) -> Self {
        let view_settings = ViewSettings::new(specie_n, particle_n);
        let sim_settings = SimSettings::new(specie_n, particle_n);
        let sim_data = SimData::new(specie_n, particle_n);
        let gfx_data = GfxData::new(cc, &view_settings, &sim_settings, &sim_data);
        Self {
            view_settings,
            sim_settings,
            gfx_data,
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint();
        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                let dt = ctx.input(|input_state| input_state.stable_dt);
                // println!("dt: {:?}", dt);
                self.sim_settings.dt = dt;

                let scale = ui.available_rect_before_wrap().size().min_elem();
                let rect = egui::Rect::from_min_size(
                    ui.available_rect_before_wrap().min,
                    Vec2::new(scale, scale),
                );
                self.view_settings.texture_size = 2 * scale as u32;
                // 2 * because something (maybe at the os level?) does antialiasing better with that
                // let mut unit = scale / (2. * self.view_settings.zoom_scale); // TODO: what is this

                // pan and zoom
                // let r: egui::Response = ui.interact(
                //     rect,
                //     eframe::egui::Id::new("drawing"),
                //     egui::Sense::click_and_drag(),
                // );

                // if r.hovered() {
                //     let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y / unit);
                //     if scroll_delta.abs() > 0.001 {
                //         self.view_settings.zoom_scale = (self.view_settings.zoom_scale - scroll_delta).max(0.1);
                //         unit = scale / (2. * self.view_settings.zoom_scale);
                //     }
                // }

                // let scale = egui_rect.size() / (1. * egui_rect.size().min_elem());
                // let scale = [scale.x * self.scale, scale.y * self.scale];

                // let screen_to_egui =
                //     |pos: Pos| pos2(pos.x as f32, -pos.y as f32) * unit + cen.to_vec2();
                // let egui_to_screen = |pos: Pos2| {
                //     let pos = (pos - cen.to_vec2()) / unit;
                //     Pos {
                //         x: pos.x as f64,
                //         y: -pos.y as f64,
                //     }
                // };

                // if r.dragged_by(egui::PointerButton::Secondary) && r.drag_delta().length() > 0.1 {
                //     if let Some(mpos) = r.interact_pointer_pos() {
                //         let egui_to_geom = |pos: Pos2| {
                //             let Pos { x, y } = egui_to_screen(pos);
                //             cga2d::point(x, y)
                //         };
                //         let root_pos = egui_to_geom(mpos - r.drag_delta());
                //         let end_pos = egui_to_geom(mpos);

                //         let modifiers = ctx.input(|i| i.modifiers);

                //         let ms: Vec<cga2d::Blade3> = self
                //             .tiling
                //             .mirrors
                //             .iter()
                //             .map(|&m| self.camera_transform.sandwich(m))
                //             .collect();
                //         let boundary = match (modifiers.command, modifiers.alt) {
                //             (true, false) => {
                //                 let third = if self.tiling.rank == 4 {
                //                     !ms[3]
                //                 } else {
                //                     !(!ms[0] ^ !ms[1] ^ !ms[2])
                //                 };
                //                 !ms[1] ^ !ms[2] ^ third
                //             }
                //             (false, true) => {
                //                 let third = if self.tiling.rank == 4 {
                //                     !ms[3]
                //                 } else {
                //                     !(!ms[0] ^ !ms[1] ^ !ms[2])
                //                 };
                //                 !ms[0] ^ !ms[1] ^ third
                //             }
                //             _ => !ms[0] ^ !ms[1] ^ !ms[2],
                //         }; // the boundary to fix when transforming space

                //         let init_refl = !(root_pos ^ end_pos) ^ !boundary; // get root_pos to end_pos
                //         let f = end_pos ^ !boundary;
                //         let final_refl = !(!init_refl ^ f) ^ f; // restore orientation fixing the "straight line" from root_pos to end_pos

                //         self.camera_transform =
                //             (final_refl * init_refl * self.camera_transform).normalize();
                //     }
                // }

                // wgpu stuff
                self.gfx_data
                    .render(&self.view_settings, &self.sim_settings);

                egui::widgets::Image::from_texture(egui::load::SizedTexture::new(
                    self.gfx_data.texture_id,
                    Vec2::new(10.0, 10.0), // arbitrary size
                ))
                .paint_at(ui, rect);

                // settings ui
                // TODO: make the window thing go on the right
                // TODO: change particle_n, species_n and regenerate
                egui::Frame::popup(ui.style())
                    .outer_margin(10.)
                    .shadow(egui::Shadow::NONE)
                    // .stroke(egui::Stroke::NONE)
                    .show(ui, |ui| {
                        egui::CollapsingHeader::new("settings").show(ui, |ui| {
                            ui.collapsing("view_settings", |ui| {
                                ui.horizontal(|ui| {
                                    ui.add(
                                        egui::Slider::new(
                                            &mut self.view_settings.particle_radius,
                                            0.0..=0.01,
                                        )
                                        .clamping(egui::SliderClamping::Never)
                                        .text("particle_radius"), // .logarithmic(true),
                                    );
                                });
                                ui.horizontal(|ui| {
                                    ui.add(
                                        egui::Slider::new(
                                            &mut self.view_settings.zoom_scale,
                                            1.0..=10.0,
                                        )
                                        .clamping(egui::SliderClamping::Never)
                                        .text("zoom_scale"),
                                    );
                                });
                                ui.horizontal(|ui| {
                                    ui.add(
                                        egui::Slider::new(
                                            &mut self.view_settings.zoom_center.x,
                                            0.0..=1.0,
                                        )
                                        .clamping(egui::SliderClamping::Never)
                                        .text("zoom_center.x"),
                                    );
                                });
                                ui.horizontal(|ui| {
                                    ui.add(
                                        egui::Slider::new(
                                            &mut self.view_settings.zoom_center.y,
                                            0.0..=1.0,
                                        )
                                        .clamping(egui::SliderClamping::Never)
                                        .text("zoom_center.y"),
                                    );
                                });
                                ui.horizontal(|ui| {
                                    for color in self.view_settings.specie_colors.iter_mut() {
                                        let mut c = [color.r(), color.g(), color.b()];
                                        ui.color_edit_button_rgb(&mut c);
                                        *color = egui::Rgba::from_rgb(c[0], c[1], c[2]);
                                    }
                                })
                            });
                            ui.collapsing("sim_settings", |ui| {
                                ui.horizontal(|ui| {
                                    ui.add(
                                        egui::Slider::new(&mut self.sim_settings.substep_n, 1..=16)
                                            .clamping(egui::SliderClamping::Never)
                                            .text("substep_n"),
                                    );
                                });
                                ui.horizontal(|ui| {
                                    ui.add(
                                        egui::Slider::new(
                                            &mut self.sim_settings.local_radius,
                                            0.0..=0.2,
                                        )
                                        .clamping(egui::SliderClamping::Never)
                                        .text("local_radius"),
                                    );
                                });
                                ui.horizontal(|ui| {
                                    ui.add(
                                        egui::Slider::new(
                                            &mut self.sim_settings.friction_half_life,
                                            0.0..=1.0,
                                        )
                                        .clamping(egui::SliderClamping::Never)
                                        .text("friction_half_life")
                                        .logarithmic(true),
                                    );
                                });
                                ui.horizontal(|ui| {
                                    ui.add(
                                        egui::Slider::new(
                                            &mut self.sim_settings.time_scale,
                                            0.0..=1.0,
                                        )
                                        .clamping(egui::SliderClamping::Never)
                                        .text("time_scale"),
                                    );
                                });
                                ui.collapsing("attractions", |ui| {
                                    if ui.button("randomize").clicked() {
                                        let mut rng = thread_rng();
                                        self.sim_settings.attractions =
                                            (0..self.sim_settings.specie_n)
                                                .map(|_| {
                                                    (0..self.sim_settings.specie_n)
                                                        .map(|_| rng.gen_range(-1.0..=1.0))
                                                        .collect()
                                                })
                                                .collect();
                                    }
                                    ui.horizontal(|ui| {
                                        ui.add(
                                            egui::Button::new("")
                                                .min_size(ui.spacing().interact_size), // this is how drag values work
                                        );
                                        for col in 0..self.sim_settings.specie_n {
                                            ui.add(
                                                egui::Button::new("")
                                                    .min_size(ui.spacing().interact_size)
                                                    .fill(self.view_settings.specie_colors[col]),
                                            );
                                        }
                                    });
                                    for row in 0..self.sim_settings.specie_n {
                                        ui.horizontal(|ui| {
                                            ui.add(
                                                egui::Button::new("")
                                                    .min_size(ui.spacing().interact_size)
                                                    .fill(self.view_settings.specie_colors[row]),
                                            );
                                            for col in 0..self.sim_settings.specie_n {
                                                ui.add(
                                                    ColorDragValue::new(
                                                        &mut self.sim_settings.attractions[row]
                                                            [col],
                                                    )
                                                    .range(-1.0..=1.0)
                                                    .clamp_existing_to_range(false)
                                                    .update_while_editing(false)
                                                    .speed(0.02)
                                                    .fixed_decimals(2),
                                                );
                                            }
                                        });
                                    }
                                });
                            });
                        });
                    });
            });
    }
}
