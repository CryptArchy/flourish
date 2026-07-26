struct VertexInput {
    @location(0) center: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) rotation: f32,
    @location(4) seed: f32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) local: vec2<f32>,
    @location(2) @interpolate(flat) seed: f32,
};

fn rough_radius(seed: f32, corner: u32) -> f32 {
    let phase = seed * 1.618 + f32(corner) * 12.9898;
    return 0.84 + 0.16 * fract(sin(phase) * 43758.5453);
}

@vertex
fn vertex_main(input: VertexInput, @builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    let segment = vertex_index / 3u;
    let triangle_vertex = vertex_index % 3u;
    var local = vec2<f32>(0.0);
    if triangle_vertex > 0u {
        let corner = segment + triangle_vertex - 1u;
        let angle = f32(corner) / 9.0 * 6.2831853;
        local = vec2<f32>(cos(angle), sin(angle)) * rough_radius(input.seed, corner);
    }

    let rotated = vec2<f32>(
        local.x * cos(input.rotation) - local.y * sin(input.rotation),
        local.x * sin(input.rotation) + local.y * cos(input.rotation),
    );
    let screen = input.center + rotated * input.size;

    var output: VertexOutput;
    output.position = vec4<f32>(screen.x * 2.0 - 1.0, 1.0 - screen.y * 2.0, 0.0, 1.0);
    output.color = input.color;
    output.local = local;
    output.seed = input.seed;
    return output;
}

fn hash21(point: vec2<f32>) -> f32 {
    var p = fract(point * vec2<f32>(123.34, 456.21));
    p += dot(p, p + 45.32);
    return fract(p.x * p.y);
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let light = normalize(vec2<f32>(-0.55, -0.84));
    let facing = 0.78 + 0.20 * max(dot(normalize(input.local + vec2<f32>(0.001)), -light), 0.0);
    let edge = smoothstep(0.58, 1.02, length(input.local));
    let grain = hash21(floor(input.local * 17.0) + input.seed) - 0.5;
    let shade = facing - edge * 0.16 + grain * 0.055;
    // Instance alpha carries overall presence, which is how the calm path
    // cross-fades a pipeline that has no uniforms of its own. Premultiplied,
    // to match the shared catalog and the surface's composite mode.
    let alpha = clamp(input.color.a, 0.0, 1.0);
    return vec4<f32>(input.color.rgb * shade * alpha, alpha);
}
