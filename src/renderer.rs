use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use flourish::Flourish;
use thiserror::Error;
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, window::Window};

use crate::doom_fire::{DoomFireSimulation, SIZE as DOOM_SIZE};
use crate::gravel::GravelEffect;

#[derive(Debug, Error)]
pub enum RendererError {
    #[error("could not create a GPU surface: {0}")]
    CreateSurface(#[from] wgpu::CreateSurfaceError),
    #[error("no compatible graphics adapter was found")]
    NoAdapter,
    #[error("could not create a GPU device: {0}")]
    RequestDevice(#[from] wgpu::RequestDeviceError),
    #[error("the window surface exposes no color formats")]
    NoSurfaceFormat,
    #[error("the window surface cannot composite transparent pixels")]
    NoTransparentAlphaMode,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderOutcome {
    /// The frame reached the compositor.
    Presented,
    /// The surface is stale; the caller should resize and try again.
    Reconfigure,
    /// The surface was lost and has been reconfigured in place. This frame was
    /// dropped, but the next one should succeed.
    Recovered,
    /// Nothing was drawn and nothing is wrong (occluded, timed out).
    Skipped,
    /// The surface could not be recovered after repeated attempts.
    SurfaceLost,
    /// The GPU rejected the frame. Reported at most once per flourish.
    ValidationError,
}

/// Mirrors `Uniforms` in `shaders/flourishes.wgsl`. Field names, order, and
/// types must stay in lockstep with that struct.
///
/// `effect_size` is live data, not tail padding: the Doom Fire shader uses it
/// to index the heat buffer. The layout below is deliberately 32 bytes with no
/// implicit gaps, which `bytemuck::Pod` enforces at compile time.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    resolution: [f32; 2],
    time: f32,
    exit_progress: f32,
    effect_id: u32,
    seed: u32,
    effect_size: [f32; 2],
}

pub struct FlourishRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    uniforms: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    doom_fire: DoomFireSimulation,
    gravel: GravelEffect,
    size: PhysicalSize<u32>,
    seed: u32,
    reported_validation_error: bool,
    consecutive_surface_losses: u32,
}

/// How many back-to-back surface losses to absorb before treating the surface
/// as genuinely gone rather than momentarily disturbed by a display change.
const MAX_SURFACE_RECOVERY_ATTEMPTS: u32 = 3;

impl FlourishRenderer {
    pub async fn new(window: Arc<Window>) -> Result<Self, RendererError> {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_with_display_handle(
            Box::new(Arc::clone(&window)),
        ));
        let surface = instance.create_surface(window)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
                apply_limit_buckets: false,
            })
            .await
            .map_err(|_| RendererError::NoAdapter)?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Flourish device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
            })
            .await?;

        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .or_else(|| capabilities.formats.first().copied())
            .ok_or(RendererError::NoSurfaceFormat)?;
        let alpha_mode = select_alpha_mode(&capabilities.alpha_modes)
            .ok_or(RendererError::NoTransparentAlphaMode)?;

        let width = size.width.max(1);
        let height = size.height.max(1);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 2,
            alpha_mode,
            view_formats: vec![],
            color_space: wgpu::SurfaceColorSpace::Auto,
        };
        surface.configure(&device, &config);

        let seed = fresh_seed();
        let doom_fire = DoomFireSimulation::new(&device);
        let gravel = GravelEffect::new(&device, format, size, seed);
        let (pipeline, uniform_buffer, bind_group) = create_pipeline(
            &device,
            format,
            size_to_resolution(size),
            seed,
            doom_fire.render_layout(),
        );

        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            uniforms: uniform_buffer,
            bind_group,
            doom_fire,
            gravel,
            size,
            seed,
            reported_validation_error: false,
            consecutive_surface_losses: 0,
        })
    }

    /// A lost surface is usually a transient display change (mode switch, hot
    /// plug, GPU handoff) rather than a dead device, so reconfigure and let the
    /// next frame try again instead of abandoning the overlay outright.
    fn recover_lost_surface(&mut self) -> RenderOutcome {
        self.consecutive_surface_losses += 1;
        if self.consecutive_surface_losses > MAX_SURFACE_RECOVERY_ATTEMPTS {
            return RenderOutcome::SurfaceLost;
        }

        self.surface.configure(&self.device, &self.config);
        RenderOutcome::Recovered
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }

        self.size = size;
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
        self.gravel.resize(size);
    }

    /// Re-seeds and rewinds any stateful simulation so a repeat performance of
    /// the same flourish does not replay the previous one frame for frame.
    pub fn start_effect(&mut self, effect: Flourish) {
        self.seed = fresh_seed();
        self.reported_validation_error = false;
        match effect {
            Flourish::DoomFire => self.doom_fire.reset(&self.queue, self.seed),
            Flourish::GravelFall => self.gravel.reset(self.size, self.seed),
            _ => {}
        }
    }

    pub fn render(&mut self, effect: Flourish, time: f32, exit_progress: f32) -> RenderOutcome {
        let uniforms = Uniforms {
            resolution: size_to_resolution(self.size),
            time,
            exit_progress,
            effect_id: effect.shader_id(),
            seed: self.seed,
            effect_size: DOOM_SIZE,
        };
        self.queue
            .write_buffer(&self.uniforms, 0, bytemuck::bytes_of(&uniforms));
        if effect == Flourish::GravelFall {
            self.gravel.prepare(&self.queue, time, exit_progress);
        }

        let (frame, reconfigure_after_present) = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => {
                self.consecutive_surface_losses = 0;
                (frame, false)
            }
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                self.consecutive_surface_losses = 0;
                (frame, true)
            }
            wgpu::CurrentSurfaceTexture::Outdated => return RenderOutcome::Reconfigure,
            wgpu::CurrentSurfaceTexture::Lost => return self.recover_lost_surface(),
            wgpu::CurrentSurfaceTexture::Timeout | wgpu::CurrentSurfaceTexture::Occluded => {
                return RenderOutcome::Skipped;
            }
            wgpu::CurrentSurfaceTexture::Validation => {
                // A persistent validation failure would otherwise log on every
                // frame for the whole exit animation; once per flourish is
                // enough to diagnose it.
                if !self.reported_validation_error {
                    self.reported_validation_error = true;
                    return RenderOutcome::ValidationError;
                }
                return RenderOutcome::Skipped;
            }
        };
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Flourish frame encoder"),
            });
        if effect == Flourish::DoomFire {
            self.doom_fire
                .encode_step(&mut encoder, &self.queue, time, exit_progress);
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Flourish render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if effect == Flourish::GravelFall {
                self.gravel.render(&mut pass);
            } else {
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &self.bind_group, &[]);
                pass.set_bind_group(1, self.doom_fire.render_bind_group(), &[]);
                pass.draw(0..3, 0..1);
            }
        }
        self.queue.submit(Some(encoder.finish()));
        self.queue.present(frame);
        if reconfigure_after_present {
            self.surface.configure(&self.device, &self.config);
        }
        RenderOutcome::Presented
    }
}

fn create_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    resolution: [f32; 2],
    seed: u32,
    doom_fire_layout: &wgpu::BindGroupLayout,
) -> (wgpu::RenderPipeline, wgpu::Buffer, wgpu::BindGroup) {
    let uniforms = Uniforms {
        resolution,
        time: 0.0,
        exit_progress: 0.0,
        effect_id: Flourish::Curtain.shader_id(),
        seed,
        effect_size: DOOM_SIZE,
    };
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Flourish uniforms"),
        contents: bytemuck::bytes_of(&uniforms),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Flourish bind group layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Flourish bind group"),
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Flourish shader catalog"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders/flourishes.wgsl").into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Flourish pipeline layout"),
        bind_group_layouts: &[Some(&bind_group_layout), Some(doom_fire_layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Flourish pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vertex_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
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

    (pipeline, uniform_buffer, bind_group)
}

/// A fresh seed for one performance of a flourish.
///
/// Procedural effects are otherwise bit-identical on every run, which makes a
/// tool built to delight an audience feel canned the second time they see it.
/// Falls back to a fixed value if the clock is unavailable; a repeated flourish
/// is a far smaller problem than a panic mid-presentation.
fn fresh_seed() -> u32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    // The wall clock alone is not enough: its granularity is coarser than the
    // gap between two quick calls, so back-to-back seeds can land in the same
    // tick and repeat. A counter guarantees they always differ, while the clock
    // keeps separate launches from sharing a sequence.
    static COUNTER: AtomicU32 = AtomicU32::new(0);

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0x9e37_79b9, |elapsed| elapsed.subsec_nanos());
    let counted = COUNTER
        .fetch_add(1, Ordering::Relaxed)
        .wrapping_mul(0x9e37_79b9);
    // Avalanche the low-entropy inputs so nearby launches do not produce
    // visually similar output.
    let mut mixed = nanos ^ counted ^ 0x2545_f491;
    mixed ^= mixed >> 16;
    mixed = mixed.wrapping_mul(0x7feb_352d);
    mixed ^= mixed >> 15;
    mixed = mixed.wrapping_mul(0x846c_a68b);
    mixed ^ (mixed >> 16)
}

fn select_alpha_mode(modes: &[wgpu::CompositeAlphaMode]) -> Option<wgpu::CompositeAlphaMode> {
    modes
        .contains(&wgpu::CompositeAlphaMode::PreMultiplied)
        .then_some(wgpu::CompositeAlphaMode::PreMultiplied)
        .or_else(|| {
            modes
                .contains(&wgpu::CompositeAlphaMode::PostMultiplied)
                .then_some(wgpu::CompositeAlphaMode::PostMultiplied)
        })
}

#[allow(clippy::cast_precision_loss)]
fn size_to_resolution(size: PhysicalSize<u32>) -> [f32; 2] {
    // Physical display dimensions are many orders of magnitude below the first
    // u32 values that f32 cannot represent usefully as pixel coordinates.
    [size.width as f32, size.height as f32]
}

#[cfg(test)]
mod tests {
    use super::{Uniforms, fresh_seed, select_alpha_mode};
    use flourish::Flourish;
    use wgpu::CompositeAlphaMode;

    #[test]
    fn premultiplied_alpha_is_preferred_regardless_of_reported_order() {
        let modes = [
            CompositeAlphaMode::PostMultiplied,
            CompositeAlphaMode::PreMultiplied,
        ];

        assert_eq!(
            select_alpha_mode(&modes),
            Some(CompositeAlphaMode::PreMultiplied)
        );
    }

    #[test]
    fn uniform_layout_matches_the_shader_struct() {
        // WGSL requires a uniform struct to be a multiple of 16 bytes, and
        // every field here has a counterpart in shaders/flourishes.wgsl at the
        // same offset. Drifting from that produces silently wrong visuals
        // rather than a compile error, so pin it down.
        assert_eq!(size_of::<Uniforms>(), 32);
        assert_eq!(size_of::<Uniforms>() % 16, 0);
        assert_eq!(std::mem::offset_of!(Uniforms, resolution), 0);
        assert_eq!(std::mem::offset_of!(Uniforms, time), 8);
        assert_eq!(std::mem::offset_of!(Uniforms, exit_progress), 12);
        assert_eq!(std::mem::offset_of!(Uniforms, effect_id), 16);
        assert_eq!(std::mem::offset_of!(Uniforms, seed), 20);
        assert_eq!(std::mem::offset_of!(Uniforms, effect_size), 24);
    }

    #[test]
    fn the_shader_catalog_covers_every_id_it_is_asked_to_draw() {
        // flourishes.wgsl switches on these exact ids. Gravel is the one
        // effect that never reaches that shader.
        let catalog_ids = Flourish::ALL
            .iter()
            .copied()
            .filter(|effect| !effect.has_dedicated_pipeline())
            .map(Flourish::shader_id)
            .collect::<Vec<_>>();

        assert_eq!(catalog_ids.len(), Flourish::ALL.len() - 1);
        assert!(!catalog_ids.contains(&Flourish::GravelFall.shader_id()));
    }

    #[test]
    fn successive_seeds_differ() {
        // Identical seeds would make every performance a rerun of the last.
        let seeds = (0..8).map(|_| fresh_seed()).collect::<Vec<_>>();
        assert!(
            seeds.windows(2).any(|pair| pair[0] != pair[1]),
            "seeds were all identical: {seeds:?}"
        );
    }
}
