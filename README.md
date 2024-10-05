particle life implemented on the gpu in rust.
run with ```cargo run --release```

[video](https://www.youtube.com/watch?v=p4YirERTVF0)

particle life is like a cellular automaton.
it consists of a set of particles each with a position, velocity, and species.
each specie has an attraction coefficient to each other specie.
each simulation tick, each particle, for each other particle in a radius, gets a force applied determined by the activation function.
the activation function is negative for small distances, proportional to the species' attraction coefficient for medium distances, and zero for large distances.
then friction is applied.

