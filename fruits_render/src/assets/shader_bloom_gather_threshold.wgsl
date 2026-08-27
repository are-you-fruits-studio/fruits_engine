@group(0) @binding(0) var src_texture: texture_2d<f32>;
@group(0) @binding(1) var src_sampler: sampler;
@group(0) @binding(2) var<uniform> threshold: vec2<f32>;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4f {
    var pos = array<vec2f, 3>(
        vec2f(-1.0, -3.0),
        vec2f(3.0, 1.0),
        vec2f(-1.0, 1.0)
    );
    return vec4f(pos[vertex_index], 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    let dim = textureDimensions(src_texture);
    let uv = pos.xy / vec2f(f32(dim.x), f32(dim.y));
    let src_color = textureSampleLevel(src_texture, src_sampler, uv, 0.0);

    let luminance = src_color.x + src_color.y + src_color.z;

    let epsilon = abs(threshold.y * 0.5);

    let t = smoothstep(threshold.x - epsilon, threshold.x + epsilon, luminance);

    return vec4f(mix(vec3f(0.0, 0.0, 0.0), src_color.xyz, t), 1.0);
}