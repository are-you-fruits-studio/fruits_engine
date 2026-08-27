@group(0) @binding(0) var src_texture: texture_2d<f32>;
@group(0) @binding(1) var src_sampler: sampler;
@group(0) @binding(2) var<uniform> bloom_uniform: BloomUniform;
@group(0) @binding(3) var<storage, read> storage_layers: array<BloomStorageLayer>;

struct BloomUniform {
    threshold: vec2<f32>,
    intensity: f32,
}

struct BloomStorageLayer {
    uv_scale_offset: vec4<f32>,
    direction: vec2<f32>,
    intensity: f32,
}

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) instance_index: u32,
    @location(1) uv: vec2<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    var pos = array<vec2f, 3>(
        vec2f(-1.0, -3.0),
        vec2f(3.0, 1.0),
        vec2f(-1.0, 1.0)
    );

    let p = pos[in.vertex_index];

    out.position = vec4f(p, 0.0, 1.0);
    out.instance_index = in.instance_index;
    out.uv = p * 0.5 + 0.5;
    out.uv.y = 1.0 - out.uv.y;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let layer = storage_layers[in.instance_index];

    let uv = in.uv * layer.uv_scale_offset.xy + layer.uv_scale_offset.zw;
    let src_color = textureSampleLevel(src_texture, src_sampler, uv, 0.0);

    return vec4f(src_color.xyz, 1.0);
}