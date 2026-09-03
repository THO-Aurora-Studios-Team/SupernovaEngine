// Supernova 2D sprite shader with per-vertex tint and optional
// alpha cutoff for spritesheet transparency.
struct SpriteUniforms {
    // .xy = screen/camera size, .zw = unused
    viewport : vec4<f32>,
};

@group(0) @binding(0) var<uniform> sprite : SpriteUniforms;
@group(0) @binding(1) var tex_sampler : sampler;
@group(0) @binding(2) var tex : texture_2d<f32>;

struct VsIn {
    @location(0) position : vec2<f32>,
    @location(1) uv       : vec2<f32>,
    @location(2) color    : vec4<f32>,
};

struct VsOut {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) uv    : vec2<f32>,
    @location(1) color : vec4<f32>,
};

@vertex
fn vs_main(in : VsIn) -> VsOut {
    var out : VsOut;
    let x = (in.position.x / sprite.viewport.x) * 2.0 - 1.0;
    let y = (in.position.y / sprite.viewport.y) * 2.0 - 1.0;
    out.clip_position = vec4<f32>(x, -y, 0.0, 1.0);
    out.uv = in.uv;
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in : VsOut) -> @location(0) vec4<f32> {
    let t = textureSample(tex, tex_sampler, in.uv) * in.color;
    if (t.a < 0.05) { discard; }
    return t;
}
