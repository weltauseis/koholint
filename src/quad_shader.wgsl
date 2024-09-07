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

@group(0) @binding(2)

var<storage> tile_map_indices: array<u32>;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32, @builtin(instance_index) in_instance_index: u32) -> VertexOutput {
    var out: VertexOutput;

    // place the quad based on the instance
    let i = f32(in_instance_index);
    var clip_position = (v_positions[in_vertex_index]);
    clip_position /= 32.0;
    clip_position += vec2<f32>(-1.0, 1.0);
    clip_position += vec2<f32>(1.0/32.0, -1.0/32.0); // top left corner
    clip_position += (i % 32.0) * vec2<f32>(1.0/16.0, 0.0);
    clip_position += floor(i / 32.0) * vec2<f32>(0.0, -1.0/16.0); // shifted x and y

    // place the texture coordinates on the atlas based on the index in the tile map indices buffer

    let tilemap_idx = tile_map_indices[in_instance_index];
    var texcoord = (v_texcoords[in_vertex_index]);
    texcoord /= 32.0; // top left corner
    texcoord += f32(tilemap_idx % 32) * vec2<f32>(1.0/32.0, 0.0);
    texcoord += f32(tilemap_idx / 32) * vec2<f32>(0.0, 1.0/32.0); // shifted x and y


    out.clip_position = vec4<f32>(clip_position, 0.0, 1.0);
    out.texcoord = texcoord;
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