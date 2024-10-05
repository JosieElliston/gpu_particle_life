// mod boids_example_gfx;

mod gfx;
mod my_utils;

use eframe::egui::{self, Vec2};
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

struct ViewSettings {
    particle_radius: f32,
    texture_size: u32,
    zoom_scale: f32,
    zoom_center: Vec2,
}
impl ViewSettings {
    fn new(_specie_n: usize, _particle_n: usize) -> Self {
        const INITIAL_TEXTURE_SIZE: u32 = 100;
        Self {
            particle_radius: 0.002, // TODO: this should vary with particle n
            // particle_radius: 0.05,
            texture_size: INITIAL_TEXTURE_SIZE,
            zoom_scale: 1.0,
            zoom_center: Vec2::new(0.5, 0.5),
        }
    }
}

struct SimSettings {
    substep_n: usize,
    specie_n: usize,
    particle_n: usize,
    local_radius: f32,
    friction_half_life: f32,
    dt: f32,
}
impl SimSettings {
    fn new(specie_n: usize, particle_n: usize) -> Self {
        Self {
            substep_n: 8,
            specie_n,
            particle_n,
            // TODO: vary with particle_n
            // TODO: if i use grid optimization, should be enforced that this is of the form 1/n for some n, it's probably incorrect near the right/bottom if it's not
            local_radius: 0.1,
            // local_radius: 0.05,
            friction_half_life: 0.04,
            dt: 0.01,
        }
    }
}

// stuff that should live on the gpu in the future
// stuff that changes each tick
// stuff that of dynamic size
// TODO: wgsl likes "normalized device coordinates" which are in [-1.0, 1.0] instead of [0, 1.0], but also it's cool how its ambiguous over whether it's y-down
#[derive(Debug)]
struct SimData {
    poses: Vec<Vec2>,
    vels: Vec<Vec2>,
    forces: Vec<Vec2>,
    species: Vec<u32>,
    attractions: Vec<Vec<f32>>,
    specie_colors: Vec<egui::Rgba>,
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
            forces: vec![Vec2::new(0.0, 0.0); particle_n],
            species: (0..particle_n)
                .map(|_| rng.gen_range(0..specie_n as _))
                .collect(),
            attractions: (0..specie_n)
                .map(|_| (0..specie_n).map(|_| rng.gen_range(-1.0..=1.0)).collect())
                .collect(),
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

    fn compute_force(&self, i: usize, sim_settings: &SimSettings, force: &mut Vec2) {
        let mut force_temp = Vec2::new(0.0, 0.0);
        // let position = *unsafe { self.poses.get_unchecked(i) };
        let position = self.poses[i];
        // for neighbor_i in self.spacial_partition.get_neighbors(&position) {
        for neighbor_i in 0..self.poses.len() {
            // let neighbor_i = *neighbor_i;
            if i == neighbor_i {
                continue;
            }
            // let neighbor_position: Position = *unsafe { self.poses.get_unchecked(neighbor_i) };
            let neighbor_position = self.poses[neighbor_i];
            let mut to_neighbor = neighbor_position - position;
            if to_neighbor.x > 0.5 {
                to_neighbor.x -= 1.0;
            } else if to_neighbor.x < -0.5 {
                to_neighbor.x += 1.0;
            }
            if to_neighbor.y > 0.5 {
                to_neighbor.y -= 1.0;
            } else if to_neighbor.y < -0.5 {
                to_neighbor.y += 1.0;
            }
            let distance2 = to_neighbor.length_sq();
            if distance2 > sim_settings.local_radius * sim_settings.local_radius {
                continue;
            }
            if distance2 == 0.0 {
                continue;
            }
            let distance = distance2.sqrt();
            force_temp += (to_neighbor / distance)
                * get_attraction_force(
                    distance / sim_settings.local_radius,
                    self.attractions[self.species[i] as usize][self.species[neighbor_i] as usize],
                );
        }
        // *force = force_temp * sim_settings.force_multiplier;
        *force = force_temp;
    }

    fn populate_forces(&mut self, sim_settings: &SimSettings) {
        use rayon::prelude::*;

        let mut forces = vec![Vec2::new(0., 0.); self.forces.len()];
        // TODO: this is very wrong and reallocs each call, should just zero forces
        // forces.iter_mut().enumerate().for_each(|(i, force)| {

        forces.par_iter_mut().enumerate().for_each(|(i, force)| {
            self.compute_force(i, sim_settings, force);
        });
        self.forces = forces;
    }

    fn tick_with_substeps(&mut self, dt: f32, sim_settings: &SimSettings) {
        let dt = dt.min(0.1) / sim_settings.substep_n as f32;
        for _ in 0..sim_settings.substep_n {
            // self.spacial_partition.populate(&self.poses);

            // compute forces
            self.populate_forces(sim_settings);

            // do friction
            let friction = (0.5f32).powf(dt / sim_settings.friction_half_life);
            self.vels.iter_mut().for_each(|velocity| {
                *velocity *= friction;
            });

            // do kinematics
            for i in 0..self.poses.len() {
                // let position = unsafe { positions.get_unchecked_mut(i) };
                // let velocity = unsafe { velocities.get_unchecked_mut(i) };
                // let force = unsafe { forces.get_unchecked(i) };

                let position = &mut self.poses[i];
                let velocity = &mut self.vels[i];
                let force = self.forces[i];

                *velocity += force * dt;
                *position += *velocity * dt;
            }

            // do wall/screen wrapping
            for pos in self.poses.iter_mut() {
                if pos.x > 1.0 {
                    pos.x = 0.0;
                } else if pos.x < 0.0 {
                    pos.x = 1.0;
                }
                if pos.y > 1.0 {
                    pos.y = 0.0;
                } else if pos.y < 0.0 {
                    pos.y = 1.0;
                }
            }
            // TODO: why not this?
            // for pos in self.poses.iter_mut() {
            //     if pos.x > 1.0 {
            //         pos.x = pos.x.fract() - 1.0;
            //     } else if pos.x < 0.0 {
            //         pos.x = pos.x.fract() + 1.0;
            //     }
            //     if pos.y > 1.0 {
            //         pos.y = pos.y.fract() - 1.0;
            //     } else if pos.y < 0.0 {
            //         pos.y = pos.y.fract() + 1.0;
            //     }
            //     assert!((0.0..=1.0).contains(&pos.x));
            //     assert!((0.0..=1.0).contains(&pos.y));
            // }

            assert_eq!(sim_settings.particle_n, self.poses.len());
            assert_eq!(sim_settings.particle_n, self.vels.len());
            assert_eq!(sim_settings.particle_n, self.forces.len());
            assert_eq!(sim_settings.specie_n, self.attractions.len());
            assert_eq!(sim_settings.specie_n, self.attractions[0].len());
            // println!("{:?}", self);
        }
    }
}

fn get_attraction_force(distance: f32, attraction: f32) -> f32 {
    const BETA: f32 = 0.3;
    if distance < BETA {
        distance / BETA - 1.0
    } else {
        attraction * (1.0 - (2.0 * distance - 1.0 - BETA).abs() / (1.0 - BETA))
    }
}

struct App {
    view_settings: ViewSettings,
    sim_settings: SimSettings,
    // sim_data: SimData,
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
            // sim_data,
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

                // let particle_to_ui = |pos: Vec2| {
                //     // particles are in [0, 1] x [0, 1]
                //     // ui Pos2 is in whatever rect is
                //     rect.lerp_inside(pos)
                //     // (pos*scale).into()
                // };

                // self.sim_data.frame(dt, &self.sim_settings);

                // // draw sim_data
                // for i in 0..self.sim_settings.particle_n {
                //     ui.painter().circle_filled(
                //         particle_to_ui(self.sim_data.poses[i]),
                //         self.view_settings.particle_radius * scale,
                //         self.view_settings.specie_colors[self.sim_data.species[i] as usize],
                //     );
                // }

                self.view_settings.texture_size = 2 * scale as u32;
                self.sim_settings.dt = dt;

                // TODO: try to get the texture to be higher resolution. it's currently 1440x900, but my display can do 2560x1600
                // actually rendering it at 2x resolution look good
                // but the tips of triangles look wobbly at higher resolutions
                // TODO: try multisampling
                self.gfx_data
                    .render(&self.view_settings, &self.sim_settings);

                egui::widgets::Image::from_texture(egui::load::SizedTexture::new(
                    self.gfx_data.texture_id,
                    Vec2::new(10., 10.),
                ))
                .paint_at(ui, rect)
            });
    }
}
