struct Uniforms {
    grayscale: u32,
    vignette_intensity: f32,
    vignette_radius: f32,
    chromatic_aberration: f32,
    brightness: f32,
    contrast: f32,
    _pad: vec2<f32>,
};
@group(0) @binding(0) var t_src: texture_2d<f32>;
@group(0) @binding(1) var s_src: sampler;
@group(0) @binding(2) var<uniform> uniforms: Uniforms;

struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VOut {
    var o: VOut;
    o.pos = vec4<f32>(-1.0, -1.0, 0.0, 1.0);
    o.uv = vec2<f32>(0.0, 1.0);
    if idx == 1 {
        o.pos = vec4<f32>(3.0, -1.0, 0.0, 1.0);
        o.uv = vec2<f32>(2.0, 1.0);
    }
    if idx == 2 {
        o.pos = vec4<f32>(-1.0, 3.0, 0.0, 1.0);
        o.uv = vec2<f32>(0.0, -1.0);
    }
    return o;
}

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    var color: vec4<f32>;
    
    // Chromatic aberration
    if uniforms.chromatic_aberration > 0.0 {
        let ca = uniforms.chromatic_aberration;
        let r = textureSample(t_src, s_src, in.uv + vec2<f32>(ca, 0.0)).r;
        let g = textureSample(t_src, s_src, in.uv).g;
        let b = textureSample(t_src, s_src, in.uv - vec2<f32>(ca, 0.0)).b;
        color = vec4<f32>(r, g, b, 1.0);
    } else {
        color = textureSample(t_src, s_src, in.uv);
    }
    
    // Brightness (WGSL forbids assigning to swizzles like color.rgb, so we reconstruct the vec4)
    color = vec4<f32>(color.rgb * uniforms.brightness, color.a);
    
    // Contrast
    color = vec4<f32>((color.rgb - 0.5) * uniforms.contrast + 0.5, color.a);
    
    // Grayscale
    if uniforms.grayscale == 1u {
        let gray = dot(color.rgb, vec3<f32>(0.299, 0.587, 0.114));
        color = vec4<f32>(vec3<f32>(gray), color.a);
    }
    
    // Vignette
    let dist = distance(in.uv, vec2<f32>(0.5));
    let vignette = 1.0 - smoothstep(uniforms.vignette_radius, uniforms.vignette_radius + 0.3, dist) * uniforms.vignette_intensity;
    color = vec4<f32>(color.rgb * vignette, color.a);
    
    return vec4<f32>(color.rgb, 1.0);
}
