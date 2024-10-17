particle life on the gpu implemented in rust with eframe and wgpu.
run with ```cargo run --release```

[very good video](https://www.youtube.com/watch?v=p4YirERTVF0)

particle life is like boids or a cellular automaton. it consists of a set of particles each with a position (which are in [0.0, 1.0]x[0.0, 1.0]), velocity, and species. each specie has an random attraction coefficient to each other specie. each simulation tick, each particle, for each other particle in a radius, gets a force applied determined by an activation function. the activation function is negative for small distances, proportional to the species' attraction coefficient for medium distances, and zero for distances greater than the local radius. then friction is applied. because the attraction coefficients aren't symmetric, the simulation doesn't conserve energy.

some parameters and their default values are particle_n = 5000, substep_n = 8, and local_radius = 0.1, which are important for performance (local radius isn't right now but may be in the future), and specie_n = 6, friction_half_life = 0.04, and attraction coefficients randomly in [-1.0, 1.0], which are non-performance-impacting aspects of the simulation. in the shader, i'm trying to do something with force scaling to make it stable across many particle counts.

the current algorithm is the naive O(particle_n**2), but each gpu thread(?) only does O(particle_n) work.

i want to try integration methods other than the euler method.
[verlet](https://en.wikipedia.org/wiki/Verlet_integration)
[leapfrog](https://en.wikipedia.org/wiki/Leapfrog_integration)

i may do a grid partitioning where you put particles into squares of diameter local_radius so for each particle, to calculate the force you only need to check the particles in the neighboring squares. however, this probably requires dynamic sized arrays which is annoying on the gpu, and the nature of particle life causes particles to clump together and not be evenly distributed such that not many particles would end up culled.

i want to try something like this [gpu boids](https://observablehq.com/@rreusser/gpgpu-boids) implementation that uses the [particle mesh method](https://en.wikipedia.org/wiki/Particle_mesh).
maybe also [Barnes-Hut Method](https://en.wikipedia.org/wiki/Barnes%E2%80%93Hut_simulation) or [Fast Multipole Method](https://en.wikipedia.org/wiki/Fast_multipole_method)

[wgpu/wgsl boids example](https://github.com/gfx-rs/wgpu/blob/trunk/examples/src/boids/mod.rs)

another particle sim:
[rendering circles video](https://www.youtube.com/watch?v=VEnglRKNHjU)
[rendering circles github](https://github.com/DeadlockCode/quarkstrom)