// Supernova unlit (flat/texture) shader — good for UI, sprites, and
// stylized geometry where you don't need lighting.
struct FrameUniforms {
    view_proj : mat4x4<f32>,
};

@group(0) @binding(0) var<uniform> frame : FrameUniforms;
@group(0) @binding(1) var tex_sampler : sampler;
@group(0) @binding(2) var tex : texture_2d<f32>;

struct ModelUniforms {
    model      : mat4x4<f32>,
    tint       : vec4<f32>,
};

@group(1) @binding(0) var<uniform> model : ModelUniforms;

struct VsIn {
    @location(0) position : vec3<f32>,
    @location(1) normal   : vec3<f32>,
    @location(2) uv       : vec2<f32>,
};

struct VsOut {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) uv : vec2<f32>,
};

@vertex
fn vs_main(in : VsIn) -> VsOut {
    var out : VsOut;
    out.clip_position = frame.view_proj * model.model * vec4<f32>(in.position, 1.0);
    out.uv = in.uv;
    return out;
}

@fragment
fn fs_main(in : VsOut) -> @location(0) vec4<f32> {
    return textureSample(tex, tex_sampler, in.uv) * model.tint;
}
