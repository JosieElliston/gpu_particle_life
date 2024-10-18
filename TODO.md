# TODO

- do side by side compare of different substep_n / dt / integration methods
- to fix feature = "accesskit" unused lints, try adding it
- README: remove default/approximate values for things (expect for the particle count)
- collect general TODOs here
- make references/resources {README section} / {file}
- do github about thing (!= README)

## view_settings

- view_settings.show_grid: bool in background (spacing is local_radius)
- reset colors button
- better default colors

## sim_data: changing particle/sim data

- randomize positions
- init positions uniformly / spiral / other things
- randomize vels
- randomize species
- equalize specie counts
- change specie counts

## mouse_settings

- add particle
- add many particles (per frame)
- mouse click to add 1 particle
- mouse hold to add many particles
- new cpu_struct: mouse_settings
- param particles_per_second
- param hold_time_until_continuous_adding (theres a better word for continuous_adding) (also better word that add, maybe insert/___)
- position perturbations
    - None,
    - Uniform(radius: f32),
    - Gaussian/Normal(sd: f32),
    - not enum because want radius and sd to persists
    - rename sd
    - default radius = local_radius / 2.0
    - default sd = local_radius / 4.0
- pdf for which specie to add
    - idea 0: 6 buttons at top to do pure that specie
    - idea 1: 6 sliders
        - button the the right of each to do pure that specie
    - idea 2: slider with 6 sections + idea 0
        - so 5 handles
            - or maybe 6 handles with the leftmost or rightmost one being static but you can double click them to do pure that specie
                - add docs on hover for this and weird things like it (add to readme that docs on hover are a thing)
        - handles are either
            - handle_width = 0
                - bad because you can't grab the handle if >=2 are stacked/coincident
            - circle things that don't take up width in the slider and overlap the colors
                - bad because coincident but worse
            - grey/back rectangles that take up width in the slider and don't overlap the colors
                - bad because it kinda throws off how sampling is choosing the color of a random point in the slider
            - handles on top of slider
                - they try follow {the division between species} / {where they should be}
                but don't overlap
                and maybe don't go over the left / right edges
                - shape
                    - circles
                        - touching at 1 point might look weird
                    - rects
                        - touching together with no space might look weird
    - idea 3: 6 drag_values horizontally
        - bar chart like idea 2 for rendering
        - double click (or something) the drag_values to do pure that specie

- del particle?
    - specific particle
    - particle nearest to mouse
    - particles within radius of mouse

## selections

- select group of particles
    - idea 1: all in radius of mouse
    - idea 2: smart select creature

- rotate/steer creature (vels and poses) (keybinds for steering)
- center camera on them
- camera lock on creature
    - with rotation
    - without rotation

## keybinds

- movement
    - wasd panning
    - qe rotate if (no selection) {everything} else {selection}
    - +- zoom?
    - ? reset zoom
    - ? pause sim (time_scale: Option\<f32>)
    - ? select largest/next creature
    - ? center on selection
    - ?+scroll change radius (different keys for different radiuses)
        - only for ones where it matters where your mouse is
        - ie for adding particles or sections but not local_radius or particle_radius

## refactoring

- verlet integration: 3 pos buffers and 0 vel buffers
- rename local_radius
- rename frame_parity to sim_step_n/count
- rename substep_n to steps_per_frame/ticks_per_frame
- rename \*_n to \*_count?
    - bad because specie_n refers to the number of different species, specie_count refers to how many particles are of that specie
- should have better names for {specie_n = the number of different species} and {specie_count(s) = how many particles are of a/each specie}
- move cpu_structs out of main.rs
- use tick and step consistently
    - i use step i think
    - but that's maybe more ambiguous than tick

## particle_n invariance

- change particle_radius setting to particle_radius_mul so (in gpu params) particle_radius = particle_radius_mul * particle_n.sqrt()
- make small particle radiuses draw correctly
    - antialiasing?
    - blur something?
    - probably in [particle sim framework](https://github.com/DeadlockCode/quarkstrom)

## extensions

- why is 2 trailing spaces ok?
    - MD009/no-trailing-spaces: Trailing spaces [Expected: 0 or 2; Actual: 3] markdownlintMD009
- category:formatters plaintext
- typing a character then tab autocompletes things, can cycle with more pressing tab
- markdown: in a list, make a new entry (instead of enter then space then dash)
