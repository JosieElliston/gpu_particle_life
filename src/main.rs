// mod boids_example_gfx;

mod gfx;
mod my_utils;

use eframe::egui::{self, Rect, Vec2};
use gfx::GfxData;
use rand::prelude::*;

fn main() -> eframe::Result {
    // std::env::set_var("RUST_BACKTRACE", "1");
    // env_logger::init();

    let native_options = eframe::NativeOptions {
        follow_system_theme: false,
        // renderer: eframe::Renderer::Wgpu, // TODO: it says to do this on crates.io
        ..Default::default()
    };

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
                egui::Color32::LIGHT_RED,
                egui::Color32::LIGHT_GREEN,
                egui::Color32::LIGHT_BLUE,
                egui::Color32::LIGHT_YELLOW,
                egui::Color32::DARK_RED,
                egui::Color32::DARK_GREEN,
                egui::Color32::DARK_BLUE,
                egui::Color32::GRAY,
            ][..specie_n]
                .iter()
                .map(|color| {
                    egui::Rgba::from_srgba_premultiplied(color.r(), color.g(), color.b(), color.a())
                })
                .collect::<Vec<_>>(),
        }
    }
}

struct SimSettings {
    substep_n: usize,
    specie_n: usize,
    particle_n: usize,
    local_radius: f32,
    friction_half_life: f32,
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
                println!("dt: {:?}", dt);

                let scale = ui.available_rect_before_wrap().size().min_elem();
                let rect = egui::Rect::from_min_size(
                    ui.available_rect_before_wrap().min,
                    Vec2::new(scale, scale),
                );

                self.view_settings.texture_size = 2 * scale as u32;
                self.sim_settings.dt = dt;

                self.gfx_data
                    .render(&self.view_settings, &self.sim_settings);

                egui::widgets::Image::from_texture(egui::load::SizedTexture::new(
                    self.gfx_data.texture_id,
                    Vec2::new(10., 10.),
                ))
                .paint_at(ui, rect);

                // settings
                // struct ViewSettings {
                //     particle_radius: f32,
                //     texture_size: u32,
                //     zoom_scale: f32,
                //     zoom_center: Vec2,
                //     specie_colors: Vec<egui::Rgba>,
                // }
                // struct SimSettings {
                //     substep_n: usize,
                //     specie_n: usize,
                //     particle_n: usize,
                //     local_radius: f32,
                //     friction_half_life: f32,
                //     attractions: Vec<Vec<f32>>,
                //     dt: f32,
                // }
                egui::Frame::popup(ui.style())
                    .outer_margin(10.)
                    .shadow(egui::Shadow::NONE)
                    // .stroke(egui::Stroke::NONE)
                    .show(ui, |ui| {
                        egui::CollapsingHeader::new("settings").show(ui, |ui| {
                            ui.collapsing("view settings", |ui| {
                                ui.horizontal(|ui| {
                                    ui.add(egui::Slider::new(
                                        &mut self.view_settings.particle_radius,
                                        0.0..=0.01,
                                    ));
                                });
                            });
                            ui.collapsing("sim settings", |ui| {
                                ui.horizontal(|ui| {
                                    ui.add(egui::Slider::new(
                                        &mut self.sim_settings.substep_n,
                                        1..=16,
                                    ));
                                });
                                ui.horizontal(|ui| {
                                    ui.add(egui::Slider::new(
                                        &mut self.sim_settings.local_radius,
                                        0.0..=0.2,
                                    ));
                                });
                                ui.horizontal(|ui| {
                                    ui.add(egui::Slider::new(
                                        &mut self.sim_settings.friction_half_life,
                                        0.0..=0.1,
                                    ));
                                });
                                ui.collapsing("attractions", |ui| {
                                    for row in 0..self.sim_settings.specie_n {
                                        ui.horizontal(|ui| {
                                            // ui.painter().add(Rect::);
                                            for col in 0..self.sim_settings.specie_n {
                                                // ui.add(egui::Slider::new(
                                                //     &mut self.sim_settings.attractions[row][col],
                                                //     -2.0..=2.0,
                                                // ));
                                                ui.add(
                                                    egui::widgets::DragValue::new(
                                                        &mut self.sim_settings.attractions[row]
                                                            [col],
                                                    )
                                                    .range(-1.0..=1.0)
                                                    .update_while_editing(false),
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
