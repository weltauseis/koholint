struct VertexOut {
    @location(0) texcoords: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

var<private> v_positions: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(1.0, -1.0),
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(-1.0, 1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(1.0, -1.0),
);

var<private> v_texcoords: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2<f32>(0, 1.0),
    vec2<f32>(1.0, 0.0),
    vec2<f32>(0.0, 0.0),
    vec2<f32>(0.0, 1.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(1.0, 0.0),
);

@group(0) @binding(0) var ourSampler: sampler;
@group(0) @binding(1) var tile_atlas: texture_2d<f32>;

@vertex
fn vs_main(@builtin(vertex_index) v_idx: u32) -> VertexOut {
    var out: VertexOut;

    out.position = vec4<f32>(v_positions[v_idx], 0.0, 1.0);
    out.texcoords = v_texcoords[v_idx];

    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    //return vec4f(in.texcoords, 0, 1);
    return textureSample(tile_atlas, ourSampler, in.texcoords);
    //return textureSample(tilemap, ourSampler, in.texcoords);
}