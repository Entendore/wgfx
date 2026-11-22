struct Uniforms { mvp: mat4x4<f32> };
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) pos: vec3<f32>,
    @location(1) color: vec4<f32>
) -> VOut {
    var o: VOut;
    o.pos = uniforms.mvp * vec4<f32>(pos, 1.0);
    o.color = color;
    return o;
}

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    return in.color;
}
