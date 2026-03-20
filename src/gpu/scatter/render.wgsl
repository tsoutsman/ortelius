struct SceneParams {
    projection_matrix: mat4x4<f32>,
    inverse_projection_matrix: mat4x4<f32>,
    viewport_size: vec2<f32>,
    _padding: vec2<f32>,
};
@group(0) @binding(0) var<uniform> scene: SceneParams;

struct Params {
    colour: vec4<f32>,
    radius: f32,
    _pad_0: f32,
    _pad_1: f32,
    _pad_2: f32,
}
@group(1) @binding(0) var<storage, read> points: array<vec2<f32>>;
@group(1) @binding(1) var<uniform> scatter: Params;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local_pos: vec2<f32>, // Passes the -1 to 1 quad coordinates to the fragment shader
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32
) -> VertexOutput {
    var out: VertexOutput;

    let ndc_center = scene.projection_matrix * vec4<f32>(points[instance_index], 0.0, 1.0);

    var quad_pos = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0,  1.0),
        vec2<f32>( 1.0, -1.0),
        vec2<f32>( 1.0,  1.0) 
    );
    
    let local_coord = quad_pos[vertex_index];
    out.local_pos = local_coord;

    let ndc_radius = (scatter.radius / scene.viewport_size) * 2.0;

    let offset = local_coord * ndc_radius;
    out.position = ndc_center + vec4<f32>(offset, 0.0, 0.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = length(in.local_pos);

    let pixel_size = fwidth(dist);
    let alpha = 1.0 - smoothstep(1.0 - pixel_size, 1.0, dist);
    
    if (alpha < 0.01) {
        discard;
    }

    return vec4<f32>(scatter.colour.rgb, scatter.colour.a * alpha);
}