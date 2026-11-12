//! Fondos de tarjeta dibujados por la app: pixel art de 32×40 generado con código,
//! que la interfaz escala sin suavizar.

use image::{Rgba, RgbaImage};

pub const COUNT: u8 = 6;
pub const W: u32 = 32;
pub const H: u32 = 40;

/// Generador pseudoaleatorio mínimo: el mismo fondo sale siempre igual.
struct Lcg(u32);

impl Lcg {
    fn next(&mut self) -> u32 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        self.0 >> 16
    }
    fn below(&mut self, n: u32) -> u32 {
        self.next() % n
    }
}

fn rgb(c: u32) -> Rgba<u8> {
    Rgba([(c >> 16) as u8, (c >> 8) as u8, c as u8, 255])
}

/// Franjas horizontales de color, de arriba hacia abajo.
fn bands(img: &mut RgbaImage, colors: &[u32]) {
    let n = colors.len() as u32;
    for y in 0..H {
        let c = rgb(colors[(y * n / H) as usize]);
        for x in 0..W {
            img.put_pixel(x, y, c);
        }
    }
}

fn dots(img: &mut RgbaImage, r: &mut Lcg, count: u32, max_y: u32, color: u32) {
    for _ in 0..count {
        let (x, y) = (r.below(W), r.below(max_y));
        img.put_pixel(x, y, rgb(color));
    }
}

/// Colinas: altura que sube y baja de a un píxel.
fn hills(img: &mut RgbaImage, r: &mut Lcg, base: u32, top: u32, body: u32) {
    let mut h = base as i32;
    for x in 0..W {
        h = (h + r.below(3) as i32 - 1).clamp(base as i32 - 4, base as i32 + 3);
        for y in h as u32..H {
            img.put_pixel(x, y, rgb(if y == h as u32 { top } else { body }));
        }
    }
}

fn rect(img: &mut RgbaImage, x: u32, y: u32, w: u32, h: u32, color: u32) {
    for yy in y..(y + h).min(H) {
        for xx in x..(x + w).min(W) {
            img.put_pixel(xx, yy, rgb(color));
        }
    }
}

/// Fondo predefinido número `i` (módulo `COUNT`).
pub fn preset(i: u8) -> RgbaImage {
    let i = i % COUNT;
    let mut img = RgbaImage::new(W, H);
    let mut r = Lcg(0x5eed + i as u32);
    match i {
        // Pizarra: capas de grafito con vetas plateadas.
        0 => {
            bands(
                &mut img,
                &[0x2a292f, 0x302f36, 0x2a292f, 0x35343c, 0x2d2c33],
            );
            dots(&mut img, &mut r, 70, H, 0x222126);
            dots(&mut img, &mut r, 40, H, 0x403f48);
            dots(&mut img, &mut r, 6, H, 0xb8c4d6);
        }
        // Cielo de día con nubes.
        1 => {
            bands(
                &mut img,
                &[0x5aa9e6, 0x6bb5ea, 0x7fc1ee, 0x93cdf1, 0xa9d8f4],
            );
            for (x, y) in [(3, 6), (18, 13), (8, 24)] {
                rect(&mut img, x, y, 9, 2, 0xffffff);
                rect(&mut img, x + 2, y - 1, 4, 1, 0xffffff);
                rect(&mut img, x + 1, y + 2, 8, 1, 0xd9ecf8);
            }
        }
        // Noche estrellada con luna.
        2 => {
            bands(&mut img, &[0x0e1330, 0x121a3d, 0x17214a, 0x1c2857]);
            dots(&mut img, &mut r, 26, H - 6, 0xe8ecff);
            dots(&mut img, &mut r, 10, H - 6, 0x8f9cd9);
            rect(&mut img, 22, 4, 5, 5, 0xf3f0d7);
            rect(&mut img, 24, 4, 3, 2, 0xd8d3ae);
            hills(&mut img, &mut r, 35, 0x22304f, 0x161f38);
        }
        // Pradera: cielo, colinas verdes y tierra.
        3 => {
            bands(&mut img, &[0x8fd0f0, 0x9ed6f2, 0xaedcf3, 0xbfe3f4]);
            rect(&mut img, 4, 4, 4, 4, 0xfff3a3);
            hills(&mut img, &mut r, 26, 0x6cc644, 0x4f9a31);
            rect(&mut img, 0, 35, W, H - 35, 0x7a5532);
        }
        // Atardecer con sol bajo.
        4 => {
            bands(
                &mut img,
                &[0x2d1b4e, 0x5b2a6e, 0x9c3d6e, 0xd8615a, 0xf29b4c, 0xf8c25c],
            );
            rect(&mut img, 11, 24, 10, 6, 0xffe08a);
            rect(&mut img, 13, 22, 6, 2, 0xffe08a);
            hills(&mut img, &mut r, 31, 0x3a1f3f, 0x24132a);
        }
        // Cueva: roca oscura con cristales plateados.
        _ => {
            bands(
                &mut img,
                &[0x15171d, 0x1b1e26, 0x20242d, 0x1b1e26, 0x15171d],
            );
            dots(&mut img, &mut r, 60, H, 0x2a2e38);
            for _ in 0..5 {
                let (x, y) = (r.below(W - 2), 6 + r.below(H - 10));
                rect(&mut img, x, y, 1, 3, 0xb8c4d6);
                rect(&mut img, x + 1, y + 1, 1, 2, 0xe2e8f2);
                img.put_pixel(x, y + 3, rgb(0x5d6779));
            }
        }
    }
    img
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_are_distinct_and_stable() {
        let all: Vec<_> = (0..COUNT).map(preset).collect();
        for (i, a) in all.iter().enumerate() {
            assert_eq!(a.dimensions(), (W, H));
            assert!(a.pixels().all(|p| p[3] == 255), "fondo {i} con huecos");
            assert_eq!(*a, preset(i as u8), "fondo {i} no es determinista");
            for b in &all[i + 1..] {
                assert_ne!(a, b);
            }
        }
        assert_eq!(preset(COUNT), all[0]);
    }

    /// Guarda los fondos ampliados para revisarlos a ojo.
    #[test]
    #[ignore = "solo para revisar a mano; correr con --ignored"]
    fn write_presets() {
        let dir = std::env::var("DEEPSLATE_RENDER_OUT").expect("falta DEEPSLATE_RENDER_OUT");
        for i in 0..COUNT {
            let big = image::imageops::resize(&preset(i), W * 4, H * 4, image::imageops::Nearest);
            big.save(std::path::Path::new(&dir).join(format!("fondo{i}.png")))
                .unwrap();
        }
    }
}
