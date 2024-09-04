struct VertexOut {
    @location(0) texcoords: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

struct Uniforms {
    @size(16) angle: f32, // pad to 16 bytes
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

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

@vertex
fn vs_main(@builtin(vertex_index) v_idx: u32) -> VertexOut {
    var out: VertexOut;

    out.position = vec4<f32>(v_positions[v_idx], 0.0, 1.0);
    out.position.x = out.position.x * cos(uniforms.angle);
    out.texcoords = v_texcoords[v_idx];

    return out;
}

@group(0) @binding(1) var ourSampler: sampler;
@group(0) @binding(2) var ourTexture: texture_2d<f32>;

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    //return vec4f(in.texcoords, 0, 1);
    return textureSample(ourTexture, ourSampler, in.texcoords);
}