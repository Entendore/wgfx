struct Particle {
    pos: vec2<f32>,
    vel: vec2<f32>,
    color: vec4<f32>,
    age: f32,
    lifetime: f32,
};
@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;

struct Uniforms {
    mvp: mat4x4<f32>,
    dt: f32,
    count: u32,
    gravity: vec2<f32>,
    _pad1: vec4<f32>,
};
@group(0) @binding(1) var<uniform> uniforms: Uniforms;

@compute @workgroup_size(64)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    let idx = id.x;
    // Guard against going out of bounds of the active particle count
    if idx >= uniforms.count { return; }
    
    var p = particles[idx];
    
    // Only update living particles
    if p.age < p.lifetime {
        p.age += uniforms.dt;
        p.vel += uniforms.gravity * uniforms.dt;
        p.pos += p.vel * uniforms.dt;
    }
    
    particles[idx] = p;
}
