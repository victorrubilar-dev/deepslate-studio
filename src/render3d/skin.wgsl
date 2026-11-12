// Personaje con luz direccional simple. Los vértices llegan en espacio de cámara.

struct Uniforms {
    mvp: mat4x4<f32>,
    // xyz: dirección de la luz; w: intensidad (0 = color plano).
    light: vec4<f32>,
    // x: índice de la parte seleccionada, o -1.
    selected: vec4<f32>,
};

@group(0) @binding(0) var<uniform> u: Uniforms;
@group(0) @binding(1) var skin: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;

struct VIn {
    @location(0) pos: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) part: f32,
};

struct VOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) shade: f32,
    @location(2) selected: f32,
};

@vertex
fn vs(v: VIn) -> VOut {
    var o: VOut;
    o.clip = u.mvp * vec4<f32>(v.pos, 1.0);
    o.uv = v.uv;
    let lit = 0.6 + 0.4 * max(dot(normalize(v.normal), normalize(u.light.xyz)), 0.0);
    o.shade = mix(1.0, lit, u.light.w);
    o.selected = select(0.0, 1.0, abs(v.part - u.selected.x) < 0.5);
    return o;
}

fn shade(rgb: vec3<f32>, i: VOut) -> vec3<f32> {
    // Menta de ui/style.rs (64, 214, 180).
    let accent = vec3<f32>(0.25, 0.84, 0.71);
    return mix(rgb * i.shade, accent, 0.3 * i.selected);
}

// La capa base se ve siempre opaca, como en el juego. Lo que aún no se pintó se ve gris.
@fragment
fn fs_base(i: VOut) -> @location(0) vec4<f32> {
    let c = textureSample(skin, samp, i.uv);
    let rgb = select(c.rgb, vec3<f32>(0.35), c.a == 0.0);
    return vec4<f32>(shade(rgb, i), 1.0);
}

@fragment
fn fs_overlay(i: VOut) -> @location(0) vec4<f32> {
    let c = textureSample(skin, samp, i.uv);
    if c.a < 0.004 {
        discard;
    }
    return vec4<f32>(shade(c.rgb, i), c.a);
}
