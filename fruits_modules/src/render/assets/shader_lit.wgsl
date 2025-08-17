@group(0) @binding(0) var<uniform> global_data: GlobalData;
@group(1) @binding(0) var albedo_texture: texture_2d<f32>;
@group(1) @binding(1) var albedo_sampler: sampler;

struct GlobalData {
    matrix_world_to_clip: mat4x4<f32>,
    albedo_color: vec4<f32>,
    emission_color: vec4<f32>,
    camera_position_world: vec3<f32>,
    metallic: f32,
    roughness: f32,
    alpha_threshold: f32,
}

struct VertexAttributes {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv: vec2<f32>,
};

struct InstanceRawAttributes {
    @location(5) local_to_world_c0: vec4<f32>,
    @location(6) local_to_world_c1: vec4<f32>,
    @location(7) local_to_world_c2: vec4<f32>,
    @location(8) local_to_world_c3: vec4<f32>,
};

struct InstanceAttributes {
    local_to_world: mat4x4<f32>,
};

@vertex
fn vs_main(vertex: VertexAttributes, instance_raw: InstanceRawAttributes) -> VertexOutput {
    var instance = map_instance_data(instance_raw);

    return customer_vertex(vertex, instance);
}

fn map_instance_data(input: InstanceRawAttributes) -> InstanceAttributes {
    var output: InstanceAttributes;

    output.local_to_world = mat4x4<f32>(
        input.local_to_world_c0,
        input.local_to_world_c1,
        input.local_to_world_c2,
        input.local_to_world_c3
    );

    return output;
}

// Lighting

const PI = radians(180.0);

struct Light {
    direction: vec3<f32>,
    color: vec3<f32>,
}

fn light() -> Light {
    var output: Light;

    output.direction = normalize(vec3<f32>(2.0, 3.0, -1.0));
    output.color = vec3<f32>(1.0, 1.0, 1.0) * 5.0;

    return output;
}

fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (vec3<f32>(1.0) - f0) * pow(1.0 - cos_theta, 5.0);
}

fn ggx_distribution(n: vec3<f32>, h: vec3<f32>, roughness: f32) -> f32 {
    let alpha = roughness * roughness;
    let alpha2 = alpha * alpha;
    let nh = max(dot(n, h), 0.0);
    let denom = (nh * nh) * (alpha2 - 1.0) + 1.0;
    return alpha2 / (PI * denom * denom);
}

fn geometry_schlick_ggx(n: vec3<f32>, v: vec3<f32>, roughness: f32) -> f32 {
    let k = (roughness + 1.0) * (roughness + 1.0) / 8.0;
    let nv = max(dot(n, v), 0.0);
    return nv / (nv * (1.0 - k) + k);
}

fn cook_torrance_brdf(
    n: vec3<f32>,
    v: vec3<f32>,
    albedo: vec3<f32>,
    metallic: f32,
    roughness: f32,
    light_direction: vec3<f32>,
    light_color: vec3<f32>
) -> vec3<f32> {
    let h = normalize(v + light_direction);
    
    let nv = max(dot(n, v), 0.0);
    let nl = max(dot(n, light_direction), 0.0);
    
    let f0 = mix(vec3<f32>(0.04), albedo, metallic);
    let fresnel = fresnel_schlick(max(dot(h, v), 0.0), f0);
    
    let d = ggx_distribution(n, h, roughness);
    let g = geometry_schlick_ggx(n, v, roughness) * geometry_schlick_ggx(n, light_direction, roughness);
    
    let specular = (d * g * fresnel) / max(4.0 * nv * nl, 0.001);
    
    let k_d = (1.0 - fresnel) * (1.0 - metallic);
    let diffuse = (k_d * albedo) / PI;
    
    return (diffuse + specular) * light_color * nl;
}

//

struct VertexOutput {
    @builtin(position) position_clip: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) normal_world: vec3<f32>,
    @location(2) position_world: vec3<f32>,
    @location(3) uv: vec2<f32>,
};

fn customer_vertex(vertex: VertexAttributes, instance: InstanceAttributes) -> VertexOutput {
    var out: VertexOutput;

    var position_world = instance.local_to_world * vec4<f32>(vertex.position, 1.0);

    out.position_clip = global_data.matrix_world_to_clip * position_world;
    out.color = vertex.color * global_data.albedo_color;
    out.normal_world = (instance.local_to_world * vec4<f32>(vertex.normal, 0.0)).xyz;
    out.position_world = position_world.xyz;
    out.uv = vertex.uv;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var light = light();

    let albedo_color = in.color * textureSample(albedo_texture, albedo_sampler, in.uv);

    if (albedo_color.w < global_data.alpha_threshold) {
        discard;
    }

    var color_lit = cook_torrance_brdf(
        normalize(in.normal_world),
        normalize(global_data.camera_position_world - in.position_world),
        albedo_color.xyz,
        global_data.metallic,
        global_data.roughness,
        light.direction,
        light.color
    );

    return vec4<f32>(color_lit + global_data.emission_color.xyz, 1.0);
}