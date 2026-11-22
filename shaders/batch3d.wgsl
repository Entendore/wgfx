struct Uniforms {
    view_proj: mat4x4<f32>,
    light_dir: vec3<f32>,
    time: f32,
};
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var t_diffuse: texture_2d<f32>;
@group(0) @binding(2) var s_diffuse: sampler;

struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

@vertex
fn vs_main(
    @location(0) pos: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) m0: vec4<f32>,
    @location(4) m1: vec4<f32>,
    @location(5) m2: vec4<f32>,
    @location(6) m3: vec4<f32>,
    @location(7) i_color: vec4<f32>
) -> VOut {
    var o: VOut;
    let model = mat4x4<f32>(m0, m1, m2, m3);
    
    o.pos = uniforms.view_proj * model * vec4<f32>(pos, 1.0); 
    o.color = i_color;
    o.uv = uv;
    o.normal = normalize((model * vec4<f32>(normal, 0.0)).xyz);
    return o;
}

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_diffuse, s_diffuse, in.uv);
    let light = max(dot(in.normal, uniforms.light_dir), 0.0);
    let ambient = 0.2;
    let diffuse = light * 0.8;
    let final_color = in.color.rgb * tex_color.rgb * (vec3<f32>(ambient) + vec3<f32>(diffuse));
    return vec4<f32>(final_color, in.color.a);
}
