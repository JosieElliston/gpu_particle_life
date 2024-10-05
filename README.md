particle life implemented on the gpu in rust.
run with ```cargo run --release```

[very good video](https://www.youtube.com/watch?v=p4YirERTVF0)

particle life is like boids or a cellular automaton. it consists of a set of particles each with a position, velocity, and species. each specie has an attraction coefficient to each other specie. each simulation tick, each particle, for each other particle in a radius, gets a force applied determined by an activation function. the activation function is negative for small distances, proportional to the species' attraction coefficient for medium distances, and zero for distances greater than the local radius. then friction is applied. because the attraction coefficients aren't symmetric, the simulation doesn't conserve energy.

some parameters and their approximate values are particle count ~ 5000, simulation substep count ~ 8, and local radius ~ 0.1, which are important for performance (local radius isn't right now but may be in the future), and species count ~ 6 and friction half life ~ 0.04, which are non-performance-impacting aspects of the simulation. in the shader, i'm trying to do something with force scaling to make it stable across many particle counts. position is in [0.0, 1.0], velocity is approximately in [-1.0, ~1.0], and species is approximately in {0, 1, ..., ~6}.

the current algorithm is the naive O(particle_n**2).

i want to try integration methods other than the euler method.

i may do a grid partitioning where you put particles into squares of diameter local_radius so for each particle, to calculate the force you only need to check the particles in the neighboring squares. however, this probably requires dynamic sized arrays which is annoying on the gpu, and the nature of particle life causes particles to clump together and not be evenly distributed such that not many particles would end up culled.

i want to try something like this [gpu boids](https://observablehq.com/@rreusser/gpgpu-boids) implementation that uses the [particle mesh method](https://en.wikipedia.org/wiki/Particle_mesh).
