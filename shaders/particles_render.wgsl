struct Particle {
    pos: vec2<f32>,
    vel: vec2<f32>,
    color: vec4<f32>,
    age: f32,
    lifetime: f32,
};
@group(0) @binding(0) var<storage, read> particles: array<Particle>;

struct Uniforms {
    mvp: mat4x4<f32>,
    dt: f32,
    count: u32,
    gravity: vec2<f32>,
    _pad1: vec4<f32>,
};
@group(0) @binding(1) var<uniform> uniforms: Uniforms;

struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(@builtin(instance_index) idx: u32) -> VOut {
    var o: VOut;
    let p = particles[idx];
    o.pos = uniforms.mvp * vec4<f32>(p.pos, 0.0, 1.0);
    
    // Fade out based on age/lifetime
    let life_t = p.age / p.lifetime;
    let alpha = 1.0 - life_t;
    o.color = vec4<f32>(p.color.rgb, p.color.a * alpha);
    return o;
}

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    if in.color.a < 0.01 { discard; }
    return in.color;
}
