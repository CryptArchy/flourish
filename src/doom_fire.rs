use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

pub const WIDTH: u32 = 640;
pub const HEIGHT: u32 = 144;
pub const SIZE: [f32; 2] = [640.0, 144.0];
const MAX_HEAT: u32 = 36;
const WORKGROUP_SIZE: u32 = 256;
const STEP_SECONDS: f32 = 1.0 / 30.0;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ComputeUniforms {
    width: u32,
    height: u32,
    frame: u32,
    source_strength: u32,
}

pub struct DoomFireSimulation {
    buffers: [wgpu::Buffer; 2],
    compute_bind_groups: [wgpu::BindGroup; 2],
    render_bind_groups: [wgpu::BindGroup; 2],
    render_layout: wgpu::BindGroupLayout,
    compute_pipeline: wgpu::ComputePipeline,
    compute_uniforms: wgpu::Buffer,
    current: usize,
    frame: u32,
    last_step_time: f32,
}

impl DoomFireSimulation {
    pub fn new(device: &wgpu::Device) -> Self {
        let cells = initial_cells();
        let make_buffer = |label| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(label),
                contents: bytemuck::cast_slice(&cells),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            })
        };
        let buffers = [
            make_buffer("Doom fire heat A"),
            make_buffer("Doom fire heat B"),
        ];

        let compute_uniforms = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Doom fire compute uniforms"),
            contents: bytemuck::bytes_of(&ComputeUniforms {
                width: WIDTH,
                height: HEIGHT,
                frame: 0,
                source_strength: MAX_HEAT,
            }),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let compute_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Doom fire compute layout"),
            entries: &[
                storage_entry(0, true, wgpu::ShaderStages::COMPUTE),
                storage_entry(1, false, wgpu::ShaderStages::COMPUTE),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let compute_bind_groups = [
            compute_bind_group(
                device,
                &compute_layout,
                &buffers[0],
                &buffers[1],
                &compute_uniforms,
                "Doom fire compute A to B",
            ),
            compute_bind_group(
                device,
                &compute_layout,
                &buffers[1],
                &buffers[0],
                &compute_uniforms,
                "Doom fire compute B to A",
            ),
        ];
        let compute_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Doom fire compute shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/doom_fire.wgsl").into()),
        });
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Doom fire compute pipeline layout"),
                bind_group_layouts: &[Some(&compute_layout)],
                immediate_size: 0,
            });
        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Doom fire compute pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_shader,
            entry_point: Some("compute_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        let render_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Doom fire render layout"),
            entries: &[storage_entry(0, true, wgpu::ShaderStages::FRAGMENT)],
        });
        let render_bind_groups = [
            render_bind_group(device, &render_layout, &buffers[0], "Doom fire render A"),
            render_bind_group(device, &render_layout, &buffers[1], "Doom fire render B"),
        ];

        Self {
            buffers,
            compute_bind_groups,
            render_bind_groups,
            render_layout,
            compute_pipeline,
            compute_uniforms,
            current: 0,
            frame: 0,
            last_step_time: -STEP_SECONDS,
        }
    }

    pub fn render_layout(&self) -> &wgpu::BindGroupLayout {
        &self.render_layout
    }

    pub fn render_bind_group(&self) -> &wgpu::BindGroup {
        &self.render_bind_groups[self.current]
    }

    /// Rewinds the heat field. The frame counter starts from `seed` rather than
    /// zero because it drives the propagation hash, so a fresh seed gives the
    /// fire a different shape on every performance.
    pub fn reset(&mut self, queue: &wgpu::Queue, seed: u32) {
        let cells = initial_cells();
        for buffer in &self.buffers {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&cells));
        }
        self.current = 0;
        self.frame = seed;
        self.last_step_time = -STEP_SECONDS;
    }

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::cast_sign_loss
    )]
    pub fn encode_step(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &wgpu::Queue,
        effect_time: f32,
        exit_progress: f32,
    ) {
        if effect_time < self.last_step_time + STEP_SECONDS {
            return;
        }
        self.last_step_time = effect_time;
        self.frame = self.frame.wrapping_add(1);
        let source_strength =
            (MAX_HEAT as f32 * (1.0 - exit_progress.clamp(0.0, 1.0))).round() as u32;
        queue.write_buffer(
            &self.compute_uniforms,
            0,
            bytemuck::bytes_of(&ComputeUniforms {
                width: WIDTH,
                height: HEIGHT,
                frame: self.frame,
                source_strength,
            }),
        );

        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Doom fire propagation"),
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.compute_pipeline);
            pass.set_bind_group(0, &self.compute_bind_groups[self.current], &[]);
            pass.dispatch_workgroups((WIDTH * HEIGHT).div_ceil(WORKGROUP_SIZE), 1, 1);
        }
        self.current = 1 - self.current;
    }
}

fn storage_entry(
    binding: u32,
    read_only: bool,
    visibility: wgpu::ShaderStages,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn compute_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    source: &wgpu::Buffer,
    destination: &wgpu::Buffer,
    uniforms: &wgpu::Buffer,
    label: &'static str,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: source.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: destination.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: uniforms.as_entire_binding(),
            },
        ],
    })
}

fn render_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    buffer: &wgpu::Buffer,
    label: &'static str,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }],
    })
}

fn initial_cells() -> Vec<u32> {
    let mut cells = vec![0; (WIDTH * HEIGHT) as usize];
    let bottom_start = (WIDTH * (HEIGHT - 1)) as usize;
    cells[bottom_start..].fill(MAX_HEAT);
    cells
}

#[cfg(test)]
mod tests {
    use super::{HEIGHT, MAX_HEAT, WIDTH, initial_cells};

    #[test]
    fn initial_heat_field_only_seeds_the_bottom_row() {
        let cells = initial_cells();
        let bottom_start = (WIDTH * (HEIGHT - 1)) as usize;

        assert_eq!(cells.len(), (WIDTH * HEIGHT) as usize);
        assert!(cells[..bottom_start].iter().all(|heat| *heat == 0));
        assert!(cells[bottom_start..].iter().all(|heat| *heat == MAX_HEAT));
    }
}
