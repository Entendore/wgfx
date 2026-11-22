struct Uniforms { mvp: mat4x4<f32> };
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VOut {
    @builtin(position) pos: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) kind: u32,
};

@vertex
fn vs_main(
    @location(0) pos: vec2<f32>,
    @location(1) i_pos: vec2<f32>,
    @location(2) i_size: vec2<f32>,
    @location(3) i_color: vec4<f32>,
    @location(4) i_kind: u32,
    @location(5) i_rot: f32
) -> VOut {
    var o: VOut;
    let cs = cos(i_rot);
    let sn = sin(i_rot);
    let p = pos - 0.5;
    let rp = vec2<f32>(p.x * cs - p.y * sn, p.x * sn + p.y * cs);
    o.pos = uniforms.mvp * vec4<f32>(rp * i_size + i_pos, 0.0, 1.0);
    o.color = i_color;
    o.uv = pos;
    o.kind = i_kind;
    return o;
}

@fragment
fn fs_main(in: VOut) -> @location(0) vec4<f32> {
    if (in.kind == 1u) {
        // Circle
        let d = distance(in.uv, vec2<f32>(0.5));
        if d > 0.5 { discard; }
        let alpha = 1.0 - smoothstep(0.45, 0.5, d);
        return vec4<f32>(in.color.rgb, in.color.a * alpha);
    }
    if (in.kind == 2u) {
        // Rounded rect
        let radius = 0.3;
        let center = vec2<f32>(0.5);
        let d = max(abs(in.uv - center) - (vec2<f32>(0.5) - vec2<f32>(radius)), vec2<f32>(0.0));
        let dist = length(d);
        if dist > radius { discard; }
        let alpha = 1.0 - smoothstep(radius - 0.05, radius, dist);
        return vec4<f32>(in.color.rgb, in.color.a * alpha);
    }
    // Rect (default)
    return in.color;
}
