// Supernova PBR shader — reference implementation.
// Provides directional + point lighting, normal mapping, and
// metallic/roughness workflow. Intended as a solid starting point
// for project shaders; replace with your own tuned version as needed.

struct FrameUniforms {
    view_proj   : mat4x4<f32>,
    camera_pos  : vec4<f32>,
    sun_dir     : vec4<f32>,
    sun_color   : vec4<f32>,
    point_pos   : vec4<f32>,
    point_color : vec4<f32>,
    ambient     : vec4<f32>,
};

@group(0) @binding(0) var<uniform> frame : FrameUniforms;
@group(0) @binding(1) var albedoSampler : sampler;
@group(0) @binding(2) var albedoMap : texture_2d<f32>;
@group(0) @binding(3) var normalMap : texture_2d<f32>;
@group(0) @binding(4) var mrmapMap : texture_2d<f32>;

struct ModelUniforms {
    model       : mat4x4<f32>,
    base_color  : vec4<f32>,
    emissive    : vec4<f32>,
    params      : vec4<f32>, // .x = metallic, .y = roughness, .z = normal_scale, .w = unused
};

@group(1) @binding(0) var<uniform> model : ModelUniforms;

struct VsIn {
    @location(0) position : vec3<f32>,
    @location(1) normal   : vec3<f32>,
    @location(2) tangent  : vec3<f32>,
    @location(3) uv       : vec2<f32>,
};

struct VsOut {
    @builtin(position) clip_position : vec4<f32>,
    @location(0) world_pos : vec3<f32>,
    @location(1) world_nrm : vec3<f32>,
    @location(2) world_tan : vec3<f32>,
    @location(3) uv        : vec2<f32>,
};

@vertex
fn vs_main(in : VsIn) -> VsOut {
    var out : VsOut;
    out.world_pos = (model.model * vec4<f32>(in.position, 1.0)).xyz;
    out.clip_position = frame.view_proj * vec4<f32>(out.world_pos, 1.0);
    out.world_nrm = normalize((model.model * vec4<f32>(in.normal, 0.0)).xyz);
    out.world_tan = normalize((model.model * vec4<f32>(in.tangent, 0.0)).xyz);
    out.uv = in.uv;
    return out;
}

fn srgb_to_linear(c : vec3<f32>) -> vec3<f32> {
    return pow(c, vec3<f32>(2.2));
}

@fragment
fn fs_main(in : VsOut) -> @location(0) vec4<f32> {
    let albedo = srgb_to_linear(textureSample(albedoMap, albedoSampler, in.uv).rgb) * model.base_color.rgb;

    // Sample tangent-space normal and transform to world space.
    var n = textureSample(normalMap, albedoSampler, in.uv).xyz * 2.0 - 1.0;
    n *= model.params.z;
    let bitangent = cross(in.world_nrm, in.world_tan);
    var N = normalize(in.world_tan * n.x + bitangent * n.y + in.world_nrm * n.z);
    if (length(N) < 0.001) { N = in.world_nrm; }

    let metalRough = textureSample(mrmapMap, albedoSampler, in.uv);
    let metallic  = metalRough.b * model.params.x;
    let roughness = metalRough.g * model.params.y;

    let V = normalize(frame.camera_pos.xyz - in.world_pos);
    let NdotV = max(dot(N, V), 0.0);

    // Directional sun.
    let L_sun = normalize(-frame.sun_dir.xyz);
    let NdotL_sun = max(dot(N, L_sun), 0.0);
    let diff_sun = albedo * (1.0 - metallic) * NdotL_sun;
    let half_sun = normalize(V + L_sun);
    let spec_sun = pow(max(dot(N, half_sun), 0.0), max(1.0 - roughness * 0.85, 0.001) * 128.0) * NdotL_sun;
    let color_sun = frame.sun_color.rgb * (diff_sun + spec_sun * vec3<f32>(0.3 + 0.7 * metallic));

    // Point light.
    let L_p = normalize(frame.point_pos.xyz - in.world_pos);
    let dist = distance(frame.point_pos.xyz, in.world_pos);
    let atten = 1.0 / (1.0 + 0.09 * dist + 0.032 * dist * dist);
    let NdotL_p = max(dot(N, L_p), 0.0);
    let diff_p = albedo * (1.0 - metallic) * NdotL_p * atten;
    let color_p = frame.point_color.rgb * diff_p;

    // Ambient.
    let ambient = albedo * frame.ambient.rgb * frame.ambient.a;

    let color = ambient + color_sun + color_p + model.emissive.rgb * srgb_to_linear(albedo * 0.0 + 1.0);
    return vec4<f32>(color * (1.0 - metallic) + albedo * metallic, model.base_color.a);
}
