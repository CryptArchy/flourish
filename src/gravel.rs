use bytemuck::{Pod, Zeroable};
use winit::dpi::PhysicalSize;

const BOULDER_COUNT: usize = 18;
const LARGE_COUNT: usize = 36;
const MEDIUM_COUNT: usize = 210;
const SMALL_COUNT: usize = 1_392;
const MAX_ROCKS: usize = BOULDER_COUNT + LARGE_COUNT + MEDIUM_COUNT + SMALL_COUNT;
const PILE_BINS: usize = 320;
const PILE_BINS_F32: f32 = 320.0;
const PILE_LAST_F32: f32 = 319.0;
// Once a z-plane fills the viewport, keep later stones grazing the top edge
// instead of allowing the heightfield to push them entirely off-canvas.
const PILE_CEILING: f32 = 0.035;
const ROCK_VERTICES: u32 = 27;
const SPAWN_RATE: f32 = 86.0;
const HOLD_GRAVITY: f32 = 1.08;
const RELEASE_GRAVITY: f32 = 2.35;
// A stone may never occupy more than this fraction of the width. Radii are
// derived by dividing by the aspect ratio, so a portrait or rotated display
// would otherwise produce stones wider than the screen and a spawn span that
// runs backwards.
const MAX_HORIZONTAL_RADIUS: f32 = 0.45;

const GRAVEL_PALETTE: [[f32; 4]; 6] = [
    [0.48, 0.41, 0.31, 1.0],
    [0.58, 0.48, 0.36, 1.0],
    [0.34, 0.33, 0.30, 1.0],
    [0.68, 0.59, 0.45, 1.0],
    [0.24, 0.24, 0.22, 1.0],
    [0.55, 0.35, 0.20, 1.0],
];

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum GravelLayer {
    Boulder,
    Large,
    Medium,
    Small,
}

impl GravelLayer {
    const fn radius_range(self) -> (f32, f32) {
        match self {
            Self::Boulder => (0.120, 0.220),
            Self::Large => (0.055, 0.095),
            Self::Medium => (0.028, 0.052),
            Self::Small => (0.015, 0.028),
        }
    }

    const fn spawn_rate(self) -> f32 {
        match self {
            Self::Boulder => 12.0,
            Self::Large => 28.0,
            Self::Medium => 105.0,
            Self::Small => 225.0,
        }
    }

    const fn pause_after(self) -> f32 {
        match self {
            Self::Boulder => 0.65,
            Self::Large | Self::Small => 0.0,
            Self::Medium => 0.30,
        }
    }

    const fn gravity_scale(self) -> f32 {
        match self {
            Self::Boulder => 0.72,
            Self::Large => 0.86,
            Self::Medium => 1.0,
            Self::Small => 1.12,
        }
    }

    const fn color_scale(self) -> f32 {
        match self {
            Self::Boulder => 0.62,
            Self::Large => 0.78,
            Self::Medium => 0.91,
            Self::Small => 1.0,
        }
    }

    const fn pile_index(self) -> usize {
        match self {
            Self::Boulder => 0,
            Self::Large | Self::Medium => 1,
            Self::Small => 2,
        }
    }
}

const fn layer_for_index(index: usize) -> Option<GravelLayer> {
    if index < BOULDER_COUNT {
        Some(GravelLayer::Boulder)
    } else if index < BOULDER_COUNT + LARGE_COUNT {
        Some(GravelLayer::Large)
    } else if index < BOULDER_COUNT + LARGE_COUNT + MEDIUM_COUNT {
        Some(GravelLayer::Medium)
    } else if index < BOULDER_COUNT + LARGE_COUNT + MEDIUM_COUNT + SMALL_COUNT {
        Some(GravelLayer::Small)
    } else {
        None
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GravelInstance {
    center: [f32; 2],
    size: [f32; 2],
    color: [f32; 4],
    rotation: f32,
    seed: f32,
    padding: [f32; 2],
}

#[derive(Clone, Copy, Debug)]
struct Rock {
    center: [f32; 2],
    size: [f32; 2],
    color: [f32; 4],
    velocity: f32,
    rotation: f32,
    angular_velocity: f32,
    seed: f32,
    settled: bool,
    layer: GravelLayer,
}

impl Rock {
    fn instance(self) -> GravelInstance {
        GravelInstance {
            center: self.center,
            size: self.size,
            color: self.color,
            rotation: self.rotation,
            seed: self.seed,
            padding: [0.0; 2],
        }
    }
}

struct GravelSimulation {
    rocks: Vec<Rock>,
    heightfields: [Vec<f32>; 3],
    instances: Vec<GravelInstance>,
    random_state: u64,
    spawn_accumulator: f32,
    last_time: f32,
    aspect: f32,
    released: bool,
}

/// Mixes a 32-bit seed into a non-zero 64-bit xorshift state.
///
/// The state must never be zero, or the generator latches at zero forever.
const fn seed_state(seed: u32) -> u64 {
    let spread = (seed as u64) << 32 | (seed as u64 ^ 0xa5a5_a5a5);
    spread ^ 0x5eed_fade_cafe_beef
}

impl GravelSimulation {
    fn new(aspect: f32, seed: u32) -> Self {
        Self {
            rocks: Vec::with_capacity(MAX_ROCKS),
            heightfields: std::array::from_fn(|_| vec![1.0; PILE_BINS]),
            instances: Vec::with_capacity(MAX_ROCKS),
            random_state: seed_state(seed),
            spawn_accumulator: 1.0,
            last_time: 0.0,
            aspect,
            released: false,
        }
    }

    fn reset(&mut self, aspect: f32, seed: u32) {
        self.rocks.clear();
        for heightfield in &mut self.heightfields {
            heightfield.fill(1.0);
        }
        self.instances.clear();
        self.random_state = seed_state(seed);
        self.spawn_accumulator = 1.0;
        self.last_time = 0.0;
        self.aspect = aspect;
        self.released = false;
    }

    /// Retargets a running simulation at a new aspect ratio without discarding
    /// the pile the audience is already looking at.
    fn retarget(&mut self, aspect: f32) {
        self.aspect = aspect;
    }

    fn update(&mut self, time: f32, exit_progress: f32) -> &[GravelInstance] {
        let delta_time = (time - self.last_time).clamp(0.0, 0.05);
        self.last_time = time;

        if !self.released && exit_progress > 0.0 {
            self.released = true;
            for rock in &mut self.rocks {
                rock.settled = false;
                rock.velocity = rock.velocity.max(0.05 + rock.seed.fract() * 0.12);
                rock.angular_velocity = (rock.seed * 7.3).sin() * 1.8;
            }
        }

        if !self.released {
            let next_layer = layer_for_index(self.rocks.len());
            self.spawn_accumulator +=
                delta_time * next_layer.map_or(SPAWN_RATE, GravelLayer::spawn_rate);
            while self.spawn_accumulator >= 1.0 {
                let Some(layer) = layer_for_index(self.rocks.len()) else {
                    break;
                };
                self.spawn_rock(layer);
                self.spawn_accumulator -= 1.0;
                if layer_for_index(self.rocks.len()) != Some(layer) {
                    let next_rate = layer_for_index(self.rocks.len())
                        .map_or(SPAWN_RATE, GravelLayer::spawn_rate);
                    self.spawn_accumulator = -layer.pause_after() * next_rate;
                    break;
                }
            }
        }

        let gravity = if self.released {
            RELEASE_GRAVITY
        } else {
            HOLD_GRAVITY
        };
        let mut newly_settled = Vec::new();
        for (index, rock) in self.rocks.iter_mut().enumerate() {
            if rock.settled {
                continue;
            }

            rock.velocity += gravity * rock.layer.gravity_scale() * delta_time;
            rock.center[1] += rock.velocity * delta_time;
            rock.rotation += rock.angular_velocity * delta_time;

            if !self.released {
                let pile_height =
                    collision_height(&self.heightfields[rock.layer.pile_index()], *rock);
                if rock.center[1] + rock.size[1] >= pile_height {
                    rock.center[1] = pile_height - rock.size[1];
                    rock.velocity = 0.0;
                    rock.angular_velocity = 0.0;
                    rock.settled = true;
                    newly_settled.push(index);
                }
            }
        }
        for index in newly_settled {
            let rock = self.rocks[index];
            add_to_heightfield(&mut self.heightfields[rock.layer.pile_index()], rock);
        }

        self.instances.clear();
        self.instances.extend(
            self.rocks
                .iter()
                .filter(|rock| rock.center[1] - rock.size[1] < 1.12)
                .copied()
                .map(Rock::instance),
        );
        &self.instances
    }

    fn spawn_rock(&mut self, layer: GravelLayer) {
        let size_bias = self.next_f32();
        let (minimum_radius, maximum_radius) = layer.radius_range();
        let radius_y = minimum_radius + size_bias * size_bias * (maximum_radius - minimum_radius);
        let shape_aspect = 0.74 + self.next_f32() * 0.62;
        let radius_x = (radius_y * shape_aspect / self.aspect.max(0.1)).min(MAX_HORIZONTAL_RADIUS);
        // Clamped above, so this span is always non-negative and the stone
        // always spawns fully within the viewport.
        let x = radius_x + self.next_f32() * (1.0 - radius_x * 2.0);
        let color_index =
            usize::try_from(self.next_u32()).expect("u32 fits in usize") % GRAVEL_PALETTE.len();
        let rotation = self.next_f32() * std::f32::consts::TAU;
        let seed = self.next_f32() * 8192.0;
        let start_y = -radius_y - self.next_f32() * 0.22;
        let velocity = 0.050 + self.next_f32() * 0.14;
        let angular_velocity = (self.next_f32() - 0.5) * 2.8;
        let mut color = GRAVEL_PALETTE[color_index];
        let color_scale = layer.color_scale();
        for channel in &mut color[0..3] {
            *channel *= color_scale;
        }
        self.rocks.push(Rock {
            center: [x, start_y],
            size: [radius_x, radius_y],
            color,
            velocity,
            rotation,
            angular_velocity,
            seed,
            settled: false,
            layer,
        });
    }

    fn next_u32(&mut self) -> u32 {
        self.random_state ^= self.random_state << 13;
        self.random_state ^= self.random_state >> 7;
        self.random_state ^= self.random_state << 17;
        u32::try_from((self.random_state >> 16) & u64::from(u32::MAX))
            .expect("masked random state fits in u32")
    }

    #[allow(clippy::cast_precision_loss)]
    fn next_f32(&mut self) -> f32 {
        let mantissa = self.next_u32() >> 8;
        mantissa as f32 / 16_777_216.0
    }
}

pub struct GravelEffect {
    simulation: GravelSimulation,
    instance_buffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    instance_count: u32,
}

impl GravelEffect {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        size: PhysicalSize<u32>,
        seed: u32,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Gravel fall shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/gravel.wgsl").into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Gravel fall pipeline layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Gravel fall pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Some(instance_layout())],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let buffer_size = u64::try_from(MAX_ROCKS * std::mem::size_of::<GravelInstance>())
            .expect("gravel instance buffer size fits in u64");
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Gravel fall instances"),
            size: buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            simulation: GravelSimulation::new(screen_aspect(size), seed),
            instance_buffer,
            pipeline,
            instance_count: 0,
        }
    }

    pub fn reset(&mut self, size: PhysicalSize<u32>, seed: u32) {
        self.simulation.reset(screen_aspect(size), seed);
        self.instance_count = 0;
    }

    /// Adopts a new aspect ratio for stones spawned from here on.
    ///
    /// Deliberately preserves the existing pile: a resize mid-flourish is
    /// usually a one-off display change, and restarting the simulation would
    /// make the pile vanish and rebuild in front of the audience.
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.simulation.retarget(screen_aspect(size));
    }

    pub fn prepare(&mut self, queue: &wgpu::Queue, time: f32, exit_progress: f32) {
        let instances = self.simulation.update(time, exit_progress);
        self.instance_count = u32::try_from(instances.len()).expect("rock count fits in u32");
        if !instances.is_empty() {
            queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(instances));
        }
    }

    pub fn render<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.instance_buffer.slice(..));
        pass.draw(0..ROCK_VERTICES, 0..self.instance_count);
    }
}

fn instance_layout() -> wgpu::VertexBufferLayout<'static> {
    const ATTRIBUTES: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
        2 => Float32x4,
        3 => Float32,
        4 => Float32
    ];
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<GravelInstance>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &ATTRIBUTES,
    }
}

fn covered_bins(rock: Rock) -> std::ops::RangeInclusive<usize> {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let start = ((rock.center[0] - rock.size[0]).clamp(0.0, 1.0) * PILE_LAST_F32).floor() as usize;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let end = ((rock.center[0] + rock.size[0]).clamp(0.0, 1.0) * PILE_LAST_F32).ceil() as usize;
    start..=end
}

#[allow(clippy::cast_precision_loss)]
fn collision_height(heightfield: &[f32], rock: Rock) -> f32 {
    let (total, count) = covered_bins(rock)
        .map(|index| heightfield[index])
        .fold((0.0, 0_usize), |(total, count), height| {
            (total + height, count + 1)
        });
    total / count as f32
}

#[allow(clippy::cast_precision_loss)]
fn add_to_heightfield(heightfield: &mut [f32], rock: Rock) {
    for index in covered_bins(rock) {
        let bin_x = (index as f32 + 0.5) / PILE_BINS_F32;
        let horizontal = ((bin_x - rock.center[0]) / rock.size[0]).clamp(-1.0, 1.0);
        let vertical_extent = rock.size[1] * (1.0 - horizontal * horizontal).sqrt();
        heightfield[index] = heightfield[index]
            .min(rock.center[1] - vertical_extent)
            .max(PILE_CEILING);
    }
}

#[allow(clippy::cast_precision_loss)]
fn screen_aspect(size: PhysicalSize<u32>) -> f32 {
    size.width.max(1) as f32 / size.height.max(1) as f32
}

#[cfg(test)]
mod tests {
    use super::{
        BOULDER_COUNT, GRAVEL_PALETTE, GravelLayer, GravelSimulation, LARGE_COUNT,
        MAX_HORIZONTAL_RADIUS, MAX_ROCKS, MEDIUM_COUNT, SMALL_COUNT, seed_state,
    };
    use std::collections::HashSet;

    /// Pinned so pile assertions stay reproducible; production seeds per run.
    const TEST_SEED: u32 = 0x1234_5678;

    #[test]
    fn gravel_spawns_varied_rocks_and_builds_a_pile() {
        let mut simulation = GravelSimulation::new(16.0 / 10.0, TEST_SEED);
        let mut time = 0.0;
        for _ in 0..900 {
            simulation.update(time, 0.0);
            time += 1.0 / 60.0;
        }

        let sizes = simulation
            .rocks
            .iter()
            .map(|rock| rock.size[1].to_bits())
            .collect::<HashSet<_>>();
        let colors = simulation
            .rocks
            .iter()
            .map(|rock| rock.color.map(f32::to_bits))
            .collect::<HashSet<_>>();

        assert_eq!(simulation.rocks.len(), MAX_ROCKS);
        assert!(simulation.rocks.iter().any(|rock| rock.settled));
        assert!(
            simulation
                .heightfields
                .iter()
                .flatten()
                .any(|height| *height < 0.99)
        );
        assert!(sizes.len() > 12);
        assert!(colors.len() >= GRAVEL_PALETTE.len());
    }

    #[test]
    fn gravel_uses_four_ordered_layers_without_tiny_filler() {
        let mut simulation = GravelSimulation::new(16.0 / 9.0, TEST_SEED);
        let mut time = 0.0;
        for _ in 0..900 {
            simulation.update(time, 0.0);
            time += 1.0 / 60.0;
        }

        let count = |layer| {
            simulation
                .rocks
                .iter()
                .filter(|rock| rock.layer == layer)
                .count()
        };
        assert_eq!(count(GravelLayer::Boulder), BOULDER_COUNT);
        assert_eq!(count(GravelLayer::Large), LARGE_COUNT);
        assert_eq!(count(GravelLayer::Medium), MEDIUM_COUNT);
        assert_eq!(count(GravelLayer::Small), SMALL_COUNT);
        assert_eq!(MAX_ROCKS, 1_656);
        assert_eq!(
            GravelLayer::Large.pile_index(),
            GravelLayer::Medium.pile_index()
        );
        assert_ne!(
            GravelLayer::Boulder.pile_index(),
            GravelLayer::Large.pile_index()
        );
        assert_ne!(
            GravelLayer::Small.pile_index(),
            GravelLayer::Medium.pile_index()
        );
        assert!(
            simulation.rocks[..BOULDER_COUNT]
                .iter()
                .all(|rock| rock.layer == GravelLayer::Boulder)
        );
        assert!(
            simulation.rocks[BOULDER_COUNT..BOULDER_COUNT + LARGE_COUNT]
                .iter()
                .all(|rock| rock.layer == GravelLayer::Large)
        );
        assert!(
            simulation
                .rocks
                .iter()
                .all(|rock| rock.size[1] >= GravelLayer::Small.radius_range().0)
        );
        assert!(
            simulation
                .rocks
                .iter()
                .filter(|rock| rock.layer == GravelLayer::Boulder)
                .all(|rock| rock.size[1] >= GravelLayer::Boulder.radius_range().0)
        );
        assert!(simulation.heightfields.iter().all(|heightfield| {
            heightfield
                .iter()
                .any(|height| (*height - 1.0).abs() > f32::EPSILON)
        }));
    }

    #[test]
    fn gravel_layers_build_independent_pile_surfaces() {
        let mut simulation = GravelSimulation::new(16.0 / 9.0, TEST_SEED);
        let mut time = 0.0;
        while simulation.rocks.len() < BOULDER_COUNT {
            simulation.update(time, 0.0);
            time += 1.0 / 60.0;
        }
        for _ in 0..180 {
            simulation.update(time, 0.0);
            time += 1.0 / 60.0;
        }

        let boulder_surface = &simulation.heightfields[GravelLayer::Boulder.pile_index()];
        let small_surface = &simulation.heightfields[GravelLayer::Small.pile_index()];
        assert!(boulder_surface.iter().any(|height| *height < 0.99));
        assert!(
            small_surface
                .iter()
                .all(|height| (*height - 1.0).abs() < f32::EPSILON)
        );
    }

    #[test]
    fn settled_rocks_remain_onscreen_and_in_the_instance_set() {
        let mut simulation = GravelSimulation::new(16.0 / 9.0, TEST_SEED);
        let mut time = 0.0;
        for _ in 0..1_200 {
            simulation.update(time, 0.0);
            time += 1.0 / 60.0;
        }

        assert_eq!(simulation.rocks.len(), MAX_ROCKS);
        assert_eq!(simulation.instances.len(), MAX_ROCKS);
        assert!(simulation.rocks.iter().all(|rock| rock.settled));
        let offscreen = simulation
            .rocks
            .iter()
            .filter(|rock| {
                rock.center[1] + rock.size[1] < 0.0 || rock.center[1] - rock.size[1] > 1.0
            })
            .collect::<Vec<_>>();
        assert!(
            offscreen.is_empty(),
            "offscreen settled rocks: {offscreen:#?}"
        );
    }

    #[test]
    fn removing_the_floor_releases_every_settled_rock() {
        let mut simulation = GravelSimulation::new(16.0 / 9.0, TEST_SEED);
        let mut time = 0.0;
        for _ in 0..180 {
            simulation.update(time, 0.0);
            time += 1.0 / 60.0;
        }
        assert!(simulation.rocks.iter().any(|rock| rock.settled));

        simulation.update(3.1, 0.1);

        assert!(simulation.released);
        assert!(simulation.rocks.iter().all(|rock| !rock.settled));
        assert!(simulation.rocks.iter().all(|rock| rock.velocity > 0.0));
    }

    #[test]
    fn stones_stay_on_screen_on_portrait_and_extreme_aspects() {
        // A rotated or portrait display divides radii by an aspect below one,
        // which used to push stones wider than the viewport and invert the
        // spawn span so they landed off-canvas entirely.
        for aspect in [0.05, 0.4, 0.5625, 1.0, 16.0 / 9.0, 32.0 / 9.0] {
            let mut simulation = GravelSimulation::new(aspect, TEST_SEED);
            let mut time = 0.0;
            for _ in 0..600 {
                simulation.update(time, 0.0);
                time += 1.0 / 60.0;
            }

            assert!(!simulation.rocks.is_empty(), "no rocks at aspect {aspect}");
            for rock in &simulation.rocks {
                assert!(
                    rock.size[0] <= MAX_HORIZONTAL_RADIUS,
                    "stone wider than the clamp at aspect {aspect}: {:?}",
                    rock.size
                );
                assert!(
                    rock.center[0] - rock.size[0] >= -0.001
                        && rock.center[0] + rock.size[0] <= 1.001,
                    "stone spawned off-canvas at aspect {aspect}: \
                     center {:?} size {:?}",
                    rock.center,
                    rock.size
                );
            }
        }
    }

    #[test]
    fn different_seeds_build_different_piles() {
        let run = |seed| {
            let mut simulation = GravelSimulation::new(16.0 / 9.0, seed);
            let mut time = 0.0;
            for _ in 0..600 {
                simulation.update(time, 0.0);
                time += 1.0 / 60.0;
            }
            simulation
                .rocks
                .iter()
                .map(|rock| rock.center[0].to_bits())
                .collect::<Vec<_>>()
        };

        assert_ne!(run(1), run(2));
        assert_eq!(run(7), run(7), "a given seed must stay reproducible");
    }

    #[test]
    fn seeding_never_produces_a_dead_generator() {
        // A zero xorshift state latches at zero and every stone would stack in
        // the same column forever.
        for seed in [0, 1, u32::MAX, 0xa5a5_a5a5] {
            assert_ne!(seed_state(seed), 0, "seed {seed} produced a zero state");
        }
    }

    #[test]
    fn resizing_mid_flourish_keeps_the_pile() {
        let mut simulation = GravelSimulation::new(16.0 / 9.0, TEST_SEED);
        let mut time = 0.0;
        for _ in 0..300 {
            simulation.update(time, 0.0);
            time += 1.0 / 60.0;
        }
        let settled_before = simulation.rocks.iter().filter(|rock| rock.settled).count();
        assert!(settled_before > 0);

        simulation.retarget(4.0 / 3.0);

        assert_eq!(
            simulation.rocks.iter().filter(|rock| rock.settled).count(),
            settled_before,
            "retargeting the aspect must not discard the pile"
        );
        assert!((simulation.aspect - 4.0 / 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn reset_clears_the_pile_and_reseeds_the_floor() {
        let mut simulation = GravelSimulation::new(1.6, TEST_SEED);
        simulation.update(1.0, 0.0);
        simulation.reset(2.0, TEST_SEED);

        assert!(simulation.rocks.is_empty());
        assert!(
            simulation
                .heightfields
                .iter()
                .flatten()
                .all(|height| (*height - 1.0).abs() < f32::EPSILON)
        );
        assert!(!simulation.released);
        assert!((simulation.aspect - 2.0).abs() < f32::EPSILON);
    }
}
