use graphics::math::{Vec2d, add, mul_scalar};
use piston_window::*; // Create a GUI program
use rand::prelude::*;
use std::alloc::{GlobalAlloc, System, Layout}; // Controls for memory allocation
use std::time::Instant;

#[global_allocator]
static ALLOCATOR: ReportingAllocator = ReportingAllocator;

// Provides a fairly accurate indication of time taken for dynamic memory allocation
struct ReportingAllocator; 

unsafe impl GlobalAlloc for ReportingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let start = Instant::now();
        
        // Defers the memory allocation to the default memory allocator;
        let ptr = System.alloc(layout); 

        let end = Instant::now();
        let time_taken = end - start;
        let bytes_requested = layout.size();

        eprintln!("{}\t{}", bytes_requested, time_taken.as_nanos());
        ptr              
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
    }
}

/// Contains the data that will be used through the lifetime of the program.
struct World {
    current_turn: u64,
    particles: Vec<Box<Particle>>,
    height: f64,
    width: f64,
    rng: ThreadRng,
}

// Defines the shape of and object in 2D space
struct Particle {
    height: f64,
    width: f64,
    position: Vec2d<f64>,
    velocity: Vec2d<f64>,
    acceleration: Vec2d<f64>,
    color: [f32; 4],
}

impl Particle {
    fn new(world: &World) -> Particle {
        let mut rng = thread_rng();

        // Starts at a random positin along the bottom of the window
        let x = rng.gen_range(0.0..=world.width);
        let y = world.height;
        let x_velocity = 0.0;
        let y_velocity = rng.gen_range(-2.0..0.0);
        let x_acceleration = 0.0;
        let y_acceleration = rng.gen_range(0.0..0.15);

        Particle {
            height: 4.0,
            width: 4.0,
            position: [x, y].into(),
            velocity: [x_velocity, y_velocity].into(),
            // Slows down the particle as it travels along the screen
            acceleration: [x_acceleration, y_acceleration].into(),
            color: [1.0, 1.0, 1.0, 0.99], // almost transparent white color
        }
    }

    fn update(&mut self) {
        self.velocity = add(self.velocity, self.acceleration);
        self.position = add(self.position, self.velocity);
        self.acceleration = mul_scalar(self.acceleration, 0.7);

        // Make the particcle more transparent over time
        self.color[3] *= 0.995;
    }
}

impl World {
    fn new(width: f64, height: f64) -> World {
        World {
            current_turn: 0,

            // Use Box instead of Particle in order to use extra more memory allocation
            particles: Vec::<Box<Particle>>::new(),
            height: height,
            width: width,
            rng: thread_rng(),
        }
    }

    fn add_shapes(&mut self, n: i32) {
        for _ in 0..n.abs() {

            // Create a particle as local variable in the Stack (memory)
            let particle = Particle::new(&self);

            // Move the particle to the heap and create a reference to it
            // in the Stack
            let boxed_particle = Box::new(particle);
            self.particles.push(boxed_particle);
        }
    }

    fn remove_shapes(&mut self, n: i32) {
        for _ in 0..n.abs() {
            let mut to_delete = None;

            // Split into its own variable to more easily fit on the page
            let particle_iter = self.particles
                .iter()
                .enumerate();

            // Remove the fist particle if it is invisible
            // otherwise remove the oldest
            for (i, particle) in particle_iter {
                if particle.color[3] < 0.02 {
                    to_delete = Some(i);
                }
                break;
            }

            if let Some(i) = to_delete {
                self.particles.remove(i);
            } else {
                self.particles.remove(0);
                
            };    
        }
    }

    fn update(&mut self) {
        // Generate a random number between -3 a 3 inclusive
        let n = self.rng.gen_range(-3..=3);

        if n > 0 {
            self.add_shapes(n);
        } else {
            self.remove_shapes(n);
        }

        self.particles.shrink_to_fit();
        for shape in &mut self.particles {
            shape.update();
        }

        self.current_turn += 1;
    }
}

/// Render particles along the screen using the Piston game engine.
fn main() {
    let (width, height) = (1280.0, 960.0);

    // This does not work on Arch Linux x64 running in VirtualBox
    // not even enabling 3D acceleration
    let mut window: PistonWindow = WindowSettings::new(
        "particles", [width, height]
    )
        .exit_on_esc(true)
        .build()
        .expect("Could not create a window.");

    let mut world = World::new(width, height);
    world.add_shapes(1000);

    while let Some(event) = window.next() {
        world.update();

        window.draw_2d(&event, |ctx, renderer, _device| {
            clear([0.15, 0.17, 0.17, 0.9], renderer);

            for s in &mut world.particles {
                let size = [s.position[0], s.position[1], s.width, s.height];
                rectangle(s.color, size, ctx.transform, renderer);
            }
        });
    }
    
}
