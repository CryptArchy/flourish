// Mirrors `Uniforms` in renderer.rs. Field order, types, and offsets must stay
// in lockstep; a test there pins the Rust side's layout.
struct Uniforms {
    resolution: vec2<f32>,
    time: f32,
    exit_progress: f32,
    effect_id: u32,
    seed: u32,
    // Dimensions of the Doom Fire heat field. Live data, not padding.
    effect_size: vec2<f32>,
    // Overall opacity. Carries the entrance and exit when motion is reduced.
    presence: f32,
};

// Effect ids. These must match `Flourish::shader_id` in lib.rs.
const EFFECT_CURTAIN: u32 = 0u;
const EFFECT_POND_RIPPLES: u32 = 1u;
const EFFECT_FIRE: u32 = 2u;
const EFFECT_BLACKOUT: u32 = 3u;
const EFFECT_KALEIDOSCOPE: u32 = 4u;
const EFFECT_MOSAIC: u32 = 5u;
const EFFECT_DOOM_FIRE: u32 = 6u;
// Id 7 is Gravel Fall, which draws through its own pipeline and never reaches
// this shader. It is deliberately absent from the switch below.
const EFFECT_PROJECTOR_IRIS: u32 = 8u;
const EFFECT_GEOLOGICAL_STRATA: u32 = 9u;
const EFFECT_FROSTED_GLASS: u32 = 10u;
const EFFECT_CRT_SHUTDOWN: u32 = 11u;

// Must match `MAX_HEAT` in doom_fire.rs.
const DOOM_MAX_HEAT: f32 = 36.0;

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var<storage, read> doom_heat: array<u32>;

// A per-performance offset into every procedural domain, so a flourish shown
// twice in one talk is not the same picture twice.
fn seed_offset() -> vec2<f32> {
    let low = f32(uniforms.seed & 0xffffu) / 65536.0;
    let high = f32((uniforms.seed >> 16u) & 0xffffu) / 65536.0;
    return vec2<f32>(low, high) * 128.0;
}

fn seed_phase() -> f32 {
    return f32(uniforms.seed & 0xffffu) / 65536.0 * 6.2831853;
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn vertex_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );
    var output: VertexOutput;
    output.position = vec4<f32>(positions[vertex_index], 0.0, 1.0);
    return output;
}

fn ease_in_out_cubic(value: f32) -> f32 {
    if value < 0.5 {
        return 4.0 * value * value * value;
    }
    let shifted = -2.0 * value + 2.0;
    return 1.0 - shifted * shifted * shifted * 0.5;
}

// Both generators fold in the per-performance seed, so every hash-driven
// detail (sparks, grain, frost cells, pebbles, tile colors) reshuffles between
// runs while all timing and easing stay exactly as authored.
fn hash21(point: vec2<f32>) -> f32 {
    var p = fract((point + seed_offset()) * vec2<f32>(123.34, 456.21));
    p += dot(p, p + 45.32);
    return fract(p.x * p.y);
}

fn value_noise(point: vec2<f32>) -> f32 {
    // Inherits the seed through its hash21 lattice samples; offsetting here as
    // well would only re-randomize an already-randomized field.
    let cell = floor(point);
    let local = fract(point);
    let blend = local * local * (3.0 - 2.0 * local);
    let low = mix(hash21(cell), hash21(cell + vec2<f32>(1.0, 0.0)), blend.x);
    let high = mix(
        hash21(cell + vec2<f32>(0.0, 1.0)),
        hash21(cell + vec2<f32>(1.0, 1.0)),
        blend.x,
    );
    return mix(low, high, blend.y);
}

// Every effect ends here, which is what lets one multiply give the whole
// catalog a cross-fade when motion is reduced.
fn composite(color: vec3<f32>, alpha: f32) -> vec4<f32> {
    let safe_alpha = clamp(alpha, 0.0, 1.0) * clamp(uniforms.presence, 0.0, 1.0);
    // Premultiplication is intentional even on the post-multiplied fallback:
    // no transparent frame may retain bright RGB for the window compositor.
    return vec4<f32>(color * safe_alpha, safe_alpha);
}

fn panel_width(y: f32, progress: f32, side: f32) -> f32 {
    let eased = ease_in_out_cubic(progress);
    let motion = sin(progress * 3.14159265);
    let lower_lag = 0.075 * smoothstep(0.15, 1.0, y) * motion;
    let local_progress = clamp(eased - lower_lag, 0.0, 1.0);
    let settling = 0.009 * sin(progress * 18.0 + y * 4.0 + side) * motion * y;
    return max(0.0, 0.505 * (1.0 - local_progress) + settling);
}

fn fabric_spot(point: vec2<f32>, center: vec2<f32>, radius: f32, lean: f32) -> f32 {
    var delta = point - center;
    delta.x += delta.y * lean;
    delta.x *= uniforms.resolution.x / uniforms.resolution.y;
    let radial = length(delta) / radius;
    return 1.0 - smoothstep(0.08, 1.0, radial);
}

fn curtain(uv: vec2<f32>) -> vec4<f32> {
    let progress = clamp(uniforms.exit_progress, 0.0, 1.0);
    let left_width = panel_width(uv.y, progress, 0.0);
    let right_width = panel_width(uv.y, progress, 2.2);
    let pixel = 1.5 / uniforms.resolution.x;

    let left_alpha = 1.0 - smoothstep(left_width - pixel, left_width + pixel, uv.x);
    let right_edge = 1.0 - right_width;
    let right_alpha = smoothstep(right_edge - pixel, right_edge + pixel, uv.x);
    let alpha = max(left_alpha, right_alpha);
    if alpha <= 0.001 {
        return vec4<f32>(0.0);
    }

    let is_left = left_alpha >= right_alpha;
    let width = select(max(right_width, 0.0001), max(left_width, 0.0001), is_left);
    let from_outer = select((1.0 - uv.x) / width, uv.x / width, is_left);
    let side = select(-1.0, 1.0, is_left);

    // Curtain is woven entirely from sines, so it needs the seed applied
    // directly; the hash-driven effects pick it up through hash21.
    let drift = seed_phase();
    let rustle = 0.13 * sin(uv.y * 8.0 + uniforms.time * 0.72 + drift)
        + 0.05 * sin(uv.y * 19.0 - uniforms.time * 0.41 + drift * 1.7);
    let phase = from_outer * 18.0 * 3.14159265 + rustle + side * 0.3;
    let fold = 0.5 + 0.5 * cos(phase);
    let fine_fold = 0.5 + 0.5 * cos(phase * 2.0 + uv.y * 2.5);
    let broad_shadow = mix(0.34, 1.06, pow(fold, 0.76));
    let velvet_glint = 0.075 * pow(fine_fold, 7.0);

    let inner_distance = select(uv.x - right_edge, left_width - uv.x, is_left);
    let edge_highlight = 0.12 * exp(-max(inner_distance, 0.0) * 105.0);
    let center_shadow = 1.0 - 0.30 * exp(-max(inner_distance, 0.0) * 48.0);
    let vertical_falloff = mix(1.02, 0.67, smoothstep(0.0, 1.0, uv.y));

    // Recover the cloth's rest coordinate so the circular pools are painted
    // onto the fabric before its folds and opening compression are applied.
    let fabric_x = select(1.0 - from_outer * 0.5, from_outer * 0.5, is_left);
    let fold_warp = 0.010 * sin(phase) * mix(0.35, 1.0, uv.y);
    let fabric_uv = vec2<f32>(
        fabric_x + fold_warp,
        uv.y + 0.004 * sin(phase * 0.5 + rustle),
    );
    let left_pool = fabric_spot(fabric_uv, vec2<f32>(0.22, 0.34), 0.31, 0.22);
    let center_pool = fabric_spot(fabric_uv, vec2<f32>(0.50, 0.30), 0.34, 0.0);
    let right_pool = fabric_spot(fabric_uv, vec2<f32>(0.78, 0.34), 0.31, -0.22);
    let pools = max(center_pool, max(left_pool, right_pool) * 0.84);

    let theater_black = vec3<f32>(0.035, 0.001, 0.007);
    let velvet = vec3<f32>(0.115, 0.004, 0.020);
    let lit_velvet = vec3<f32>(0.31, 0.022, 0.060);
    let cloth_base = mix(mix(theater_black, velvet, 0.58), lit_velvet, pools * 0.72);
    var color = cloth_base * broad_shadow * center_shadow * vertical_falloff;
    color += lit_velvet * (velvet_glint * 0.45 + edge_highlight * 0.32) * (0.35 + pools * 0.65);
    color += vec3<f32>(0.16, 0.045, 0.012) * pools * pow(fold, 1.7) * 0.18;

    let center_piping = 1.0 - smoothstep(0.003, 0.012, max(inner_distance, 0.0));
    let lower_band = smoothstep(0.944, 0.958, uv.y);
    let lower_edge = smoothstep(0.929, 0.938, uv.y) * (1.0 - smoothstep(0.947, 0.956, uv.y));
    let braid = 0.74 + 0.26 * sin((uv.x * uniforms.resolution.x + uv.y * 55.0) * 0.075);
    let gold = vec3<f32>(0.84, 0.57, 0.14);
    let gold_light = vec3<f32>(1.0, 0.83, 0.37);
    let trim = max(center_piping, max(lower_band * braid, lower_edge));
    color = mix(color, mix(gold, gold_light, 0.30 + pools * 0.24), trim * alpha);
    return composite(color, alpha);
}

fn pond_ripples(uv: vec2<f32>) -> vec4<f32> {
    let exit = ease_in_out_cubic(clamp(uniforms.exit_progress, 0.0, 1.0));
    let remain = 1.0 - exit;
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    var wave = 0.0;
    var glint = 0.0;
    let impact_centers = array<vec2<f32>, 7>(
        vec2<f32>(0.12, 0.23),
        vec2<f32>(0.39, 0.16),
        vec2<f32>(0.73, 0.25),
        vec2<f32>(0.89, 0.51),
        vec2<f32>(0.64, 0.74),
        vec2<f32>(0.31, 0.67),
        vec2<f32>(0.09, 0.84),
    );
    for (var index = 0; index < 7; index += 1) {
        let seed = f32(index);
        let center = impact_centers[index];
        var delta = uv - center;
        delta.x *= aspect;
        let distance_from_impact = length(delta);
        // Stagger which impacts lead so the pond is not the same rain twice.
        let age = fract(uniforms.time * 0.135 + seed * 0.173 + seed_phase());
        let radius = age * 0.48;
        let life = sin(age * 3.14159265);
        let width = mix(0.005, 0.016, age);
        let primary = exp(-pow((distance_from_impact - radius) / width, 2.0));
        let echo_radius = max(radius - 0.038, 0.0);
        let echo = exp(-pow((distance_from_impact - echo_radius) / (width * 1.25), 2.0));
        wave += (primary + echo * 0.42) * life;
        glint += primary * life * (0.65 + 0.35 * sin(seed * 8.1 + uniforms.time * 0.5));
    }

    let calm_shimmer = 0.5 + 0.5 * sin((uv.x * aspect + uv.y) * 18.0 + uniforms.time * 0.22);
    let alpha = (0.025 + wave * 0.30) * remain * remain;
    let deep_water = vec3<f32>(0.055, 0.20, 0.28);
    let reflected_sky = vec3<f32>(0.50, 0.82, 0.91);
    let color = deep_water * (0.40 + calm_shimmer * 0.10) + reflected_sky * glint * 0.80;
    return composite(color, alpha);
}

fn fire(uv: vec2<f32>) -> vec4<f32> {
    let exit = ease_in_out_cubic(clamp(uniforms.exit_progress, 0.0, 1.0));
    let remain = 1.0 - exit;
    let upward = 1.0 - uv.y;
    let coarse = value_noise(vec2<f32>(uv.x * 7.0, uv.y * 3.5 - uniforms.time * 1.7));
    let fine = value_noise(vec2<f32>(uv.x * 19.0 + 4.2, uv.y * 8.0 - uniforms.time * 3.2));
    let tongues = 0.19 + coarse * 0.18 + fine * 0.055;
    let height = tongues * (0.08 + 0.92 * remain);
    let edge_noise = value_noise(vec2<f32>(
        uv.x * 47.0 + uniforms.time * 0.37,
        uv.y * 21.0 - uniforms.time * 4.4,
    ));
    let edge_detail = value_noise(vec2<f32>(
        uv.x * 91.0 - 3.2,
        uv.y * 37.0 - uniforms.time * 6.8,
    ));
    let breakup = edge_noise * 0.70 + edge_detail * 0.30 - 0.5;
    let signed_edge = height - upward + breakup * 0.070;
    let feather = mix(1.5, 3.5, edge_noise) / max(uniforms.resolution.y, 1.0);
    var flame = smoothstep(-feather, feather, signed_edge);
    let edge_zone = 1.0 - smoothstep(0.0, feather * 7.0, abs(signed_edge));
    let fragment_gate = smoothstep(0.18, 0.70, edge_noise * 0.58 + edge_detail * 0.42);
    flame *= mix(1.0, fragment_gate, edge_zone * 0.58);
    flame = smoothstep(0.08, 0.92, flame);
    let inner = 1.0 - smoothstep(0.015, max(height * 0.66, 0.02), upward);
    let heat = clamp(1.0 - upward / max(height, 0.001), 0.0, 1.0);
    let edge_heat = edge_zone * mix(0.22, 0.60, edge_noise);
    let visible_heat = max(heat, edge_heat);
    let ember = vec3<f32>(0.66, 0.035, 0.008);
    let orange = vec3<f32>(1.0, 0.28, 0.015);
    let yellow = vec3<f32>(1.0, 0.78, 0.16);
    var color = mix(ember, orange, visible_heat);
    color = mix(color, yellow, inner * 0.72);

    let spark_flow = vec2<f32>(uv.x * 30.0, uv.y * 18.0 + uniforms.time * 2.8);
    let spark_cell = floor(spark_flow);
    let spark_local = fract(spark_flow) - 0.5;
    let spark_random = hash21(spark_cell);
    let spark = (1.0 - smoothstep(0.018, 0.085, length(spark_local)))
        * step(0.91, spark_random)
        * smoothstep(0.12, 0.95, uv.y);
    color += yellow * spark * 1.4;
    let alpha = (flame * (0.80 + inner * 0.20) + spark) * remain;
    return composite(color, alpha);
}

fn doom_palette(heat: f32) -> vec3<f32> {
    let black = vec3<f32>(0.0, 0.0, 0.0);
    let blood = vec3<f32>(0.30, 0.004, 0.008);
    let red = vec3<f32>(0.88, 0.035, 0.006);
    let orange = vec3<f32>(1.0, 0.28, 0.0);
    let yellow = vec3<f32>(1.0, 0.82, 0.10);
    let white_hot = vec3<f32>(1.0, 0.98, 0.78);
    if heat < 0.16 {
        return mix(black, blood, heat / 0.16);
    }
    if heat < 0.38 {
        return mix(blood, red, (heat - 0.16) / 0.22);
    }
    if heat < 0.62 {
        return mix(red, orange, (heat - 0.38) / 0.24);
    }
    if heat < 0.84 {
        return mix(orange, yellow, (heat - 0.62) / 0.22);
    }
    return mix(yellow, white_hot, (heat - 0.84) / 0.16);
}

fn doom_fire(uv: vec2<f32>) -> vec4<f32> {
    let size = vec2<u32>(u32(uniforms.effect_size.x), u32(uniforms.effect_size.y));
    let screen_aspect = uniforms.resolution.x / uniforms.resolution.y;
    let portrait_factor = 1.12;
    let visible_columns = min(
        size.x,
        u32(ceil(f32(size.y) * screen_aspect * portrait_factor)),
    );
    let horizontal_offset = (size.x - visible_columns) / 2u;
    let visible_coordinate = min(
        vec2<u32>(uv * vec2<f32>(f32(visible_columns), f32(size.y))),
        vec2<u32>(visible_columns - 1u, size.y - 1u),
    );
    let coordinate = vec2<u32>(horizontal_offset + visible_coordinate.x, visible_coordinate.y);
    let index = coordinate.y * size.x + coordinate.x;
    let heat = f32(doom_heat[index]) / DOOM_MAX_HEAT;
    let exit = ease_in_out_cubic(clamp(uniforms.exit_progress, 0.0, 1.0));
    let alpha = smoothstep(0.0, 0.12, heat) * (1.0 - exit);
    return composite(doom_palette(heat), alpha);
}

fn blackout(uv: vec2<f32>) -> vec4<f32> {
    let exit = ease_in_out_cubic(clamp(uniforms.exit_progress, 0.0, 1.0));
    let coordinate = uv.x + uv.y * 0.28;
    let threshold = mix(-0.06, 1.34, exit);
    let alpha = smoothstep(threshold - 0.018, threshold + 0.018, coordinate);
    let grain = hash21(floor(uv * uniforms.resolution * 0.5) + floor(uniforms.time * 4.0));
    let vignette = 1.0 - smoothstep(0.25, 0.95, distance(uv, vec2<f32>(0.5)));
    let color = vec3<f32>(0.006 + grain * 0.006 + vignette * 0.004);
    return composite(color, alpha);
}

fn kaleidoscope(uv: vec2<f32>) -> vec4<f32> {
    var point = uv - 0.5;
    point.x *= uniforms.resolution.x / uniforms.resolution.y;
    let radius = length(point);
    // Start the mirror at a different rotation each time.
    let angle = atan2(point.y, point.x) + uniforms.time * 0.10 + seed_phase();
    let sector = 3.14159265 * 2.0 / 12.0;
    let folded_angle = abs(fract(angle / sector + 0.5) - 0.5) * sector;
    let folded = vec2<f32>(cos(folded_angle), sin(folded_angle)) * radius;
    let weave = sin(folded.x * 36.0 - uniforms.time * 0.9)
        + sin(folded.y * 46.0 + uniforms.time * 0.7)
        + sin((folded.x + folded.y) * 25.0);
    let facets = 0.5 + 0.5 * sin(weave * 1.7 + radius * 34.0 - uniforms.time * 0.8);
    let palette_phase = vec3<f32>(0.0, 2.1, 4.2) + facets * 4.2 + radius * 8.0;
    var color = vec3<f32>(0.48) + 0.44 * cos(palette_phase);
    color *= 0.72 + 0.28 * cos(folded_angle * 24.0);
    let lead = 1.0 - smoothstep(0.025, 0.075, abs(fract(weave * 0.24) - 0.5));
    color = mix(color, vec3<f32>(0.025, 0.018, 0.045), lead * 0.55);

    let exit = ease_in_out_cubic(clamp(uniforms.exit_progress, 0.0, 1.0));
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let corner_radius = length(vec2<f32>(aspect * 0.5, 0.5));
    let aperture = mix(-0.08, corner_radius + 0.08, exit);
    let alpha = smoothstep(aperture - 0.035, aperture + 0.035, radius);
    return composite(color, alpha);
}

fn mosaic(uv: vec2<f32>) -> vec4<f32> {
    let grid = vec2<f32>(14.0, 9.0);
    let row = floor(uv.y * grid.y);
    let motion = 0.024 * sin(row * 1.73 + uniforms.time * 0.62);
    let tiled = (uv + vec2<f32>(motion, 0.0)) * grid;
    let cell = floor(tiled);
    let local = fract(tiled);
    let random = hash21(cell);
    let pulse = 0.5 + 0.5 * sin(uniforms.time * 0.72 + random * 6.28318);
    let palette = vec3<f32>(0.50) + 0.46 * cos(
        vec3<f32>(0.2, 2.25, 4.3) + random * 8.0 + pulse * 0.8 + row * 0.07,
    );
    let edge_distance = min(min(local.x, 1.0 - local.x), min(local.y, 1.0 - local.y));
    let bevel = smoothstep(0.025, 0.095, edge_distance);
    var color = mix(vec3<f32>(0.018, 0.014, 0.028), palette, bevel);
    color += vec3<f32>(0.12, 0.09, 0.16) * (1.0 - smoothstep(0.04, 0.13, edge_distance));

    let exit = clamp(uniforms.exit_progress, 0.0, 1.0);
    let delay = 0.08 + random * 0.56;
    let local_exit = ease_in_out_cubic(clamp((exit - delay) / 0.34, 0.0, 1.0));
    let scaled = abs(local - 0.5) / max(1.0 - local_exit * 0.76, 0.01);
    let tile_shape = 1.0 - step(0.5, max(scaled.x, scaled.y));
    let alpha = (1.0 - local_exit) * tile_shape;
    return composite(color, alpha);
}

fn projector_iris(uv: vec2<f32>) -> vec4<f32> {
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    var point = uv - 0.5;
    point.x *= aspect;
    let radius = length(point);
    let angle = atan2(point.y, point.x);
    let exit = ease_in_out_cubic(clamp(uniforms.exit_progress, 0.0, 1.0));
    let corner_radius = length(vec2<f32>(aspect * 0.5, 0.5));
    let opening = mix(0.008, corner_radius + 0.075, exit);
    let feather = 2.2 / uniforms.resolution.y;
    let alpha = smoothstep(opening - feather, opening + feather, radius);
    if alpha <= 0.001 {
        return vec4<f32>(0.0);
    }

    let blade_count = 12.0;
    let sector = 6.2831853 / blade_count;
    let mechanism_rotation = exit * 0.58 + sin(uniforms.time * 0.31) * 0.0025;
    let blade_coordinate = (angle + 3.14159265 + mechanism_rotation) / sector;
    let blade_index = floor(blade_coordinate);
    let curved_coordinate = fract(
        blade_coordinate + radius * 0.76 + sin(radius * 11.0) * 0.035,
    );
    let seam_distance = min(curved_coordinate, 1.0 - curved_coordinate);
    let overlap_seam = 1.0 - smoothstep(0.012, 0.060, seam_distance);

    let carbon = vec3<f32>(0.007, 0.008, 0.008);
    let gunmetal = vec3<f32>(0.090, 0.104, 0.108);
    let worn_steel = vec3<f32>(0.188, 0.202, 0.202);
    let blade_variation = hash21(vec2<f32>(blade_index, 4.73));
    let facet = 0.42 + 0.24 * cos((fract(blade_coordinate) - 0.5) * 3.14159265);
    let brushed = 0.5 + 0.5 * sin(radius * 285.0 + blade_index * 5.1);
    var color = mix(carbon, gunmetal, facet + blade_variation * 0.13);
    color = mix(color, worn_steel, brushed * 0.055);
    color *= 1.0 - overlap_seam * 0.52;

    let tungsten = vec3<f32>(0.835, 0.535, 0.215);
    let hot_edge = vec3<f32>(1.0, 0.755, 0.385);
    let rim = exp(-abs(radius - opening) * 105.0);
    color += mix(tungsten, hot_edge, blade_variation) * rim * (0.34 + exit * 0.24);

    let dust_cell = floor(point * uniforms.resolution.y * 0.32);
    let dust = step(0.988, hash21(dust_cell + 17.0))
        * (0.35 + 0.65 * hash21(dust_cell + 31.0));
    color += vec3<f32>(0.19, 0.17, 0.13) * dust * 0.16;
    return composite(color, alpha);
}

fn strata_fault(y: f32) -> f32 {
    return 0.5
        + sin(y * 12.7 + 0.4) * 0.022
        + sin(y * 31.0 + 2.1) * 0.010
        + (value_noise(vec2<f32>(y * 8.0, 3.7)) - 0.5) * 0.030;
}

fn inside_screen(point: vec2<f32>) -> bool {
    return all(point >= vec2<f32>(0.0)) && all(point <= vec2<f32>(1.0));
}

fn frost_cell_edge(point: vec2<f32>, scale: f32, seed: f32) -> f32 {
    let scaled = point * scale;
    let cell = floor(scaled);
    let local = fract(scaled);
    var nearest = 10.0;
    var second_nearest = 10.0;
    for (var offset_y: i32 = -1; offset_y <= 1; offset_y += 1) {
        for (var offset_x: i32 = -1; offset_x <= 1; offset_x += 1) {
            let offset = vec2<f32>(f32(offset_x), f32(offset_y));
            let identifier = cell + offset;
            let feature = offset
                + vec2<f32>(
                    hash21(identifier + seed),
                    hash21(identifier + vec2<f32>(17.1, 43.7) + seed),
                )
                - local;
            let feature_distance = length(feature);
            if feature_distance < nearest {
                second_nearest = nearest;
                nearest = feature_distance;
            } else if feature_distance < second_nearest {
                second_nearest = feature_distance;
            }
        }
    }
    return 1.0 - smoothstep(0.025, 0.125, second_nearest - nearest);
}

fn frost_bloom(point: vec2<f32>, center: vec2<f32>, seed: f32) -> f32 {
    let delta = point - center;
    let radius = length(delta);
    let domain_warp = (value_noise(delta * 4.7 + seed * 2.3) - 0.5) * 0.62;
    let angle = atan2(delta.y, delta.x)
        + domain_warp
        + sin(radius * 23.0 + seed * 4.1) * 0.11;
    let branches = 5.0 + fract(seed * 1.73) * 3.0;
    let primary = 1.0 - smoothstep(0.025, 0.165, abs(sin(angle * branches)));
    let twigs = 1.0 - smoothstep(
        0.030,
        0.145,
        abs(sin(angle * branches * 2.0 + radius * 47.0 + seed)),
    );
    let envelope = 1.0 - smoothstep(0.055, 0.61, radius);
    return max(primary, twigs * 0.56) * envelope;
}

fn frosted_glass(uv: vec2<f32>) -> vec4<f32> {
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    var point = uv - 0.5;
    point.x *= aspect;
    let exit = ease_in_out_cubic(clamp(uniforms.exit_progress, 0.0, 1.0));

    let edge_distance = min(min(uv.x, 1.0 - uv.x), min(uv.y, 1.0 - uv.y));
    let growth_noise = value_noise(point * 3.2 + vec2<f32>(1.7, 4.2));
    let growth_front = uniforms.time * 0.125 + growth_noise * 0.22 + 0.035;
    let front_coarse = value_noise(point * 9.5 + vec2<f32>(8.3, 2.4));
    let front_fine = value_noise(point * 27.0 + vec2<f32>(3.6, 14.1));
    let growth_distance = growth_front - edge_distance
        + (front_coarse - 0.5) * 0.070
        + (front_fine - 0.5) * 0.026;
    let front_feather = mix(1.5, 3.0, front_fine) / max(uniforms.resolution.y, 1.0);
    var grown = smoothstep(-front_feather, front_feather, growth_distance);
    let frontier = 1.0 - smoothstep(0.0, front_feather * 9.0, abs(growth_distance));
    let pore_noise = front_coarse * 0.42 + front_fine * 0.58;
    let porous_ice = smoothstep(0.36, 0.68, pore_noise);
    grown *= mix(1.0, porous_ice, frontier * 0.62);
    grown = smoothstep(0.08, 0.92, grown);

    let warp = vec2<f32>(
        value_noise(point * 2.6 + vec2<f32>(2.1, 7.4)),
        value_noise(point * 2.9 + vec2<f32>(11.3, 1.8)),
    ) - 0.5;
    let organic_point = point + warp * 0.145;
    let coarse_cells = frost_cell_edge(organic_point, 8.5, 3.7);
    let fine_cells = frost_cell_edge(organic_point + warp * 0.38, 18.0, 12.9);
    let bloom_centers = array<vec2<f32>, 5>(
        vec2<f32>(-0.36 * aspect, -0.28),
        vec2<f32>(0.27 * aspect, -0.34),
        vec2<f32>(-0.08 * aspect, 0.05),
        vec2<f32>(0.38 * aspect, 0.24),
        vec2<f32>(-0.31 * aspect, 0.35),
    );
    var bloom = 0.0;
    for (var bloom_index = 0; bloom_index < 5; bloom_index += 1) {
        bloom = max(
            bloom,
            frost_bloom(point, bloom_centers[bloom_index], f32(bloom_index) + 1.3),
        );
    }
    let crystal = clamp(coarse_cells * 0.46 + fine_cells * 0.22 + bloom * 0.58, 0.0, 1.0);
    let fine_ice = value_noise(point * 24.0 + vec2<f32>(uniforms.time * 0.015, 2.8));
    let frost_density = clamp(0.58 + crystal * 0.32 + fine_ice * 0.12, 0.0, 1.0);

    let melt_centers = array<vec2<f32>, 4>(
        vec2<f32>(0.18, 0.24),
        vec2<f32>(0.73, 0.18),
        vec2<f32>(0.38, 0.72),
        vec2<f32>(0.84, 0.76),
    );
    var melt = 0.0;
    var warm_rim = 0.0;
    for (var index = 0; index < 4; index += 1) {
        var delta = uv - melt_centers[index];
        delta.x *= aspect;
        let seed = f32(index);
        let angle = atan2(delta.y, delta.x);
        let irregularity = 1.0
            + sin(angle * (5.0 + seed) + seed * 1.9) * 0.075
            + (value_noise(delta * 7.0 + seed * 3.4) - 0.5) * 0.16;
        let distance_from_warmth = length(delta) * irregularity;
        let radius = exit * (0.56 + seed * 0.025) - 0.065 - seed * 0.012;
        melt = max(
            melt,
            1.0 - smoothstep(radius - 0.035, radius + 0.045, distance_from_warmth),
        );
        warm_rim = max(
            warm_rim,
            exp(-abs(distance_from_warmth - radius) * 74.0) * step(0.001, exit),
        );
    }

    let ice_white = vec3<f32>(0.918, 0.961, 0.969);
    let rime = vec3<f32>(0.784, 0.882, 0.906);
    let glacier = vec3<f32>(0.553, 0.718, 0.765);
    var color = mix(glacier, rime, 0.48 + frost_density * 0.42);
    color = mix(color, ice_white, crystal * 0.38);
    color += vec3<f32>(1.0, 0.957, 0.843) * warm_rim * 0.18;

    let final_clear = 1.0 - smoothstep(0.84, 1.0, exit);
    let alpha = (0.78 + frost_density * 0.20) * grown * (1.0 - melt) * final_clear;
    return composite(color, alpha);
}

fn crt_shutdown(uv: vec2<f32>) -> vec4<f32> {
    let exit = clamp(uniforms.exit_progress, 0.0, 1.0);
    let vertical_progress = ease_in_out_cubic(clamp(exit / 0.68, 0.0, 1.0));
    let horizontal_progress = ease_in_out_cubic(clamp((exit - 0.64) / 0.30, 0.0, 1.0));
    let half_height = mix(0.52, 0.0022, vertical_progress);
    let half_width = mix(0.52, 0.0030, horizontal_progress);
    let point = uv - 0.5;
    let feather_x = 2.0 / uniforms.resolution.x;
    let feather_y = 2.0 / uniforms.resolution.y;
    let shape_x = 1.0 - smoothstep(half_width - feather_x, half_width + feather_x, abs(point.x));
    let shape_y = 1.0 - smoothstep(half_height - feather_y, half_height + feather_y, abs(point.y));
    let final_blink = 1.0 - smoothstep(0.94, 1.0, exit);
    let alpha = shape_x * shape_y * final_blink;
    if alpha <= 0.001 {
        return vec4<f32>(0.0);
    }

    let pixel = floor(uv * uniforms.resolution * 0.24);
    let static_frame = floor(uniforms.time * 18.0);
    let snow = hash21(pixel + vec2<f32>(static_frame * 13.1, static_frame * 3.7));
    let scanline = 0.5 + 0.5 * sin(uv.y * uniforms.resolution.y * 3.14159265);
    let sync_line = exp(-pow((fract(uv.y * 1.35 - uniforms.time * 0.075) - 0.5) / 0.025, 2.0));
    let vignette = 1.0 - smoothstep(0.20, 0.73, length(point * vec2<f32>(1.0, 1.18)));

    let tube_black = vec3<f32>(0.002, 0.006, 0.004);
    let phosphor_shadow = vec3<f32>(0.078, 0.200, 0.125);
    let phosphor_green = vec3<f32>(0.549, 1.0, 0.675);
    let hot_core = vec3<f32>(0.933, 1.0, 0.945);
    var color = mix(tube_black, phosphor_shadow, 0.16 + vignette * 0.15);
    color += phosphor_shadow * (snow - 0.5) * 0.075;
    color += phosphor_green * scanline * 0.018;
    color += phosphor_green * sync_line * 0.055;

    let collapse_light = smoothstep(0.48, 0.94, vertical_progress);
    let line_core = exp(-abs(point.y) / max(half_height * 0.46, 0.0012));
    color = mix(color, phosphor_green, collapse_light * (0.52 + line_core * 0.30));
    color = mix(color, hot_core, collapse_light * line_core * (0.22 + horizontal_progress * 0.48));
    return composite(color, alpha);
}

fn geological_strata(uv: vec2<f32>) -> vec4<f32> {
    let exit = ease_in_out_cubic(clamp(uniforms.exit_progress, 0.0, 1.0));
    let travel = exit * 0.69;
    let left_source = uv - vec2<f32>(-travel, exit * exit * 0.075);
    let right_source = uv - vec2<f32>(travel, exit * exit * 0.115);
    let left_valid = inside_screen(left_source)
        && left_source.x <= strata_fault(left_source.y);
    let right_valid = inside_screen(right_source)
        && right_source.x > strata_fault(right_source.y);

    var source = left_source;
    var valid = left_valid;
    if right_valid {
        source = right_source;
        valid = true;
    }
    if !valid {
        return vec4<f32>(0.0);
    }

    let broad_warp = sin(source.x * 10.8 + 0.7) * 0.020
        + sin(source.x * 27.0 + 2.4) * 0.008;
    let stone_noise = value_noise(source * vec2<f32>(8.0, 17.0));
    let warped_y = source.y + broad_warp + (stone_noise - 0.5) * 0.035;
    let band_index = u32(clamp(i32(floor(warped_y * 9.0)), 0, 8));
    let strata_palette = array<vec3<f32>, 9>(
        vec3<f32>(0.145, 0.153, 0.151),
        vec3<f32>(0.471, 0.396, 0.288),
        vec3<f32>(0.278, 0.271, 0.253),
        vec3<f32>(0.630, 0.440, 0.240),
        vec3<f32>(0.436, 0.251, 0.184),
        vec3<f32>(0.704, 0.626, 0.466),
        vec3<f32>(0.240, 0.242, 0.229),
        vec3<f32>(0.548, 0.382, 0.241),
        vec3<f32>(0.752, 0.686, 0.557),
    );
    var color = strata_palette[band_index];

    let granular = hash21(floor(source * uniforms.resolution * 0.36));
    color *= 0.82 + stone_noise * 0.22 + (granular - 0.5) * 0.09;
    let lamination = 1.0 - smoothstep(
        0.018,
        0.070,
        abs(fract(warped_y * 36.0 + stone_noise * 0.28) - 0.5),
    );
    color *= 1.0 - lamination * 0.15;

    let pebble_space = source * vec2<f32>(46.0, 29.0);
    let pebble_cell = floor(pebble_space);
    var pebble_local = fract(pebble_space) - 0.5;
    pebble_local.x *= 1.38;
    let pebble = (1.0 - smoothstep(0.20, 0.39, length(pebble_local)))
        * step(0.875, hash21(pebble_cell + 8.0));
    color = mix(color, color * 0.62 + vec3<f32>(0.14, 0.12, 0.09), pebble * 0.72);

    let fault_distance = abs(source.x - strata_fault(source.y));
    let fault_reveal = smoothstep(0.0, 0.16, exit);
    let crack_width = fault_reveal * 0.018;
    let fault_shadow = (1.0 - smoothstep(crack_width, crack_width + 0.030, fault_distance))
        * fault_reveal;
    color *= 1.0 - fault_shadow * 0.48;
    let crack_alpha = smoothstep(
        crack_width,
        crack_width + 2.5 / uniforms.resolution.x,
        fault_distance,
    );
    let alpha = mix(1.0, crack_alpha, fault_reveal);
    return composite(color, alpha);
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let uv = input.position.xy / uniforms.resolution;
    // An exact switch on an integer id. The previous float-threshold chain
    // silently aliased unlisted ids onto whichever neighbour came next.
    switch uniforms.effect_id {
        case EFFECT_CURTAIN: { return curtain(uv); }
        case EFFECT_POND_RIPPLES: { return pond_ripples(uv); }
        case EFFECT_FIRE: { return fire(uv); }
        case EFFECT_BLACKOUT: { return blackout(uv); }
        case EFFECT_KALEIDOSCOPE: { return kaleidoscope(uv); }
        case EFFECT_MOSAIC: { return mosaic(uv); }
        case EFFECT_DOOM_FIRE: { return doom_fire(uv); }
        case EFFECT_PROJECTOR_IRIS: { return projector_iris(uv); }
        case EFFECT_GEOLOGICAL_STRATA: { return geological_strata(uv); }
        case EFFECT_FROSTED_GLASS: { return frosted_glass(uv); }
        case EFFECT_CRT_SHUTDOWN: { return crt_shutdown(uv); }
        // An unknown id draws nothing rather than an arbitrary effect. A
        // transparent overlay is a recoverable bug; the wrong flourish on
        // stage is not.
        default: { return vec4<f32>(0.0); }
    }
}
