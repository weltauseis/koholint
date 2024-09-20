const SCREEN_W: f32 = 160.0;
const SCREEN_H: f32 = 144.0;
const OBJ_W_PXL: f32 = 8.0;

const OBJ_W: f32 = 1.0 / (SCREEN_W / OBJ_W_PXL);
const OBJ_H: f32 = 1.0 / (SCREEN_H / OBJ_W_PXL);

var<private> v_positions: array<vec2<f32>, 6> = array<vec2<f32>, 6> (
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(-1.0 + OBJ_W, 1.0 - OBJ_H),
    vec2<f32>(-1.0, 1.0 - OBJ_H),
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(-1.0 + OBJ_W, 1.0),
    vec2<f32>(-1.0 + OBJ_W, 1.0 - OBJ_H),
);

// texture coordinates are flipped on the y axis
var<private> v_texcoords: array<vec2<f32>, 6> = array<vec2<f32>, 6> (
    vec2<f32>(0.0, 0.0),
    vec2<f32>(0.625, 0.5625),
    vec2<f32>(0.0, 0.5625),
    vec2<f32>(0.0, 0.0),
    vec2<f32>(0.625, 0.0),
    vec2<f32>(0.625, 0.5625),
);

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texcoord: vec2<f32>,
}

@group(0) @binding(2)
var<storage, read> x_pos_buffer: array<u32>;
@group(0) @binding(3)
var<storage, read> y_pos_buffer: array<u32>;
@group(0) @binding(4)
var<storage, read> sprite_ids_buffer: array<u32>;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32, @builtin(instance_index) in_instance_index: u32) -> VertexOutput {
    var out: VertexOutput;

    var clip_position = (v_positions[in_vertex_index]);
    clip_position += vec2<f32>(-(8.0/160.0), (16.0/144.0));

    var x_pos = x_pos_buffer[in_instance_index];
    var y_pos = y_pos_buffer[in_instance_index];
    clip_position += vec2<f32>(f32(x_pos) * (1.0 / 160), f32(y_pos) * (1.0 / 144));

    var texcoord = (v_texcoords[in_vertex_index]);

    out.clip_position = vec4<f32>(clip_position, 0.0, 1.0);
    out.texcoord = texcoord;
    return out;
}

@group(0) @binding(0)
var tile_atlas: texture_2d<f32>;
@group(0) @binding(1)
var tile_atlas_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //return vec4<f32>(in.texcoord, 0.0, 1.0);
    //return textureSample(tile_atlas, tile_atlas_sampler, in.texcoord);

    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}