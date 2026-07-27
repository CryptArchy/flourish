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
const EFFECT_MARQUEE_BULBS: u32 = 12u;
const EFFECT_CONSTELLATION: u32 = 13u;
const EFFECT_SPOTLIGHT: u32 = 14u;
const EFFECT_PAPER_TEAR: u32 = 15u;
const EFFECT_ELEVATOR_DOORS: u32 = 16u;
const EFFECT_CONFETTI: u32 = 17u;

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

// One filament's warm-up and cool-down, in seconds on the effect clock.
//
// Tungsten reaches full heat almost instantly and lets go of it slowly. That
// asymmetry is the entire difference between an incandescent sign and a row of
// LEDs blinking on a timer, so it is modelled rather than eyeballed.
fn filament_pulse(period: f32, phase: f32, cooling: f32) -> f32 {
    let age = fract(uniforms.time / period + phase) * period;
    let warming = smoothstep(0.0, 0.16, age);
    return warming * exp(-max(age - 0.16, 0.0) * cooling);
}

// Where the bulb in `cell` sits inside it. Bulbs set by hand are never on a
// perfect lattice, and the halo pass needs the same answer as the body pass.
fn bulb_offset(cell: vec2<f32>) -> vec2<f32> {
    let jitter = hash21(cell + vec2<f32>(7.3, 1.9));
    return (vec2<f32>(jitter, fract(jitter * 137.0)) - 0.5) * 0.07;
}

// Heat, exit overdrive, and remaining life for the bulb in `cell`, returned
// together because the halo pass needs a neighbour's whole state per sample.
fn bulb_state(cell: vec2<f32>) -> vec3<f32> {
    let timing = hash21(cell + vec2<f32>(2.7, 9.1));
    let slow_phase = fract(timing * 41.0);
    let quick_phase = fract(timing * 613.0);
    // Two cycles of unrelated length per bulb. A single period is enough for
    // the eye to find the loop within a few seconds of watching the sign.
    let slow = filament_pulse(mix(3.6, 6.4, timing), slow_phase, 1.9);
    let quick = filament_pulse(mix(2.2, 3.2, 1.0 - timing), quick_phase, 3.1);
    var heat = max(slow, quick * 0.86);

    // The exit drives every filament past full and then lets it go. Staggering
    // by a hash makes it a sign flaring up, not one synchronized flash.
    let stagger = fract(timing * 97.0) * 0.20;
    let local_exit = clamp(clamp(uniforms.exit_progress, 0.0, 1.0) - stagger, 0.0, 1.0);
    let surge = smoothstep(0.0, 0.34, local_exit);
    let overdrive = smoothstep(0.22, 0.60, local_exit);
    // Short, not a fade: a filament that has run past its limit goes all at
    // once, and each bulb takes its own moment to do it.
    let life = 1.0 - smoothstep(0.60, 0.71, local_exit);
    return vec3<f32>(max(heat, surge), overdrive, life);
}

// The incandescent ramp: a filament glows dull red before it is bright enough
// to read as light at all, and only reaches white once it is running hard.
fn tungsten(heat: f32) -> vec3<f32> {
    let cold_ember = vec3<f32>(0.42, 0.055, 0.010);
    let amber = vec3<f32>(1.0, 0.46, 0.090);
    let warm_white = vec3<f32>(1.0, 0.90, 0.66);
    let color = mix(cold_ember, amber, smoothstep(0.0, 0.42, heat));
    return mix(color, warm_white, smoothstep(0.38, 1.0, heat));
}

fn marquee_bulbs(uv: vec2<f32>) -> vec4<f32> {
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let exit = clamp(uniforms.exit_progress, 0.0, 1.0);
    // Cells are square whatever shape the display is, so the bulbs stay round
    // and the sign reads the same on a 16:10 laptop and a 21:9 projector.
    let rows = 8.0;
    let grid = vec2<f32>(uv.x * aspect, uv.y) * rows;
    let cell = floor(grid);
    let local = fract(grid) - 0.5;
    let feather = 1.5 * rows / max(uniforms.resolution.y, 1.0);

    // Light in the air. Every bulb within a cell contributes, which is what
    // pools warm light on the board between the bulbs instead of leaving each
    // one a bright disc on flat black.
    var halo = vec3<f32>(0.0);
    for (var offset_y = -1; offset_y <= 1; offset_y += 1) {
        for (var offset_x = -1; offset_x <= 1; offset_x += 1) {
            let offset = vec2<f32>(f32(offset_x), f32(offset_y));
            let neighbour = cell + offset;
            let state = bulb_state(neighbour);
            let lit = state.x * state.z;
            let distance_from_bulb = length(local - offset - bulb_offset(neighbour));
            // The bursting bulbs throw their light further than the waiting
            // ones, which is what makes the exit feel like a surge.
            let reach = mix(3.6, 3.0, state.y);
            // Only the nine nearest bulbs are sampled, so the kernel has to
            // reach zero inside that window; an exponential alone leaves a
            // visible square of light around every bulb.
            let window = 1.0 - smoothstep(0.85, 1.45, distance_from_bulb);
            let falloff = exp(-distance_from_bulb * reach) * window;
            halo += tungsten(state.x) * lit * falloff * (0.62 + state.y * 0.9);
        }
    }

    let point = local - bulb_offset(cell);
    let state = bulb_state(cell);
    let heat = state.x;
    let overdrive = state.y;
    let life = state.z;

    // An Edison silhouette: a slightly tall globe seated on a ribbed screw base.
    let globe_point = vec2<f32>(point.x, (point.y + 0.045) * 0.92);
    let globe_radius = length(globe_point);
    let globe = 1.0 - smoothstep(0.255 - feather, 0.255 + feather, globe_radius);
    let base_shape = (1.0 - smoothstep(0.086, 0.098, abs(point.x)))
        * smoothstep(0.190, 0.205, point.y)
        * (1.0 - smoothstep(0.340, 0.356, point.y));
    let ridges = 0.5 + 0.5 * sin(point.y * 190.0);
    let brass = mix(vec3<f32>(0.105, 0.078, 0.040), vec3<f32>(0.295, 0.220, 0.112), ridges);

    // A hairpin coil on two support wires, which is the detail that says
    // "Edison" rather than "round lamp".
    let coil = point.y + 0.020 - 0.040 * sin(point.x * 68.0 + 1.6);
    let coil_core = exp(-pow(coil / 0.014, 2.0))
        * (1.0 - smoothstep(0.108, 0.140, abs(point.x)));
    let stems = exp(-pow((abs(point.x) - 0.062) / 0.012, 2.0))
        * smoothstep(-0.010, 0.030, point.y)
        * (1.0 - smoothstep(0.168, 0.200, point.y));
    let filament = clamp(coil_core + stems * 0.45, 0.0, 1.0) * globe;

    // Cold glass is not black: it catches the sign's own light from whichever
    // neighbours happen to be burning.
    let rim = smoothstep(0.175, 0.255, globe_radius);
    let glass = vec3<f32>(0.052, 0.047, 0.057) * (0.5 + rim * 1.05) + halo * 0.30;
    let envelope = globe * (0.28 + 0.72 * (1.0 - globe_radius / 0.255));
    var emission = tungsten(heat) * heat * envelope * (0.60 + overdrive * 1.1);
    emission += mix(tungsten(heat), vec3<f32>(1.0, 0.97, 0.90), overdrive)
        * filament * heat * (2.2 + overdrive * 3.0);
    emission = emission * life + halo;

    let grain = hash21(floor(uv * uniforms.resolution * 0.5));
    let vignette = 1.0 - smoothstep(0.30, 1.0, distance(uv, vec2<f32>(0.5)) * 1.35);
    let board = vec3<f32>(0.024, 0.019, 0.017) * (0.55 + vignette * 0.9)
        + vec3<f32>(0.006, 0.005, 0.004) * grain;
    var surface = mix(board, brass * (0.30 + heat * 0.85), base_shape * life);
    surface = mix(surface, glass, globe * life);

    // The board goes before the bulbs do, so the sign burns on alone for a
    // moment against the live screen before the last filament lets go.
    let board_alpha = 1.0 - smoothstep(0.44, 0.80, exit);
    let glare = clamp(max(emission.r, max(emission.g, emission.b)), 0.0, 1.0);
    let alpha = max(board_alpha, glare);
    if alpha <= 0.001 {
        return vec4<f32>(0.0);
    }
    // composite() premultiplies, so the light is divided back out here. That
    // lets the board fade to nothing underneath without dimming the flare.
    return composite((surface * board_alpha + emission) / alpha, alpha);
}

// Distance from `point` to the segment, and how far along it the nearest point
// lies. Constellation needs both: the lines retract along themselves and the
// meteor trails taper from head to tail.
fn segment_probe(point: vec2<f32>, start: vec2<f32>, end: vec2<f32>) -> vec2<f32> {
    let span = end - start;
    let travel = clamp(dot(point - start, span) / max(dot(span, span), 1e-6), 0.0, 1.0);
    return vec2<f32>(length(point - start - span * travel), travel);
}

// Where the star belonging to `cell` sits, in the same units as the caller's
// point. Empty cells report themselves through `presence`.
fn star_position(cell: vec2<f32>, size: f32) -> vec2<f32> {
    let place = hash21(cell + vec2<f32>(4.1, 12.7));
    return (cell + vec2<f32>(place, fract(place * 137.0)) * 0.86 + 0.07) * size;
}

fn constellation(uv: vec2<f32>) -> vec4<f32> {
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let exit = clamp(uniforms.exit_progress, 0.0, 1.0);
    let point = vec2<f32>(uv.x * aspect, uv.y);
    let rows = 11.0;
    let size = 1.0 / rows;

    // Every meteor in a shower runs from one radiant, and reading that shared
    // origin is most of why a real shower looks like a shower.
    let radiant = vec2<f32>(aspect * mix(0.18, 0.82, hash21(vec2<f32>(3.0, 7.0))), -0.62);
    let lines_gone = smoothstep(0.0, 0.24, exit);
    let launched = smoothstep(0.16, 1.0, exit);
    // The exit flings the whole field away from the radiant. Writing that
    // flight as a scale about that one point is what makes it invertible: a
    // pixel can ask which star is crossing it, instead of each star having to
    // find pixels far outside the cell it was born in.
    let flight = 1.0 + pow(launched, 2.2) * 8.0;
    // The fraction of the flight a streak covers — the shutter, in effect, so
    // the trails lengthen as the field accelerates.
    let smear = 0.12 * smoothstep(0.0, 0.30, launched);
    // Sample the rest frame under the middle of the streak rather than under
    // its head, so a trail's whole length stays inside the nine cells read.
    let rest_point = radiant + (point - radiant) / (flight * (1.0 - smear * 0.5));
    let cell = floor(rest_point / size);
    let moving = step(0.001, smear);
    // Nothing outruns the sky it came from.
    let spent = 1.0 - smoothstep(0.62, 0.94, launched);

    var light = vec3<f32>(0.0);
    for (var offset_y = -1; offset_y <= 1; offset_y += 1) {
        for (var offset_x = -1; offset_x <= 1; offset_x += 1) {
            let home = cell + vec2<f32>(f32(offset_x), f32(offset_y));
            let character = hash21(home + vec2<f32>(21.3, 6.8));
            // Not every cell holds a star; a fully occupied lattice reads as
            // wallpaper rather than as a sky.
            let presence = step(0.26, character);
            // Most stars are faint and a few carry the field, which is roughly
            // how apparent magnitude is distributed overhead.
            let magnitude = pow(fract(character * 37.0), 2.6);
            let twinkle = 0.78
                + 0.22 * sin(uniforms.time * mix(0.7, 2.3, fract(character * 811.0))
                    + fract(character * 313.0) * 6.2831853);
            let brightness = presence * (0.16 + magnitude * 0.84) * twinkle;
            let tint = mix(
                vec3<f32>(0.72, 0.82, 1.0),
                vec3<f32>(1.0, 0.86, 0.68),
                fract(character * 1097.0),
            );
            let rest = star_position(home, size);

            // Radial flight, so the stars overhead barely stir while the ones
            // out at the edges tear past — the geometry does the work that a
            // per-star speed would otherwise have to fake.
            let ray = rest - radiant;
            let head = radiant + ray * flight;
            let tail = radiant + ray * flight * (1.0 - smear);
            let streak = segment_probe(point, tail, head);

            let core = exp(-streak.x * mix(220.0, 90.0, magnitude));
            let bloom = exp(-streak.x * 34.0) * 0.16;
            // Four-point diffraction, the way a bright star reads through a
            // lens. Only the bright ones earn it.
            let delta = point - head;
            let spikes = (exp(-abs(delta.x) * 520.0) + exp(-abs(delta.y) * 520.0))
                * exp(-length(delta) * 90.0)
                * smoothstep(0.45, 1.0, magnitude)
                * 0.5;
            // The trail is brightest at the head and dies out behind it.
            let taper = mix(1.0, 0.06 + 0.94 * streak.y * streak.y, moving);
            light += tint * brightness * (core + bloom + spikes) * taper * spent;

            // A line to one neighbour, drawn only where both ends exist. Chains
            // of these read as asterisms without anyone naming them.
            let partner = home + select(
                vec2<f32>(1.0, 0.0),
                vec2<f32>(f32(offset_x >= 0) * 2.0 - 1.0, 1.0),
                fract(character * 149.0) > 0.5,
            );
            let linked = step(0.72, fract(character * 271.0))
                * step(0.26, hash21(partner + vec2<f32>(21.3, 6.8)))
                * presence
                * (1.0 - lines_gone)
                * (1.0 - moving);
            let line = segment_probe(point, rest, star_position(partner, size));
            let drawn = exp(-line.x * 620.0) * step(line.y, 1.0 - lines_gone * 0.9);
            light += vec3<f32>(0.42, 0.52, 0.78) * drawn * linked * 0.32;
        }
    }

    // A dark sky with one dusty band across it, so the field has somewhere to
    // be sparse without looking empty.
    let band_axis = point.x * 0.42 + point.y * 0.91;
    let band = exp(-pow((band_axis - 0.86) / 0.34, 2.0));
    let dust = value_noise(point * vec2<f32>(5.0, 7.0) + vec2<f32>(1.7, 0.4));
    let horizon = smoothstep(0.0, 1.4, uv.y);
    var sky = mix(vec3<f32>(0.011, 0.013, 0.032), vec3<f32>(0.020, 0.017, 0.044), horizon);
    sky += vec3<f32>(0.036, 0.038, 0.058) * band * (0.30 + dust * 0.70);

    // The night itself is swept off the screen behind the shower, spreading
    // out from the radiant the meteors came from.
    let far = max(
        max(distance(radiant, vec2<f32>(0.0, 0.0)), distance(radiant, vec2<f32>(aspect, 0.0))),
        max(distance(radiant, vec2<f32>(0.0, 1.0)), distance(radiant, vec2<f32>(aspect, 1.0))),
    );
    let front = ease_in_out_cubic(smoothstep(0.46, 1.0, exit)) * (far + 0.30);
    let sky_alpha = smoothstep(front - 0.30, front, distance(point, radiant));

    let glare = clamp(max(light.r, max(light.g, light.b)), 0.0, 1.0);
    let alpha = max(sky_alpha, glare * (1.0 - smoothstep(0.90, 1.0, exit)));
    if alpha <= 0.001 {
        return vec4<f32>(0.0);
    }
    // Premultiplied downstream, so dividing here lets the last meteors keep
    // burning across the screen the wipe has already given back.
    return composite((sky * sky_alpha + light) / alpha, alpha);
}

fn spotlight(uv: vec2<f32>) -> vec4<f32> {
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let exit = clamp(uniforms.exit_progress, 0.0, 1.0);
    let point = vec2<f32>(uv.x * aspect, uv.y);

    // A slow search, seeded so the light is not found in the same place twice.
    // The drift keeps running through the exit: over a 1.6 second dismissal it
    // moves the pool by a few hundredths of a screen, which is why no machinery
    // for settling it exists here.
    let drift = seed_phase();
    let center = vec2<f32>(
        aspect * (0.5 + 0.19 * sin(uniforms.time * 0.21 + drift)),
        0.55 + 0.12 * sin(uniforms.time * 0.147 + drift * 1.7),
    );
    // Squashed vertically: a beam meeting the stage at an angle throws an
    // ellipse, never a circle.
    let radial = length((point - center) * vec2<f32>(1.0, 1.18));

    // The flood has to reach the farthest corner from wherever the light
    // happens to be standing, which is the whole reason this does not read as
    // Projector Iris.
    let corner_reach = max(
        max(distance(center, vec2<f32>(0.0, 0.0)), distance(center, vec2<f32>(aspect, 0.0))),
        max(distance(center, vec2<f32>(0.0, 1.0)), distance(center, vec2<f32>(aspect, 1.0))),
    );
    let full_reach = corner_reach * 1.10;
    let radius = mix(0.185, full_reach, ease_in_out_cubic(exit));
    // The screen returns *behind* the light rather than inside it: the reveal
    // trails the pool's edge by a gap that closes as the wave leaves the
    // screen. What crosses the display is a bright ring, not a hole opening in
    // a bright disc — which is what this looked like when the reveal was
    // concentric with the pool instead of chasing it.
    // The gap closes early, so the frames where the opening is still a small
    // shape surrounded by light pass quickly; that is the eclipse-looking part
    // of a centre-out reveal and there is no geometry that avoids it entirely.
    let trail = 0.42 * (1.0 - smoothstep(0.20, 0.75, exit));
    let reveal_radius = max(radius - trail, 0.0);
    // A wide, soft trailing edge with an irregular rim, so the dark is eaten
    // away rather than punched out. A clean ellipse reads as a hole.
    // Harmonics of the bearing rather than sampled noise: a value-noise lattice
    // read around a circle leaves visible facets on a boundary this large.
    let bearing = atan2(point.y - center.y, point.x - center.x);
    let rim = sin(bearing * 3.0 + seed_phase()) * 0.55
        + sin(bearing * 5.0 - seed_phase() * 1.7) * 0.30
        + sin(bearing * 8.0 + 1.1) * 0.15;
    let eaten = radial * (1.0 + rim * 0.065);
    let feather = mix(0.12, 0.42, clamp(reveal_radius / full_reach, 0.0, 1.0));
    let opened = select(
        0.0,
        1.0 - smoothstep(max(reveal_radius - feather, 0.0), reveal_radius, eaten),
        reveal_radius > 0.001,
    );

    // A few percent of arc flicker. A perfectly steady stage light reads as a
    // rendered circle rather than as a lamp.
    let arc = 0.962 + 0.038 * sin(uniforms.time * 11.3 + sin(uniforms.time * 3.1) * 1.7);
    let pool = 1.0 - smoothstep(radius * 0.52, radius, radial);
    let core = 1.0 - smoothstep(0.0, radius * 0.64, radial);
    let spill = exp(-max(radial - radius, 0.0) * 8.5);

    // The lamp hangs in a fixed position above the proscenium, so the beam
    // swings as the pool searches — the way a follow-spot actually behaves.
    let lamp = vec2<f32>(aspect * 0.34, -0.52);
    let lamp_to_pool = center - lamp;
    let span = max(length(lamp_to_pool), 0.001);
    let axis = lamp_to_pool / span;
    let along = dot(point - lamp, axis);
    let across = length(point - lamp - axis * along);
    let reach = clamp(along / span, 0.0, 1.0);
    let cone_width = mix(0.016, radius * 0.94, reach);
    var beam = (1.0 - smoothstep(cone_width * 0.30, cone_width, across))
        * step(0.0, along)
        * (1.0 - smoothstep(span * 0.97, span * 1.15, along));
    // A shaft is only visible because there is dust in it.
    let haze_drift = vec2<f32>(uniforms.time * 0.045, uniforms.time * -0.09);
    let haze = value_noise(point * vec2<f32>(6.5, 4.5) + haze_drift);
    let motes = value_noise(point * 17.0 - haze_drift * 2.3);
    beam *= (0.45 + haze * 0.60 + motes * 0.20) * mix(1.0, 0.42, reach);

    let lamp_core = vec3<f32>(1.0, 0.953, 0.863);
    let warm_throw = vec3<f32>(1.0, 0.843, 0.604);
    let grain = hash21(floor(uv * uniforms.resolution * 0.5));
    let vignette = 1.0 - smoothstep(0.25, 1.15, distance(uv, vec2<f32>(0.5)) * 1.25);
    let stage = vec3<f32>(0.020, 0.020, 0.027) * (0.55 + vignette * 0.85)
        + vec3<f32>(0.005, 0.005, 0.006) * grain;

    var light = mix(warm_throw, lamp_core, core * 0.70) * (pool * 0.92 + core * 0.30) * arc;
    light += warm_throw * beam * 0.20 * arc;
    light += warm_throw * spill * (1.0 - pool) * 0.11;

    // The light lives on the stage, so it leaves with it. No premultiplied
    // rescue is needed here: a pixel the reveal has already opened has nothing
    // left to be lit.
    // The last of it clears unconditionally, so no corner outruns the wave.
    let alpha = (1.0 - opened) * (1.0 - smoothstep(0.88, 1.0, exit));
    if alpha <= 0.001 {
        return vec4<f32>(0.0);
    }
    return composite(stage + light, alpha);
}

// The tear is split into two scales, because they do different jobs.
//
// `tear_axis` is where the sheet comes apart at large scale, and it also
// carries the curl: a cylinder needs a reasonably straight axis, and driving it
// with the full ragged edge makes the roll wobble sideways by as much as it is
// wide, which reads as a lumpy ribbon rather than as a rolled edge.
fn tear_axis(y: f32) -> f32 {
    return 0.5 + (value_noise(vec2<f32>(y * 3.1, 11.7)) - 0.5) * 0.090;
}

// `tear_rag` is the kinks and fibre-scale roughness on top of it. It is applied
// as a difference in how much paper each row has, so it ends up as raggedness
// in how far that row has wrapped — the torn edge stays ragged while the roll
// it sits on stays straight.
fn tear_rag(y: f32) -> f32 {
    let kink = (value_noise(vec2<f32>(y * 13.0, 4.2)) - 0.5) * 0.034;
    let fibre = (value_noise(vec2<f32>(y * 61.0, 27.3)) - 0.5) * 0.012;
    return kink + fibre;
}

// Sampled in sheet coordinates, never screen coordinates: the grain has to
// travel with the paper and compress into the roll, or the sheet reads as a
// gradient sliding under a static texture.
fn paper_surface(sheet: vec2<f32>) -> vec3<f32> {
    let stock = vec3<f32>(0.941, 0.918, 0.851);
    let mottle = value_noise(sheet * vec2<f32>(3.4, 3.0) + vec2<f32>(2.7, 8.1));
    let cloud = value_noise(sheet * vec2<f32>(9.0, 8.2) + vec2<f32>(5.3, 1.4));
    // Fibres lying in the pulp, in both directions. One direction alone reads
    // as brushed metal rather than as paper.
    // Kept mildly anisotropic and sparse. Strongly stretched noise lays down a
    // lattice of rectangles, and two of them crossing reads as woven linen.
    let along = value_noise(sheet * vec2<f32>(150.0, 26.0) + vec2<f32>(0.7, 3.9));
    let across = value_noise(sheet * vec2<f32>(24.0, 190.0) + vec2<f32>(5.1, 0.3));
    let tooth = hash21(floor(sheet * 900.0));
    var color = stock * (0.930 + mottle * 0.080 + cloud * 0.045);
    color -= vec3<f32>(0.022, 0.023, 0.026) * smoothstep(0.79, 1.0, along);
    color -= vec3<f32>(0.016, 0.017, 0.020) * smoothstep(0.82, 1.0, across);
    color += vec3<f32>(0.016) * (tooth - 0.5);
    return color;
}

fn paper_tear(uv: vec2<f32>) -> vec4<f32> {
    let exit = clamp(uniforms.exit_progress, 0.0, 1.0);
    let pi = 3.14159265;

    // The crack runs top to bottom over the first third. Rows it has not
    // reached are still joined, which is what makes this a tear rather than
    // two panels sliding apart.
    // Overshoots 1.0 by more than the smoothstep below is wide, or the last
    // rows never reach full separation and the sheet keeps a joined band along
    // the bottom edge for the whole exit.
    let crack_front = ease_in_out_cubic(clamp(exit / 0.34, 0.0, 1.0)) * 1.25;
    let row_open = smoothstep(0.0, 0.11, crack_front - uv.y);
    // Rows that have been open longer are further apart, so the tear is a V
    // rather than a parallel gap — the halves swing from the bottom the way
    // torn paper does. The wedge is constant in time and sized so even the
    // slowest row clears the screen: equalizing it late made the bottom catch
    // up as a separate second motion, which reads as two sheets tearing at
    // different times rather than as one sheet coming apart.
    let opened_for = clamp((crack_front - uv.y) / 0.62, 0.0, 1.0);
    let wedge = mix(0.76, 1.0, opened_for);
    // Sized so the slowest row's roll leaves the screen just before the exit
    // ends. It is smaller than it looks: rolling up consumes paper, so the
    // curl below pulls each tangent back by its own arc length on top of this.
    let pull = 0.55 * pow(clamp((exit - 0.05) / 0.95, 0.0, 1.0), 1.35) * row_open * wedge;
    // The halves sag as they go. Without it they read as two rectangles.
    let sag = pull * pull * 0.10;

    let sheet_y = uv.y - sag;
    let axis = tear_axis(sheet_y);
    let rag = tear_rag(sheet_y);
    let split = axis + rag;

    // The curl. The paper leaves the flat plane at a tangent point, wraps a
    // cylinder of this radius through `wrap` radians, and ends at the torn
    // edge. Past a quarter turn the wrapped paper comes back over the sheet and
    // what the viewer sees is its *back face*, lying on top of the paper it
    // came from — that flap is what reads as a curl.
    //
    // A cylinder that rolls the other way, away from the viewer, shows only its
    // front: a flat bright band beside a flat bright sheet, divided by a seam.
    // That is two stacked sheets, not a curl, and no amount of shading fixes it.
    let curl_radius = 0.058;
    // Front-loaded, on a curve that leaves its slow part for the end. Below a
    // half turn the flap is only a crescent a few pixels wide at the outer
    // edge; the curl does not read until the paper comes back over the sheet,
    // so any time spent under that is time spent looking like nothing.
    //
    // No separation gate is needed: a tangent recedes by its own arc length, so
    // the two rolls hold each other apart faster than they widen.
    let wrap = 4.4 * pow(clamp((exit - 0.08) / 0.44, 0.0, 1.0), 0.7) * row_open;
    // Where the row's raggedness lives. On flat paper it is simply where the
    // edge is. Once the paper is well curled it has to move into how far that
    // row has *wrapped*, or it drags the roll's axis sideways with it. Anything
    // in between blends, because forcing rag into the wrap while the sheet is
    // still flat invents a curl on a sheet that has not curled — which draws a
    // ragged ghost line down the held sheet.
    let curl_share = smoothstep(0.0, 1.5707963, wrap);
    let side = select(-1.0, 1.0, uv.x >= split);
    let tangent = axis + side * (pull + curl_radius * wrap) + rag * (1.0 - curl_share);
    let q = -side * (uv.x - tangent);
    let wrap_row = wrap - side * rag * curl_share / curl_radius;

    // Three surfaces can be over this pixel: the flap (the cylinder's outer
    // half, the paper's back), the inside of the curl (its front face, looking
    // into the opening), and the flat sheet. The flap is nearest the viewer
    // wherever it reaches, so it is tested first.
    let flap_inner = curl_radius * sin(wrap_row);
    var region = 3u;
    var phi = 0.0;
    if wrap_row <= 0.0 {
        // This row carries less paper than the roll's line, so it simply ends
        // short of the tangent rather than curling at all.
        if q <= curl_radius * wrap_row {
            region = 0u;
        }
    } else if q > curl_radius {
        region = 3u;
    } else if wrap_row >= 1.5707963 && q >= flap_inner {
        region = 1u;
        phi = pi - asin(clamp(q / curl_radius, -1.0, 1.0));
    } else if q >= 0.0 {
        phi = asin(clamp(q / curl_radius, -1.0, 1.0));
        if phi <= wrap_row {
            region = 2u;
        } else {
            region = 3u;
        }
    } else {
        region = 0u;
    }

    if region != 3u {
        // Flat paper is rigid and simply translated, whatever its edge is
        // doing. Curled paper is addressed by arc length around the roll, and
        // lands in the same sheet coordinate, so the grain runs continuously
        // off the sheet and around the curl.
        var sheet_x = uv.x - side * pull;
        if region != 0u {
            sheet_x = split + side * curl_radius * (wrap_row - phi);
        }
        // The sheet is larger than the screen on every side, so neither the sag
        // nor the curl's offset can pull an outer edge into frame.
        if sheet_x >= -0.35 && sheet_x <= 1.35 {
            var color = paper_surface(vec2<f32>(sheet_x, sheet_y));
            // Lighting belongs to the room, not to the sheet, so it is sampled
            // in screen space: the paper travels through it.
            let sweep = value_noise(vec2<f32>(uv.x * 1.7 + uniforms.time * 0.035, uv.y * 1.5));
            let vignette = 1.0
                - 0.075 * smoothstep(0.28, 1.0, distance(uv, vec2<f32>(0.5)) * 1.3);
            color *= (0.955 + sweep * 0.090) * vignette;

            // One light, upper-left and in front of the sheet, in the plane
            // across the curl. Flat paper faces the viewer squarely.
            let light = vec2<f32>(-0.50, 0.87);
            var lit = light.y;
            if region == 1u {
                // The flap: outer surface of the cylinder, so its normal sweeps
                // a full half turn and the shading with it. `q` runs the other
                // way on the right half, so the normal's world x carries -side;
                // without it both rolls light identically and the halves read
                // as two separately lit sheets again.
                lit = max(dot(vec2<f32>(-side * sin(phi), -cos(phi)), light), 0.0);
            } else if region == 2u {
                // Inside the curl, and increasingly buried by the paper above.
                lit = max(dot(vec2<f32>(side * sin(phi), cos(phi)), light), 0.0)
                    * mix(1.0, 0.28, clamp(phi / 1.5707963, 0.0, 1.0));
            } else if wrap_row >= 1.5707963 {
                // Flat, and lying under the flap's edge.
                lit *= 1.0 - 0.40 * exp(-max(flap_inner - q, 0.0) * 55.0);
            }
            color *= 0.20 + 0.92 * lit;
            // The back of a sheet is the side that was never printed on.
            if region == 1u {
                color *= vec3<f32>(0.955, 0.935, 0.885);
            }
            return composite(color, 1.0);
        }
    }

    // No paper here, but the sheet has height above whatever is underneath it.
    // This shadow is the only cue that says so.
    // The gap runs between the two rolls' outer silhouettes, not between the
    // torn edges: the flaps stand proud of their tangents by a radius.
    let left_edge = axis - (pull + curl_radius * wrap) + rag * (1.0 - curl_share) + curl_radius;
    let right_edge = axis + (pull + curl_radius * wrap) + rag * (1.0 - curl_share) - curl_radius;
    let in_gap = step(left_edge, uv.x) * step(uv.x, right_edge) * row_open;
    // Thrown by the left half, because that is where the light is. Both edges
    // additionally get a tight contact darkening, which is occlusion rather
    // than a cast shadow and so does have a side on both.
    let thrown = exp(-max(uv.x - left_edge, 0.0) * 20.0);
    let contact = max(
        exp(-max(uv.x - left_edge, 0.0) * 70.0),
        exp(-max(right_edge - uv.x, 0.0) * 70.0),
    );
    let alpha = clamp((thrown * 0.34 + contact * 0.16) * in_gap, 0.0, 1.0);
    if alpha <= 0.001 {
        return vec4<f32>(0.0);
    }
    return composite(vec3<f32>(0.035, 0.032, 0.028), alpha);
}

// Brushed steel. Sampled in panel coordinates by its caller, so the grain
// travels with the door it belongs to; sampling it in screen space shears the
// metal as the door slides.
fn brushed_steel(panel: vec2<f32>) -> f32 {
    let fine = value_noise(vec2<f32>(panel.x * 640.0, panel.y * 2.2));
    let coarse = value_noise(vec2<f32>(panel.x * 98.0, panel.y * 1.3));
    let flecks = hash21(floor(vec2<f32>(panel.x * 900.0, panel.y * 44.0)));
    return clamp(
        0.5 + (fine - 0.5) * 0.58 + (coarse - 0.5) * 0.34 + (flecks - 0.5) * 0.10,
        0.0,
        1.0,
    );
}

fn elevator_doors(uv: vec2<f32>) -> vec4<f32> {
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let exit = clamp(uniforms.exit_progress, 0.0, 1.0);

    // Machinery: a moment under load before anything moves, then driven, with
    // no easing at the end. A door that eases out of its travel looks placed
    // rather than driven, which is the difference between this and a curtain.
    let travel = 0.56 * pow(clamp((exit - 0.085) / 0.915, 0.0, 1.0), 1.45);
    let strain = 1.0 - smoothstep(0.0, 0.085, exit);

    let left_edge = 0.5 - travel;
    let right_edge = 0.5 + travel;
    let on_left = uv.x <= left_edge;
    let on_right = uv.x >= right_edge;

    if on_left || on_right {
        let side = select(-1.0, 1.0, on_right);
        // The panel coordinate follows the door; the world coordinate does not.
        let panel = vec2<f32>((uv.x - side * travel) * aspect, uv.y);
        let world = vec2<f32>(uv.x * aspect, uv.y);
        let grain = brushed_steel(panel);

        // A flat mirror sliding within its own plane leaves the room it is
        // reflecting exactly where it was — the reflection does not travel with
        // the door, the door travels under it. Two coordinate systems, no extra
        // samples, and most of the difference between this and two grey panels.
        let drift = uniforms.time * 0.05 + seed_phase();
        let bands = 0.5
            + 0.26 * sin((world.x * 0.9 - world.y * 1.5) * 2.4 + drift)
            + 0.20 * sin((world.x * 1.7 + world.y * 0.8) * 1.5 - drift * 0.7);
        let fitting = world.x * 0.75 + world.y * 0.55 - 0.62 - sin(drift * 0.6) * 0.22;
        let sweep = exp(-pow(fitting / 0.17, 2.0));
        // Brushing smears a reflection along the grain instead of mirroring it,
        // so the bands are modulated by the striations rather than laid over.
        let smeared = mix(bands, bands * (0.55 + grain * 0.80), 0.68);

        let steel = vec3<f32>(0.541, 0.565, 0.600);
        let bright = vec3<f32>(0.776, 0.800, 0.827);
        let shadowed = vec3<f32>(0.306, 0.329, 0.361);
        var color = mix(shadowed, steel, clamp(0.34 + smeared * 0.92, 0.0, 1.0));
        color = mix(color, bright, clamp(sweep, 0.0, 1.0) * 0.55);
        // The room above the doors is brighter than the floor in front of them.
        color *= 0.86 + 0.26 * (1.0 - uv.y);

        // The leading edge is a machined lip with a chamfer that catches light.
        // At rest the two lips together are the seam.
        let inner = select(left_edge - uv.x, uv.x - right_edge, on_right);
        let chamfer = 1.0 - smoothstep(0.0, 0.011, inner);
        color = mix(color, bright, chamfer * 0.42);
        let lip = 1.0 - smoothstep(0.0, 0.0024, inner);
        color = mix(color, vec3<f32>(0.078, 0.090, 0.102), lip * 0.88);
        // Taking up the slack presses the seam shut before the drive starts.
        color *= 1.0 - 0.20 * strain * (1.0 - smoothstep(0.0, 0.016, inner));
        return composite(color, 1.0);
    }

    // Between the doors: no metal, but they are thick panels standing in front
    // of whatever they have opened onto.
    let shadow = max(
        exp(-max(uv.x - left_edge, 0.0) * 42.0),
        exp(-max(right_edge - uv.x, 0.0) * 42.0),
    );
    let alpha = clamp(shadow * 0.46, 0.0, 1.0);
    if alpha <= 0.001 {
        return vec4<f32>(0.0);
    }
    return composite(vec3<f32>(0.031, 0.035, 0.041), alpha);
}

// A festive palette, and the duller reverse a piece shows once it has tumbled
// past edge-on. Foil is printed on one side only.
fn confetti_face(pick: f32, front: f32) -> vec3<f32> {
    var colour = vec3<f32>(0.969, 0.957, 0.937);
    if pick < 0.17 {
        colour = vec3<f32>(0.949, 0.757, 0.306);
    } else if pick < 0.34 {
        colour = vec3<f32>(0.898, 0.282, 0.435);
    } else if pick < 0.51 {
        colour = vec3<f32>(0.247, 0.757, 0.788);
    } else if pick < 0.68 {
        colour = vec3<f32>(0.549, 0.776, 0.247);
    } else if pick < 0.85 {
        colour = vec3<f32>(0.949, 0.420, 0.357);
    }
    return mix(colour * vec3<f32>(0.58, 0.60, 0.64), colour, front);
}

// One depth of the shower, returned premultiplied.
//
// Falling is a translation, and a translation is invertible: the field lives in
// a flow space that slides down the screen, so a piece keeps its cell while its
// position moves, and a pixel can back-map into that space and ask which pieces
// cover it.
fn confetti_layer(
    uv: vec2<f32>,
    aspect: f32,
    rows: f32,
    speed: f32,
    dim: f32,
    salt: f32,
    exit: f32,
) -> vec4<f32> {
    // Thrown rather than dropped: the exponential is the launch settling into a
    // drift, and the exit adds an acceleration on top of it.
    let launch = 0.22 * (1.0 - exp(-uniforms.time / 0.30));
    let fall = uniforms.time * speed + launch + exit * exit * 0.85;
    let flow = vec2<f32>(uv.x * aspect, uv.y - fall) * rows;
    let cell = floor(flow);

    var accumulated = vec3<f32>(0.0);
    var coverage = 0.0;
    for (var offset_y = -1; offset_y <= 1; offset_y += 1) {
        for (var offset_x = -1; offset_x <= 1; offset_x += 1) {
            let home = cell + vec2<f32>(f32(offset_x), f32(offset_y));
            let character = hash21(home + vec2<f32>(salt, salt * 1.7));
            // Not every cell carries a piece; a full lattice reads as a grid.
            if character < 0.34 {
                continue;
            }

            let jitter = vec2<f32>(fract(character * 41.0), fract(character * 613.0)) - 0.5;
            // Sway stays within half a cell, so the nine cells read here always
            // contain whichever piece covers this pixel.
            let sway = sin(
                uniforms.time * mix(0.9, 2.1, fract(character * 97.0))
                    + fract(character * 311.0) * 6.2831853,
            ) * 0.30;
            let centre = home + 0.5 + vec2<f32>(jitter.x * 0.5 + sway, jitter.y * 0.5);

            // Tumble. A rectangle turning about its long axis presents a width
            // of |cos|, so it thins to a line and flashes back — one cosine,
            // and the whole difference between confetti and falling squares.
            let spin = uniforms.time * mix(2.4, 5.2, fract(character * 173.0))
                + fract(character * 887.0) * 6.2831853;
            let turn = cos(spin);
            let tilt = fract(character * 53.0) * 6.2831853;
            let facing = vec2<f32>(cos(tilt), sin(tilt));
            let delta = flow - centre;
            let along = dot(delta, facing);
            let across = dot(delta, vec2<f32>(-facing.y, facing.x));

            let half_long = mix(0.20, 0.30, fract(character * 29.0));
            let half_short = half_long * 0.55 * abs(turn);
            let edge = 0.035;
            let shape = (1.0 - smoothstep(half_long - edge, half_long + edge, abs(along)))
                * (1.0 - smoothstep(half_short - edge * 0.6, half_short + edge * 0.6, abs(across)));
            if shape <= 0.001 {
                continue;
            }

            var colour = confetti_face(fract(character * 1097.0), step(0.0, turn));
            // Foil catches the light as it turns through edge-on.
            colour += vec3<f32>(0.55, 0.55, 0.50) * pow(1.0 - abs(turn), 6.0) * 0.8;
            colour *= dim;

            // Painted in whatever order the loop runs; pieces of one layer
            // rarely overlap, and the near layer is composited over this one.
            let piece = shape * (1.0 - coverage);
            accumulated += colour * piece;
            coverage += piece;
        }
    }
    return vec4<f32>(accumulated, coverage);
}

fn confetti(uv: vec2<f32>) -> vec4<f32> {
    let aspect = uniforms.resolution.x / uniforms.resolution.y;
    let exit = clamp(uniforms.exit_progress, 0.0, 1.0);

    let far = confetti_layer(uv, aspect, 13.0, 0.30, 0.62, 7.3, exit);
    let near = confetti_layer(uv, aspect, 8.5, 0.44, 1.0, 31.7, exit);
    let stacked = near.a + far.a * (1.0 - near.a);
    if stacked <= 0.001 {
        return vec4<f32>(0.0);
    }
    let colour = (near.rgb + far.rgb * (1.0 - near.a)) / stacked;

    // The exit stops the source rather than fading the field: a line descends
    // the screen, the shower has run out above it, and what is below keeps
    // falling. By the end the line has passed the bottom edge, so the screen is
    // given back without needing a final clear.
    let ran_out = smoothstep(exit * 1.25 - 0.14, exit * 1.25, uv.y);
    return composite(colour, stacked * ran_out);
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
        case EFFECT_MARQUEE_BULBS: { return marquee_bulbs(uv); }
        case EFFECT_CONSTELLATION: { return constellation(uv); }
        case EFFECT_SPOTLIGHT: { return spotlight(uv); }
        case EFFECT_PAPER_TEAR: { return paper_tear(uv); }
        case EFFECT_ELEVATOR_DOORS: { return elevator_doors(uv); }
        case EFFECT_CONFETTI: { return confetti(uv); }
        // An unknown id draws nothing rather than an arbitrary effect. A
        // transparent overlay is a recoverable bug; the wrong flourish on
        // stage is not.
        default: { return vec4<f32>(0.0); }
    }
}
