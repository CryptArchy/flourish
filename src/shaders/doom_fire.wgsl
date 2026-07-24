struct ComputeUniforms {
    width: u32,
    height: u32,
    frame: u32,
    source_strength: u32,
};

@group(0) @binding(0)
var<storage, read> source_heat: array<u32>;

@group(0) @binding(1)
var<storage, read_write> destination_heat: array<u32>;

@group(0) @binding(2)
var<uniform> uniforms: ComputeUniforms;

fn hash(value: u32) -> u32 {
    var mixed = value;
    mixed = mixed ^ (mixed >> 16u);
    mixed = mixed * 0x7feb352du;
    mixed = mixed ^ (mixed >> 15u);
    mixed = mixed * 0x846ca68bu;
    return mixed ^ (mixed >> 16u);
}

@compute @workgroup_size(256)
fn compute_main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    let cell_count = uniforms.width * uniforms.height;
    if index >= cell_count {
        return;
    }

    let x = index % uniforms.width;
    let y = index / uniforms.width;
    if y == uniforms.height - 1u {
        destination_heat[index] = uniforms.source_strength;
        return;
    }

    let random = hash(index ^ (uniforms.frame * 0x9e3779b9u));
    let lateral = i32((random >> 8u) % 3u) - 1;
    let source_x = u32(clamp(i32(x) + lateral, 0, i32(uniforms.width) - 1));
    let source_index = (y + 1u) * uniforms.width + source_x;
    let source_value = source_heat[source_index];
    let cooling = random & 1u;
    destination_heat[index] = select(0u, source_value - cooling, source_value > cooling);
}
