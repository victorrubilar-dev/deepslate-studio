//! Utilidades de color: código hex y paleta a partir de una imagen.

use image::RgbaImage;
use std::collections::HashMap;

/// `#RRGGBB`, o `#RRGGBBAA` si el color no es opaco.
pub fn to_hex(c: [u8; 4]) -> String {
    let [r, g, b, a] = c;
    if a == 255 {
        format!("#{r:02X}{g:02X}{b:02X}")
    } else {
        format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
    }
}

/// Acepta `RRGGBB` o `RRGGBBAA`, con o sin `#`.
pub fn parse_hex(s: &str) -> Option<[u8; 4]> {
    let s = s.trim().trim_start_matches('#');
    if !matches!(s.len(), 6 | 8) || !s.is_ascii() {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&s[i..i + 2], 16).ok();
    Some([
        byte(0)?,
        byte(2)?,
        byte(4)?,
        if s.len() == 8 { byte(6)? } else { 255 },
    ])
}

/// Los `max` colores más usados de la imagen, sin contar los transparentes.
pub fn palette_from(img: &RgbaImage, max: usize) -> Vec<[u8; 4]> {
    let mut count: HashMap<[u8; 4], usize> = HashMap::new();
    for px in img.pixels().filter(|p| p[3] > 0) {
        *count.entry(px.0).or_default() += 1;
    }
    let mut colors: Vec<_> = count.into_iter().collect();
    // Desempate por valor para que el resultado no dependa del orden del HashMap.
    colors.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    colors.into_iter().take(max).map(|(c, _)| c).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    #[test]
    fn hex_roundtrip() {
        for c in [[0, 0, 0, 255], [196, 92, 52, 255], [1, 2, 3, 4]] {
            assert_eq!(parse_hex(&to_hex(c)), Some(c));
        }
        assert_eq!(parse_hex("ff8000"), Some([255, 128, 0, 255]));
        assert_eq!(parse_hex(" #FF8000 "), Some([255, 128, 0, 255]));
        for bad in ["", "#fff", "#gg0000", "#ff00001", "#ñañaña"] {
            assert_eq!(parse_hex(bad), None, "{bad}");
        }
    }

    #[test]
    fn palette_by_frequency() {
        let mut img = RgbaImage::new(4, 1);
        img.put_pixel(0, 0, Rgba([9, 9, 9, 255]));
        img.put_pixel(1, 0, Rgba([1, 1, 1, 255]));
        img.put_pixel(2, 0, Rgba([1, 1, 1, 255]));
        // (3, 0) queda transparente y no cuenta.
        assert_eq!(palette_from(&img, 10), vec![[1, 1, 1, 255], [9, 9, 9, 255]]);
        assert_eq!(palette_from(&img, 1).len(), 1);
    }
}
