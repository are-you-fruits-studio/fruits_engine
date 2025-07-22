@group(0) @binding(0) var<uniform> global_data: GlobalData;

struct GlobalData {
    matrix_world_to_clip: mat4x4<f32>,
    color: vec4<f32>,
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

struct VertexOutput {
    @builtin(position) position_clip: vec4<f32>,
    @location(0) color: vec4<f32>,
};

fn customer_vertex(vertex: VertexAttributes, instance: InstanceAttributes) -> VertexOutput {
    var out: VertexOutput;

    var position_world = instance.local_to_world * vec4<f32>(vertex.position, 1.0);

    out.position_clip = global_data.matrix_world_to_clip * position_world;
    out.color = vertex.color * global_data.color;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color.xyz, 1.0);
}