@group(0) @binding(0) var src_texture: texture_2d<f32>;
@group(0) @binding(1) var src_sampler: sampler;
@group(0) @binding(2) var<uniform> uv_scale_offset: vec4<f32>;
@group(0) @binding(3) var<uniform> direction: vec2<f32>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    let pos = array<vec2f, 3>(
        vec2f(-1.0, -3.0),
        vec2f(3.0, 1.0),
        vec2f(-1.0, 1.0)
    );

    let p = pos[vertex_index];

    out.position = vec4f(p, 0.0, 1.0);
    out.uv = p * 0.5 + 0.5;

    out.uv.y = 1.0 - out.uv.y;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let uv = in.uv * uv_scale_offset.xy + uv_scale_offset.zw;

    let texel = 1.0 / (vec2<f32>(textureDimensions(src_texture).xy));

    let offset = texel * direction;

    let weights = array<f32, 3>(
        0.17839992574371122,
        0.21043088737683882,
        0.22233837375889995,
    );

    var sample = vec3<f32>(0.0);
    sample += textureSample(src_texture, src_sampler, uv - 2.0 * offset).xyz * weights[0];
    sample += textureSample(src_texture, src_sampler, uv - 1.0 * offset).xyz * weights[1];
    sample += textureSample(src_texture, src_sampler, uv + 0.0 * offset).xyz * weights[2];
    sample += textureSample(src_texture, src_sampler, uv + 1.0 * offset).xyz * weights[1];
    sample += textureSample(src_texture, src_sampler, uv + 2.0 * offset).xyz * weights[0];

    return vec4f(sample, 1.0);
}