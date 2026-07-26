//! Procedural icon art.
//!
//! Every icon Flourish ships is computed from these functions rather than
//! checked in as an image, so there are no binary assets to keep in sync with
//! the palette and nothing to re-export when a size is added.
//!
//! Two icons come out of the same drawing. The menu-bar icon is the bare
//! party-popper mark on transparency, sized for a status bar. The application
//! icon sets that mark on a lit stage inside the rounded square macOS expects.

/// The theatrical palette, shared with the Curtain flourish.
const GOLD: [u8; 3] = [230, 177, 62];
const PALE_GOLD: [u8; 3] = [255, 221, 118];
const OXBLOOD: [u8; 3] = [102, 19, 36];
const HOUSE_BLACK: [u8; 3] = [24, 12, 20];
const TEAL: [u8; 3] = [66, 177, 170];

/// Supersampling rate per axis. The art is all analytic tests with hard edges,
/// so coverage averaging is what makes it look smooth.
const SAMPLES: u32 = 4;

/// Proportions Apple's macOS icon template asks for: the rounded body occupies
/// 824 of 1024 points, with a corner radius of 185.4.
const BODY_EXTENT: f32 = 824.0 / 1024.0;
const CORNER_RADIUS: f32 = 185.4 / 1024.0;
/// Where the popper mark sits inside the canvas.
///
/// The drawing's ink does not fill its own unit square — the cone sits low and
/// left, the star high and right — so the origin is nudged per axis to centre
/// the ink rather than the square. Sized generously: at 16 and 32 points the
/// streamers disappear and only the cone and star carry the icon, so the mark
/// has to be large enough that those two still read.
const MARK_ORIGIN: [f32; 2] = [0.135, 0.155];
const MARK_EXTENT: f32 = 0.73;

/// The menu-bar icon: the mark alone, on transparency.
///
/// macOS renders this as a template image, so only its coverage matters there;
/// the colors are what other platforms' trays show.
#[must_use]
pub fn tray_rgba(size: u32) -> Vec<u8> {
    rasterize(size, party_popper_color)
}

/// The application icon: the mark on a lit stage, in the rounded square macOS
/// draws for every app.
#[must_use]
pub fn app_icon_rgba(size: u32) -> Vec<u8> {
    rasterize(size, app_icon_color)
}

/// Renders `sample` over a `size` square, averaging `SAMPLES²` samples per
/// pixel. Samples that return `None` are transparent, and partial coverage at
/// an edge becomes partial alpha.
fn rasterize(size: u32, sample: impl Fn([f32; 2]) -> Option<[u8; 3]>) -> Vec<u8> {
    #[allow(clippy::cast_precision_loss)]
    let scale = |value: u32, offset: u32| {
        (value as f32 + (offset as f32 + 0.5) / SAMPLES as f32) / size as f32
    };

    let mut rgba = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let mut accumulated = [0_u32; 3];
            let mut covered = 0_u32;
            for sample_y in 0..SAMPLES {
                for sample_x in 0..SAMPLES {
                    let point = [scale(x, sample_x), scale(y, sample_y)];
                    if let Some(color) = sample(point) {
                        accumulated[0] += u32::from(color[0]);
                        accumulated[1] += u32::from(color[1]);
                        accumulated[2] += u32::from(color[2]);
                        covered += 1;
                    }
                }
            }

            if covered == 0 {
                rgba.extend_from_slice(&[0, 0, 0, 0]);
            } else {
                let alpha = u8::try_from(covered * 255 / (SAMPLES * SAMPLES))
                    .expect("supersample coverage always fits in one byte");
                rgba.extend_from_slice(&[
                    channel(accumulated[0], covered),
                    channel(accumulated[1], covered),
                    channel(accumulated[2], covered),
                    alpha,
                ]);
            }
        }
    }
    rgba
}

fn channel(accumulated: u32, covered: u32) -> u8 {
    u8::try_from(accumulated / covered).expect("an averaged channel always fits in one byte")
}

/// One sample of the application icon.
///
/// Returns `None` outside the rounded body, which is what gives the corners
/// their antialiased curve once coverage is averaged.
fn app_icon_color(point: [f32; 2]) -> Option<[u8; 3]> {
    let centered = [point[0] - 0.5, point[1] - 0.5];
    let distance = rounded_box_distance(
        centered,
        [BODY_EXTENT / 2.0, BODY_EXTENT / 2.0],
        CORNER_RADIUS,
    );
    if distance > 0.0 {
        return None;
    }

    let mut color = stage_color(point);

    // A gold hairline just inside the edge, echoing the Curtain's trim.
    let trim = 1.0 - smoothstep(0.0, 0.012, -distance);
    color = mix(color, to_float(GOLD), trim * 0.85);

    // The mark last, so it sits on top of the stage rather than tinted by it.
    let mark = [
        (point[0] - MARK_ORIGIN[0]) / MARK_EXTENT,
        (point[1] - MARK_ORIGIN[1]) / MARK_EXTENT,
    ];
    if (0.0..=1.0).contains(&mark[0])
        && (0.0..=1.0).contains(&mark[1])
        && let Some(ink) = party_popper_color(mark)
    {
        color = to_float(ink);
    }

    Some(to_bytes(color))
}

/// The lit backdrop: house black deepening toward the floor, with a warm glow
/// where the popper bursts.
fn stage_color(point: [f32; 2]) -> [f32; 3] {
    const THEATRE_BLACK: [f32; 3] = [0.055, 0.020, 0.032];
    const VELVET: [f32; 3] = [0.235, 0.035, 0.075];

    let ground = mix(VELVET, THEATRE_BLACK, smoothstep(0.0, 1.0, point[1]));

    // Centered on the star, which is where the popper reads as going off. The
    // glow is what separates the mark from the ground at small sizes, where the
    // finer streamers have already washed out.
    let burst = [
        MARK_ORIGIN[0] + 0.72 * MARK_EXTENT,
        MARK_ORIGIN[1] + 0.23 * MARK_EXTENT,
    ];
    let radial = ((point[0] - burst[0]).powi(2) + (point[1] - burst[1]).powi(2)).sqrt();
    let glow = 1.0 - smoothstep(0.0, 0.68, radial);

    let warm = [0.62, 0.27, 0.07];
    [
        glow.mul_add(warm[0] * 0.72, ground[0]),
        glow.mul_add(warm[1] * 0.72, ground[1]),
        glow.mul_add(warm[2] * 0.72, ground[2]),
    ]
}

/// Signed distance to a rounded box centred on the origin. Negative inside.
fn rounded_box_distance(point: [f32; 2], half_extent: [f32; 2], radius: f32) -> f32 {
    let q = [
        point[0].abs() - half_extent[0] + radius,
        point[1].abs() - half_extent[1] + radius,
    ];
    let outside = (q[0].max(0.0).powi(2) + q[1].max(0.0).powi(2)).sqrt();
    let inside = q[0].max(q[1]).min(0.0);
    outside + inside - radius
}

fn smoothstep(edge0: f32, edge1: f32, value: f32) -> f32 {
    let t = ((value - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mix(from: [f32; 3], to: [f32; 3], amount: f32) -> [f32; 3] {
    [
        (to[0] - from[0]).mul_add(amount, from[0]),
        (to[1] - from[1]).mul_add(amount, from[1]),
        (to[2] - from[2]).mul_add(amount, from[2]),
    ]
}

fn to_float(color: [u8; 3]) -> [f32; 3] {
    [
        f32::from(color[0]) / 255.0,
        f32::from(color[1]) / 255.0,
        f32::from(color[2]) / 255.0,
    ]
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn to_bytes(color: [f32; 3]) -> [u8; 3] {
    [
        (color[0].clamp(0.0, 1.0) * 255.0).round() as u8,
        (color[1].clamp(0.0, 1.0) * 255.0).round() as u8,
        (color[2].clamp(0.0, 1.0) * 255.0).round() as u8,
    ]
}

/// The mark itself: a party popper mid-burst, drawn in normalized coordinates
/// so it rasterizes at any size.
fn party_popper_color(point: [f32; 2]) -> Option<[u8; 3]> {
    let [x, y] = point;
    let star_x = (x - 0.72).abs();
    let star_y = (y - 0.23).abs();
    let four_point_star =
        (star_x / 0.055 + star_y / 0.20 <= 1.0) || (star_x / 0.20 + star_y / 0.055 <= 1.0);
    if four_point_star {
        return Some(PALE_GOLD);
    }

    if distance_to_segment(point, [0.48, 0.25], [0.43, 0.13]) < 0.027
        || distance_to_segment(point, [0.76, 0.49], [0.87, 0.43]) < 0.025
    {
        return Some(TEAL);
    }
    if distance_to_segment(point, [0.56, 0.42], [0.63, 0.34]) < 0.024
        || distance_to_segment(point, [0.88, 0.67], [0.84, 0.55]) < 0.026
    {
        return Some(OXBLOOD);
    }
    if distance_to_segment(point, [0.34, 0.37], [0.31, 0.24]) < 0.024 {
        return Some(GOLD);
    }

    let outer = point_in_triangle(point, [0.12, 0.88], [0.38, 0.40], [0.67, 0.68]);
    if !outer {
        return None;
    }
    let inner = point_in_triangle(point, [0.17, 0.82], [0.40, 0.46], [0.59, 0.66]);
    if !inner {
        return Some(HOUSE_BLACK);
    }

    let stripe = ((x + y) * 12.0).floor();
    Some(if stripe.rem_euclid(3.0) < 1.0 {
        OXBLOOD
    } else {
        GOLD
    })
}

fn point_in_triangle(point: [f32; 2], a: [f32; 2], b: [f32; 2], c: [f32; 2]) -> bool {
    let sign = |p: [f32; 2], q: [f32; 2], r: [f32; 2]| {
        (p[0] - r[0]) * (q[1] - r[1]) - (q[0] - r[0]) * (p[1] - r[1])
    };
    let d1 = sign(point, a, b);
    let d2 = sign(point, b, c);
    let d3 = sign(point, c, a);
    let has_negative = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let has_positive = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(has_negative && has_positive)
}

fn distance_to_segment(point: [f32; 2], start: [f32; 2], end: [f32; 2]) -> f32 {
    let segment = [end[0] - start[0], end[1] - start[1]];
    let to_point = [point[0] - start[0], point[1] - start[1]];
    let length_squared = segment[0] * segment[0] + segment[1] * segment[1];
    let projection =
        ((to_point[0] * segment[0] + to_point[1] * segment[1]) / length_squared).clamp(0.0, 1.0);
    let nearest = [
        start[0] + segment[0] * projection,
        start[1] + segment[1] * projection,
    ];
    ((point[0] - nearest[0]).powi(2) + (point[1] - nearest[1]).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::{app_icon_rgba, tray_rgba};
    use std::collections::HashSet;

    fn pixels(rgba: &[u8]) -> impl Iterator<Item = &[u8]> {
        rgba.chunks_exact(4)
    }

    #[test]
    fn the_tray_icon_is_a_non_empty_multicolor_rgba_mask() {
        let rgba = tray_rgba(32);
        assert_eq!(rgba.len(), 32 * 32 * 4);

        let colors = pixels(&rgba)
            .filter(|pixel| pixel[3] > 200)
            .map(|pixel| [pixel[0], pixel[1], pixel[2]])
            .collect::<HashSet<_>>();
        assert!(colors.len() >= 4);
        assert!(pixels(&rgba).any(|pixel| pixel[3] == 0));
        assert!(pixels(&rgba).any(|pixel| pixel[3] == 255));
    }

    #[test]
    fn the_app_icon_fills_its_rounded_body_and_clears_its_corners() {
        let size = 128;
        let rgba = app_icon_rgba(size);
        assert_eq!(rgba.len(), (size * size * 4) as usize);

        let at = |x: u32, y: u32| {
            let index = ((y * size + x) * 4) as usize;
            &rgba[index..index + 4]
        };

        // macOS draws the icon body inset from the canvas, so the extreme
        // corners must be clear or the icon will look oversized next to others.
        for (x, y) in [(0, 0), (size - 1, 0), (0, size - 1), (size - 1, size - 1)] {
            assert_eq!(at(x, y)[3], 0, "corner ({x}, {y}) should be transparent");
        }
        // ...while the middle is solid.
        assert_eq!(at(size / 2, size / 2)[3], 255);
        assert_eq!(at(size / 2, size / 4)[3], 255);
    }

    #[test]
    fn the_app_icon_renders_the_mark_over_the_stage() {
        // If the inset or the compositing order broke, the icon would come out
        // as a plain rounded rectangle. Look for the popper's gold.
        let rgba = app_icon_rgba(256);
        let bright_gold = pixels(&rgba)
            .filter(|pixel| pixel[3] == 255)
            .filter(|pixel| pixel[0] > 200 && pixel[1] > 150 && pixel[2] < 140)
            .count();

        assert!(
            bright_gold > 200,
            "expected the popper's gold over the stage, found {bright_gold} pixels"
        );
    }

    #[test]
    fn every_icon_size_renders_without_panicking() {
        // The .iconset ladder runs from 16 to 1024; small sizes are where
        // rounding in the supersampler is most likely to misbehave.
        for size in [16, 32, 64, 128, 256, 512, 1024] {
            let rgba = app_icon_rgba(size);
            assert_eq!(rgba.len(), (size * size * 4) as usize);
            assert!(
                pixels(&rgba).any(|pixel| pixel[3] > 0),
                "size {size} rendered nothing"
            );
        }
    }
}
