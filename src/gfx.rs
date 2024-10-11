use std::sync::Arc;

use eframe::wgpu::{self, util::DeviceExt};

use crate::{SimData, SimSettings, ViewSettings};

const PARTICLES_PER_GROUP: usize = 64;

pub(crate) struct GfxData {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    renderer: Arc<eframe::egui::mutex::RwLock<eframe::egui_wgpu::Renderer>>,
    texture: wgpu::Texture,
    pub(crate) texture_id: eframe::egui::TextureId,
    shader_params_buffer: wgpu::Buffer,
    pos_buffer0: wgpu::Buffer,
    vel_buffer0: wgpu::Buffer,
    pos_buffer1: wgpu::Buffer,
    vel_buffer1: wgpu::Buffer,
    specie_buffer: wgpu::Buffer,
    attraction_buffer: wgpu::Buffer,
    specie_color_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    compute_bind_groups: [wgpu::BindGroup; 2],
    compute_pipeline: wgpu::ComputePipeline,
    render_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    swap_parity: bool,
}
impl GfxData {
    pub(crate) fn new(
        cc: &eframe::CreationContext<'_>,
        view_settings: &ViewSettings,
        sim_settings: &SimSettings,
        sim_data: &SimData,
    ) -> Self {
        let render_state = cc.wgpu_render_state.as_ref().unwrap();
        let device = render_state.device.clone();
        let queue = render_state.queue.clone();
        let renderer = render_state.renderer.clone();

        // make buffers
        let shader_params_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("shader_params_buffer"),
            contents: bytemuck::bytes_of(&ShaderParams::new(view_settings, sim_settings)),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let pos_buffer0 = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("pos_buffer0"),
            contents: bytemuck::cast_slice(&sim_data.poses),
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                // | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
        });
        let vel_buffer0 = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vel_buffer0"),
            contents: bytemuck::cast_slice(&sim_data.vels),
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                // | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
        });
        let pos_buffer1 = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("pos_buffer1"),
            contents: bytemuck::cast_slice(&sim_data.poses),
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                // | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
        });
        let vel_buffer1 = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vel_buffer1"),
            contents: bytemuck::cast_slice(&sim_data.vels),
            usage: wgpu::BufferUsages::VERTEX
                | wgpu::BufferUsages::STORAGE
                // | wgpu::BufferUsages::COPY_SRC
                | wgpu::BufferUsages::COPY_DST,
        });
        let specie_buffer: wgpu::Buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("specie_buffer"),
                contents: bytemuck::cast_slice(&sim_data.species),
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
            });
        let attraction_buffer: wgpu::Buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("attraction_buffer"),
                contents: bytemuck::cast_slice(
                    &sim_settings
                        .attractions
                        .clone()
                        .into_iter()
                        .flatten()
                        .collect::<Vec<f32>>(),
                ),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
        let specie_color_buffer: wgpu::Buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("specie_color_buffer"),
                contents: bytemuck::cast_slice(&view_settings.specie_colors),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex_buffer"),
            contents: bytemuck::bytes_of(&get_triangle(view_settings.particle_radius)),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        // TODO: make compute_bind_group_layout after the buffers so i can use stuff like specie_buffer.size();

        // create compute bind layout group and compute pipeline layout and compute pipeline
        let shader_module = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let compute_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("compute_bind_group_layout"),
                entries: &[
                    // shader_params_buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                (size_of::<ShaderParams>()) as _,
                            ),
                        },
                        count: None,
                    },
                    // pos_buffer0
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                // (sim_settings.particle_n * size_of::<eframe::egui::Vec2>()) as _,
                                size_of_val(sim_data.poses.as_slice()) as _,
                            ),
                        },
                        count: None,
                    },
                    // vel_buffer0
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                // (sim_settings.particle_n * size_of::<eframe::egui::Vec2>()) as _,
                                size_of_val(sim_data.vels.as_slice()) as _,
                            ),
                        },
                        count: None,
                    },
                    // pos_buffer1
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                // (sim_settings.particle_n * size_of::<eframe::egui::Vec2>()) as _,
                                size_of_val(sim_data.poses.as_slice()) as _,
                            ),
                        },
                        count: None,
                    },
                    // vel_buffer1
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: false },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                // (sim_settings.particle_n * size_of::<eframe::egui::Vec2>()) as _,
                                size_of_val(sim_data.vels.as_slice()) as _,
                            ),
                        },
                        count: None,
                    },
                    // specie_buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                // (sim_settings.particle_n * size_of::<u8>()) as _,
                                size_of_val(sim_data.species.as_slice()) as _, // TODO: do this for all of them
                            ),
                        },
                        count: None,
                    },
                    // attraction_buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::COMPUTE,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                (sim_settings.specie_n * sim_settings.specie_n * size_of::<f32>())
                                    as _,
                            ),
                        },
                        count: None,
                    },
                ],
            });
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("compute_pipeline_layout"),
                bind_group_layouts: &[&compute_bind_group_layout],
                push_constant_ranges: &[],
            });
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("compute_pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &shader_module,
            entry_point: "main_cs",
            compilation_options: Default::default(),
        });

        // create two bind groups, one for each buffer as the src
        // where the alternate buffer is used as the dst
        let compute_bind_groups: [wgpu::BindGroup; 2] = (0..2)
            .map(|i| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some(&format!("compute_bind_group {i}")),
                    layout: &compute_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: shader_params_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: [&pos_buffer0, &pos_buffer1][i].as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: [&vel_buffer0, &vel_buffer1][i].as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: [&pos_buffer0, &pos_buffer1][(i + 1) % 2].as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: [&vel_buffer0, &vel_buffer1][(i + 1) % 2].as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: specie_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 6,
                            resource: attraction_buffer.as_entire_binding(),
                        },
                    ],
                })
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();

        // render stuff
        // let texture = create_texture(
        //     &device,
        //     wgpu::Extent3d {
        //         width: screen_size,
        //         height: screen_size,
        //         depth_or_array_layers: 1,
        //     },
        // );
        // let screen_size = cc.egui_ctx.screen_rect().size().min_elem();
        let texture = create_texture(
            &device,
            wgpu::Extent3d {
                width: view_settings.texture_size,
                height: view_settings.texture_size,
                depth_or_array_layers: 1,
            },
        );
        let texture_id = renderer.write().register_native_texture(
            &device,
            &texture.create_view(&wgpu::TextureViewDescriptor::default()),
            wgpu::FilterMode::Nearest,
        );

        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("render_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                (size_of::<ShaderParams>()) as _,
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 7,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(size_of_val(
                                view_settings.specie_colors.as_slice(),
                            )
                                as _),
                        },
                        count: None,
                    },
                ],
            });
        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("render_bind_group"),
            layout: &render_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: shader_params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: specie_color_buffer.as_entire_binding(),
                },
            ],
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render_pipeline_layout"),
                bind_group_layouts: &[&render_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "main_vs",
                compilation_options: Default::default(),
                buffers: &[
                    // @location(0) vertex_pos: vec2<f32>,
                    wgpu::VertexBufferLayout {
                        array_stride: 4 * 2,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                    },
                    // @location(1) particle_pos: vec2<f32>,
                    wgpu::VertexBufferLayout {
                        array_stride: 4 * 2,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![1 => Float32x2],
                    },
                    // @location(2) particle_vel: vec2<f32>,
                    wgpu::VertexBufferLayout {
                        array_stride: 4 * 2,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![2 => Float32x2],
                    },
                    // @location(3) particle_species: u32,
                    wgpu::VertexBufferLayout {
                        array_stride: 4,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![3 => Uint32],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "main_fs",
                compilation_options: Default::default(),
                // targets: &[Some(config.view_formats[0].into())],
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture.format(),
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::all(),
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self {
            device,
            queue,
            renderer,
            texture,
            texture_id,
            shader_params_buffer,
            pos_buffer0,
            vel_buffer0,
            pos_buffer1,
            vel_buffer1,
            specie_buffer,
            attraction_buffer,
            specie_color_buffer,
            compute_bind_groups,
            compute_pipeline,
            vertex_buffer,
            render_bind_group,
            render_pipeline,
            swap_parity: false,
        }
    }

    pub(crate) fn render(&mut self, view_settings: &ViewSettings, sim_settings: &SimSettings) {
        let mut command_encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("command_encoder"),
                });

        self.queue.write_buffer(
            &self.shader_params_buffer,
            0,
            bytemuck::bytes_of(&ShaderParams::new(view_settings, sim_settings)),
        );

        // compute pass
        command_encoder.push_debug_group("compute_pass");
        {
            self.queue.write_buffer(
                &self.attraction_buffer,
                0,
                bytemuck::cast_slice(
                    &sim_settings
                        .attractions
                        .clone()
                        .into_iter()
                        .flatten()
                        .collect::<Vec<f32>>(),
                ),
            );
            for _ in 0..sim_settings.substep_n {
                let mut compute_pass =
                    command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                        label: Some("compute_pass"),
                        timestamp_writes: None,
                    });
                compute_pass.set_pipeline(&self.compute_pipeline);
                compute_pass.set_bind_group(
                    0,
                    &self.compute_bind_groups[self.swap_parity as usize],
                    &[],
                );
                let work_group_count =
                    ((sim_settings.particle_n as f32) / (PARTICLES_PER_GROUP as f32)).ceil() as u32;
                compute_pass.dispatch_workgroups(work_group_count, 1, 1);
                self.swap_parity = !self.swap_parity;
            }
        }
        command_encoder.pop_debug_group();

        // let cpu_readable_buffer;
        // {
        //     cpu_readable_buffer = Arc::new(std::sync::Mutex::new(
        //         self.device
        //             .create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //                 label: None,
        //                 contents: bytemuck::cast_slice(
        //                     &(0..sim_settings.particle_n)
        //                         .map(|_| eframe::egui::Vec2::new(0.0, 0.0))
        //                         .collect::<Vec<_>>(),
        //                 ),
        //                 usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        //             }),
        //     ));
        //     command_encoder.copy_buffer_to_buffer(
        //         &self.pos_buffer0,
        //         0,
        //         &cpu_readable_buffer.lock().unwrap(),
        //         0,
        //         8 * sim_settings.particle_n as u64,
        //     );
        //     cpu_readable_buffer
        //         .clone()
        //         .lock()
        //         .unwrap()
        //         .slice(..)
        //         .map_async(wgpu::MapMode::Read, move |a| {
        //             dbg!(a);
        //             dbg!(cpu_readable_buffer.clone());
        //         });
        //     // let slice = cpu_readable_buffer.slice(..).get_mapped_range();
        //     // let cpu_poses = bytemuck::cast_slice::<u8, eframe::egui::Vec2>(&slice);
        //     // dbg!(cpu_poses);
        //     // queue.submit(..), instance.poll_all(..), or device.poll(..)
        // }

        // render pass
        command_encoder.push_debug_group("render_pass");
        {
            let new_size = wgpu::Extent3d {
                width: view_settings.texture_size,
                height: view_settings.texture_size,
                depth_or_array_layers: 1,
            };
            // dbg!(self.texture.size());
            if self.texture.size() != new_size {
                println!("self.texture.size() != new_size");
                self.texture = create_texture(&self.device, new_size);
                self.renderer.write().update_egui_texture_from_wgpu_texture(
                    &self.device,
                    &self
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                    eframe::wgpu::FilterMode::Nearest,
                    self.texture_id,
                );
            }
            self.queue.write_buffer(
                &self.vertex_buffer,
                0,
                bytemuck::bytes_of(&get_triangle(view_settings.particle_radius)),
            );
            self.queue.write_buffer(
                &self.specie_color_buffer,
                0,
                bytemuck::cast_slice(&view_settings.specie_colors),
            );

            let texture_view = self
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            // let color_attachments = [Some(wgpu::RenderPassColorAttachment {
            //     view: &texture_view,
            //     resolve_target: None,
            //     ops: wgpu::Operations {
            //         // Not clearing here in order to test wgpu's zero texture initialization on a surface texture.
            //         // Users should avoid loading uninitialized memory since this can cause additional overhead.
            //         load: wgpu::LoadOp::Load,
            //         store: wgpu::StoreOp::Store,
            //     },
            // })];
            let color_attachments = [Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                resolve_target: None,
                ops: wgpu::Operations::default(),
            })];
            let render_pass_descriptor = wgpu::RenderPassDescriptor {
                label: Some("render_pass_descriptor"),
                color_attachments: &color_attachments,
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            };
            let mut render_pass = command_encoder.begin_render_pass(&render_pass_descriptor);
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.render_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(
                1,
                [&self.pos_buffer0, &self.pos_buffer1][self.swap_parity as usize].slice(..),
            );
            render_pass.set_vertex_buffer(
                2,
                [&self.vel_buffer0, &self.vel_buffer1][self.swap_parity as usize].slice(..),
            );
            render_pass.set_vertex_buffer(3, self.specie_buffer.slice(..));
            render_pass.draw(0..3, 0..sim_settings.particle_n as _);
        }
        command_encoder.pop_debug_group();

        self.queue.submit([command_encoder.finish()]);
        // dbg!(cpu_readable_buffer);
    }
}

#[derive(Clone, Copy, bytemuck::NoUninit)]
#[repr(C)]
struct ShaderParams {
    specie_n: u32,
    particle_n: u32,
    local_radius: f32,
    local_radius2: f32,
    friction: f32,
    dt: f32,
    force_multiplier: f32,
    particle_radius: f32,
    particle_radius2: f32,
    texture_size: u32,
    zoom_scale: f32,
    zoom_center_x: f32,
    zoom_center_y: f32,
}
impl ShaderParams {
    fn new(view_settings: &ViewSettings, sim_settings: &SimSettings) -> Self {
        let dt = sim_settings.dt / sim_settings.substep_n as f32;
        Self {
            specie_n: sim_settings.specie_n as _,
            particle_n: sim_settings.particle_n as _,
            local_radius: sim_settings.local_radius,
            local_radius2: sim_settings.local_radius * sim_settings.local_radius,
            friction: 0.5_f32.powf(dt / sim_settings.friction_half_life),
            dt,
            force_multiplier: 32.0 / (sim_settings.particle_n as f32).sqrt(), // is 1.0 for particle_n = 1024
            particle_radius: view_settings.particle_radius,
            particle_radius2: view_settings.particle_radius * view_settings.particle_radius,
            texture_size: view_settings.texture_size,
            zoom_scale: view_settings.zoom_scale,
            zoom_center_x: view_settings.zoom_center.y,
            zoom_center_y: view_settings.zoom_center.y,
        }
    }
}

fn create_texture(device: &wgpu::Device, size: wgpu::Extent3d) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("texture"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING
            | wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[eframe::wgpu::TextureFormat::Rgba8UnormSrgb],
    })
}

fn get_triangle(radius: f32) -> [f32; 6] {
    // returns a small triangle that contains the circle centered at the origin with the given radius
    // desmos.com/calculator/ksbrvgwqfp
    // TODO: why do i need to scale it up?
    // [
    //     -3.0 * radius,
    //     -radius,
    //     radius,
    //     -radius,
    //     radius,
    //     3.0 * radius,
    // ]
    [
        -6.0 * radius,
        -2.0 * radius,
        2.0 * radius,
        -2.0 * radius,
        2.0 * radius,
        6.0 * radius,
    ]
}
