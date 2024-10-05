// struct Particle {
//     pos: vec2<f32>,
//     vel: vec2<f32>,
// };

// struct SimParams {
//     deltaT: f32,
//     rule1Distance: f32,
//     rule2Distance: f32,
//     rule3Distance: f32,
//     rule1Scale: f32,
//     rule2Scale: f32,
//     rule3Scale: f32,
// };

// @group(0) @binding(0) var<uniform> params: SimParams;
// @group(0) @binding(1) var<storage, read> particlesSrc: array<Particle>;
// @group(0) @binding(2) var<storage, read_write> particlesDst: array<Particle>;

struct Params {
    specie_n: u32,
    particle_n: u32,
    local_radius: f32,
    local_radius2: f32,
    friction_half_life: f32,
    dt: f32,
    // force_multiplier: f32,
    particle_radius: f32,
    particle_radius2: f32,
    texture_size: u32,
    zoom_scale: f32,
    zoom_center_x: f32,
    zoom_center_y: f32,
    // _padding: f32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> pos_src: array<vec2<f32>>;
@group(0) @binding(2) var<storage, read> vel_src: array<vec2<f32>>;
@group(0) @binding(3) var<storage, read_write> pos_dst: array<vec2<f32>>;
@group(0) @binding(4) var<storage, read_write> vel_dst: array<vec2<f32>>;
@group(0) @binding(5) var<storage, read> species: array<u32>;
@group(0) @binding(6) var<storage, read> attractions: array<f32>;
@group(0) @binding(7) var<storage, read> specie_colors: array<vec4<f32>>;

// https://github.com/austinEng/Project6-Vulkan-Flocking/blob/master/data/shaders/computeparticles/particle.comp
@compute
@workgroup_size(64) // TODO: wtf should i do with this
fn main_cs(@builtin(global_invocation_id) global_invocation_id: vec3<u32>) {
    let index = global_invocation_id.x;
    if (index >= params.particle_n) {
        return;
    }

    let pos: vec2<f32> = pos_src[index];
    // let pos: vec2<f32> = vec2<f32>(0.0, 0.0);
    var force: vec2<f32> = vec2<f32>(0.0, 0.0);

    for (var neighbor_i: u32 = 0; neighbor_i < params.particle_n; neighbor_i++) {
        if (neighbor_i == index) {
            continue;
        }

        let neighbor_pos = pos_src[neighbor_i];
        var to_neighbor = neighbor_pos - pos;
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

        let distance2 = dot(to_neighbor, to_neighbor);
        if distance2 > params.local_radius2 {
            continue;
        }
        if distance2 == 0.0 {
            continue;
        }
        let distance = sqrt(distance2);
        force += (to_neighbor / distance)
            * get_attraction_force(
                distance / params.local_radius,
                attractions[species[index]*params.specie_n + species[neighbor_i]],
            );
    }
    // force = vec2(0.0, 0.0);
    // force = normalize(force) * clamp(length(force), 0.0, 10.0);
    force *= 32.0 / sqrt(f32(params.particle_n)); // TODO: this is kinda weird
    // force *= 1000.0 / f32(params.particle_n);

    var new_vel = vel_src[index] + force * params.dt;
    // new_vel = normalize(new_vel) * clamp(length(new_vel), 0.0, 10.0);

    let friction = pow(0.5, params.dt / params.friction_half_life);
    new_vel *= friction;

    var new_pos = pos + new_vel * params.dt;

    // wall wrapping
    // TODO: why snap it to the edges?
    if new_pos.x > 1.0 {
        new_pos.x = 0.0;
    } else if new_pos.x < 0.0 {
        new_pos.x = 1.0;
    }
    if new_pos.y > 1.0 {
        new_pos.y = 0.0;
    } else if new_pos.y < 0.0 {
        new_pos.y = 1.0;
    }

    pos_dst[index] = new_pos;
    vel_dst[index] = new_vel;
}

// TODO: this but without distance normalized by local_radius so i can do a convolution
fn get_attraction_force(distance: f32, attraction: f32) -> f32 {
    let BETA: f32 = 0.3;
    if (distance < BETA) {
        return distance / BETA - 1.0;
    } else {
        return attraction * (1.0 - abs(2.0 * distance - 1.0 - BETA) / (1.0 - BETA));
    }
}

struct VertexOutput {
    @builtin(position) weird_pos: vec4<f32>,
    @location(1) particle_pos: vec2<f32>,
    @location(2) particle_vel: vec2<f32>,
    @location(3) particle_species: u32,
}

@vertex
fn main_vs(
    @location(0) vertex_pos: vec2<f32>,
    @location(1) particle_pos: vec2<f32>,
    @location(2) particle_vel: vec2<f32>,
    @location(3) particle_species: u32,
) -> VertexOutput {
    // let angle = -atan2(particle_vel.x, particle_vel.y);
    // // let angle = 0.0;
    // let rotated_veretex = vec2<f32>(
    //     vertex_pos.x * cos(angle) - vertex_pos.y * sin(angle),
    //     vertex_pos.x * sin(angle) + vertex_pos.y * cos(angle)
    // );
    let scaled_particle_pos = particle_pos * 2.0 - vec2(1.0, 1.0);
    // let scaled_particle_pos = vec2(particle_pos.x * 2.0 - 1.0, 1.0 - particle_pos.x * 2.0);
    // TODO: zooming
    // return VertexOutput(
    //     vec4<f32>(rotated_veretex + scaled_particle_pos, 0.0, 1.0),
    //     particle_pos, particle_vel, particle_species);
    return VertexOutput(
        vec4<f32>(vertex_pos + scaled_particle_pos, 0.0, 1.0),
        particle_pos, particle_vel, particle_species);
}

@fragment
fn main_fs(in: VertexOutput) -> @location(0) vec4<f32> {
    // rp is the rasterization_point
    let rp_pos = in.weird_pos.xy; // each pixel is 1.0 units
    // let rp_depth = in.weird_pos.z;
    // let rp_perspective_divisor = in.weird_pos.w;

    var pixel_pos = rp_pos / f32(params.texture_size);
    pixel_pos = vec2(pixel_pos.x, 1.0 - pixel_pos.y);
    // let particle_pos = (in.particle_pos + vec2(1.0, 1.0)) / 2.0;
    let particle_pos = in.particle_pos;
    // let particle_pos = vec2(in.particle_pos.x, 1.0 - in.particle_pos.y);
    // let radius = 2.0 * params.particle_radius; // *2.0 because we're in normalized device coordinates
    let to_particle = particle_pos - pixel_pos;
    // return turbo(length(to_particle), 0.0, 1.0);
    // return vec4(abs(to_particle.x), abs(to_particle.y), 0.0, 1.0);
    // return turbo(length(pixel_pos), 0.0, 1.0);
    // return vec4(pixel_pos, 0.0, 1.0);
    // return turbo(length(particle_pos), 0.0, 1.0);
    // return vec4(particle_pos, 0.0, 1.0);
    // return turbo(particle_pos.x, 0.0, 1.0);
    // return turbo(particle_pos.y, 0.0, 1.0);
    // return turbo(pixel_pos.x, 0.0, 1.0);
    // return turbo(pixel_pos.y, 0.0, 1.0);
    // it seems like a y-down / y-up problem


    if dot(to_particle, to_particle) > params.particle_radius2 {
        discard;
    }
    let color = specie_colors[in.particle_species];
    return color;
}


fn turbo(value: f32, min: f32, max: f32) -> vec4<f32> {
    let kRedVec4: vec4<f32> = vec4(0.13572138, 4.61539260, -42.66032258, 132.13108234);
    let kGreenVec4: vec4<f32> = vec4(0.09140261, 2.19418839, 4.84296658, -14.18503333);
    let kBlueVec4: vec4<f32> = vec4(0.10667330, 12.64194608, -60.58204836, 110.36276771);
    let kRedVec2: vec2<f32> = vec2(-152.94239396, 59.28637943);
    let kGreenVec2: vec2<f32> = vec2(4.27729857, 2.82956604);
    let kBlueVec2: vec2<f32> = vec2(-89.90310912, 27.34824973);

    let x = saturate((value - min) / (max - min));
    if abs(x) < 0.51 && abs(x) > 0.49 {
        return vec4(1.0, 1.0, 1.0, 1.0);
    }
    let v4: vec4<f32> = vec4( 1.0, x, x * x, x * x * x);
    let v2: vec2<f32> = v4.zw * v4.z;
    return vec4(
        dot(v4, kRedVec4)   + dot(v2, kRedVec2),
        dot(v4, kGreenVec4) + dot(v2, kGreenVec2),
        dot(v4, kBlueVec4)  + dot(v2, kBlueVec2),
        1.0,
    );
}