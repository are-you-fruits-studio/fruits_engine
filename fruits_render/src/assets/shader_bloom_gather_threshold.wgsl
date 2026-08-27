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
    @builtin(position) position: vec4f,
    @location(0) instance_index: u32,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var pos = array<vec2f, 3>(
        vec2f(-1.0, -3.0),
        vec2f(3.0, 1.0),
        vec2f(-1.0, 1.0)
    );

    var out: VertexOutput;

    out.position = vec4f(pos[in.vertex_index], 0.0, 1.0);
    out.instance_index = in.instance_index;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let dim = textureDimensions(src_texture);
    let uv = in.position.xy / vec2f(f32(dim.x), f32(dim.y));
    let src_color = textureSampleLevel(src_texture, src_sampler, uv, 0.0);

    let luminance = src_color.x + src_color.y + src_color.z;

    let threshold = bloom_uniform.threshold;

    let epsilon = abs(threshold.y * 0.5);

    let t = smoothstep(threshold.x - epsilon, threshold.x + epsilon, luminance);

    return vec4f(mix(vec3f(0.0, 0.0, 0.0), src_color.xyz, t), 1.0);
}