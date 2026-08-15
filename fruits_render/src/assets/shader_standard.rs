pub fn shader_standard(is_lit: bool, is_transparent: bool) -> String {
    let mut shader = String::new();

    shader.push_str(&code_shader_inputs());
    shader.push_str(&code_lib());
    shader.push_str(&code_fn_vs_main());
    shader.push_str(&code_fn_map_instance_data());
    if is_lit {
        shader.push_str(&code_lit_stuff());
    } else {
        shader.push_str(&code_unlit_stuff());
    }
    shader.push_str(&code_fn_fs_main(is_lit, is_transparent));

    shader
}

fn code_shader_inputs() -> String {
    String::from(r#"
@group(0) @binding(0) var<uniform> global_data: GlobalData;
@group(0) @binding(1) var<storage, read> lights: array<GenericLight>;

@group(1) @binding(0) var<uniform> material_data: MaterialData;
@group(1) @binding(1) var color_texture: texture_2d<f32>;
@group(1) @binding(2) var color_sampler: sampler;
@group(1) @binding(3) var roughness_texture: texture_2d<f32>;
@group(1) @binding(4) var roughness_sampler: sampler;
@group(1) @binding(5) var metallic_texture: texture_2d<f32>;
@group(1) @binding(6) var metallic_sampler: sampler;
@group(1) @binding(7) var normal_texture: texture_2d<f32>;
@group(1) @binding(8) var normal_sampler: sampler;
@group(1) @binding(9) var emission_texture: texture_2d<f32>;
@group(1) @binding(10) var emission_sampler: sampler;
    "#)
}

// todo: light structure. Needs to support lights:
// - Point (hdr-color, center, range)
// - Spot (hdr-color, center, range, direction_dst, fov)
// - Directional (hdr-color, direction_dst)
// - Area? (todo for the future)

fn code_lib() -> String {
    String::from(r#"
struct GlobalData {
    camera_position_world: vec3<f32>,
    lights_count: u32,
}

struct MaterialData {
    matrix_world_to_clip: mat4x4<f32>,
    color: vec4<f32>,
    emission_color: vec4<f32>,
    metallic: f32,
    roughness: f32,
    alpha_threshold: f32,
}

struct VertexAttributes {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec4<f32>,
    @location(3) color: vec4<f32>,
    @location(4) uv: vec2<f32>,
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

// Lighting

const PI = radians(180.0);

struct GenericLight {
    color: vec3<f32>,
    light_type: u32,
    center: vec3<f32>,
    range: f32,
    direction_dst: vec3<f32>,
    fov: f32,
}

struct FragmentLight {
    direction_src: vec3<f32>,
    color: vec3<f32>,
}

fn light_generic_to_fragment(light: GenericLight, fragment_pos_world: vec3<f32>) -> FragmentLight {
    var output: FragmentLight;

    output.color = light.color;

    if (light.light_type == 0) {
        // point
        output.direction_src = normalize(light.center - fragment_pos_world);
        var distance_vec = light.center - fragment_pos_world;
        var distance_sqr = dot(distance_vec, distance_vec);
        output.color /= distance_sqr;
    } else if (light.light_type == 1) {
        // spot
        output.direction_src = normalize(light.center - fragment_pos_world);
        var distance_vec = light.center - fragment_pos_world;
        var distance_sqr = dot(distance_vec, distance_vec);
        output.color /= distance_sqr;
        var projected_fragment_pos = light.center + light.direction_dst * dot(fragment_pos_world - light.center, light.direction_dst);
        var angle = acos(distance(light.center, projected_fragment_pos) / distance(light.center, fragment_pos_world)) * 2.0;
        output.color *= step(angle, light.fov);
    } else if (light.light_type == 2) {
        // directional
        output.direction_src = -light.direction_dst;
    } else {
        output.direction_src = vec3<f32>(0.0, 1.0, 0.0);
    }

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
    light_direction_src: vec3<f32>,
    light_color: vec3<f32>
) -> vec3<f32> {
    let h = normalize(v + light_direction_src);
   
    let nv = max(dot(n, v), 0.0);
    let nl = max(dot(n, light_direction_src), 0.0);
   
    let f0 = mix(vec3<f32>(0.04), albedo, metallic);
    let fresnel = fresnel_schlick(max(dot(h, v), 0.0), f0);
   
    let d = ggx_distribution(n, h, roughness);
    let g = geometry_schlick_ggx(n, v, roughness) * geometry_schlick_ggx(n, light_direction_src, roughness);
   
    let specular = (d * g * fresnel) / max(4.0 * nv * nl, 0.001);
   
    let k_d = (1.0 - fresnel) * (1.0 - metallic);
    let diffuse = (k_d * albedo) / PI;
   
    return (diffuse + specular) * light_color * nl;
}

fn create_matrix_tbn(normal: vec3<f32>, tangent_handedness: vec4<f32>) -> mat3x3<f32> {
    let N = normalize(normal);
    var T = normalize(tangent_handedness.xyz);
    T = normalize(T - N * dot(N, T));
    let B = cross(N, T) * tangent_handedness.w;
    return mat3x3<f32>(T, B, N);
}
    "#)
}

fn code_fn_vs_main() -> String {
    String::from(r#"
@vertex
fn vs_main(vertex: VertexAttributes, instance_raw: InstanceRawAttributes) -> VertexOutput {
    var instance = map_instance_data(instance_raw);

    return customer_vertex(vertex, instance);
}
    "#)
}

fn code_fn_map_instance_data() -> String {
    String::from(r#"
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
    "#)
}

fn code_lit_stuff() -> String {
    String::from(r#"
struct VertexOutput {
    @builtin(position) position_clip: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) normal_world: vec3<f32>,
    @location(2) tangent_world: vec4<f32>,
    @location(3) position_world: vec3<f32>,
    @location(4) uv: vec2<f32>,
};

fn customer_vertex(vertex: VertexAttributes, instance: InstanceAttributes) -> VertexOutput {
    var out: VertexOutput;

    var position_world = instance.local_to_world * vec4<f32>(vertex.position, 1.0);

    out.position_clip = material_data.matrix_world_to_clip * position_world;
    out.color = vertex.color * material_data.color;
    out.normal_world = (instance.local_to_world * vec4<f32>(vertex.normal, 0.0)).xyz;
    out.tangent_world = vec4<f32>((instance.local_to_world * vec4<f32>(vertex.tangent.xyz, 0.0)).xyz, vertex.tangent.w);
    out.position_world = position_world.xyz;
    out.uv = vertex.uv;
   
    return out;
}
    "#)
}

fn code_unlit_stuff() -> String {
    String::from(r#"
struct VertexOutput {
    @builtin(position) position_clip: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

fn customer_vertex(vertex: VertexAttributes, instance: InstanceAttributes) -> VertexOutput {
    var out: VertexOutput;

    var position_world = instance.local_to_world * vec4<f32>(vertex.position, 1.0);

    out.position_clip = material_data.matrix_world_to_clip * position_world;
    out.color = vertex.color * material_data.color;
    out.uv = vertex.uv;
   
    return out;
}
    "#)
}

fn code_fn_fs_main(is_lit: bool, is_transparent: bool) -> String {
    let mut code = String::new();

    code.push_str(
        r#"
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = in.color * textureSample(color_texture, color_sampler, in.uv);
    "#,
    );

    if !is_transparent {
        code.push_str(
            r#"
    if (color.w < material_data.alpha_threshold) {
        discard;
    }

    color.w = 1.0;
        "#,
        );
    }

    if is_lit {
        code.push_str(
            r#"
    var diffuse_color = color.xyz;

    var emission = material_data.emission_color.xyz * material_data.emission_color.w * textureSample(emission_texture, emission_sampler, in.uv).xyz;

    color = vec4<f32>(emission, color.w);

    let normal_tangent_space = textureSample(normal_texture, normal_sampler, in.uv).xyz * 2.0 - 1.0;
    let TBN = create_matrix_tbn(in.normal_world, in.tangent_world);
    let normal_world = normalize(TBN * normal_tangent_space);

    var view_dir = normalize(global_data.camera_position_world - in.position_world);
    var metallic = material_data.metallic * textureSample(metallic_texture, metallic_sampler, in.uv).x;
    var roughness = material_data.roughness * textureSample(roughness_texture, roughness_sampler, in.uv).x;

    for (var i: u32 = 0; i < global_data.lights_count; i++) {
        var light = light_generic_to_fragment(lights[i], in.position_world);

        var color_lit = cook_torrance_brdf(
            normal_world,
            view_dir,
            diffuse_color,
            metallic,
            roughness,
            light.direction_src,
            light.color
        );

        color = vec4<f32>(color.xyz + color_lit, color.w);
    }

        "#,
        );
    }

    code.push_str(
        r#"
    return color;
}
    "#,
    );

    code
}
