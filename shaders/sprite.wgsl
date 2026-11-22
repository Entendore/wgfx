struct Uniforms { mvp: mat4x4<f32> };
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var t_diffuse: texture_2d<f32>;
@group(0) @binding(2) var s_diffuse: sampler;

struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @location(0) v_pos: vec2<f32>,
    @location(1) v_uv: vec2<f32>,
    @location(2) i_pos: vec2<f32>,
    @location(3) i_size: vec2<f32>,
    @location(4) i_uv_off: vec2<f32>,
    @location(5) i_uv_scale: vec2<f32>,
    @location(6) i_color: vec4<f32>
) -> VOut {
    var o: VOut;
    o.pos = uniforms.mvp * vec4<f32>(v_pos * i_size + i_pos, 0.0, 1.0);
    o.color = i_color;
    o.uv = v_uv * i_uv_scale + i_uv_off;
    return o;
}

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_diffuse, s_diffuse, in.uv);
    let final_color = tex_color * in.color;
    if final_color.a < 0.01 { discard; }
    return final_color;
}
