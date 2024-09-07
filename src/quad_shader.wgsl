var<private> v_positions: array<vec2<f32>, 6> = array<vec2<f32>, 6> (
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(1.0, -1.0),
);

// texture coordinates are flipped on the y axis
var<private> v_texcoords: array<vec2<f32>, 6> = array<vec2<f32>, 6> (
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(1.0, 1.0),
);

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texcoord: vec2<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(v_positions[in_vertex_index], 0.0, 1.0);
    out.texcoord = v_texcoords[in_vertex_index];
    return out;
}

@group(0) @binding(0)
var tile_atlas: texture_2d<f32>;
@group(0) @binding(1)
var atlas_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //return vec4<f32>(in.texcoord, 0.0, 1.0);
    return textureSample(tile_atlas, atlas_sampler, in.texcoord);
}