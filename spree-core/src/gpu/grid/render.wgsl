struct SceneParams {
    projection_matrix: mat4x4<f32>,
    inverse_projection_matrix: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> scene: SceneParams;

struct Params {
    spacing: vec2<f32>,
    thickness: f32,
    axis_thickness: f32,
};
@group(1) @binding(0) var<uniform> params: Params;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_pos: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    var quad_pos = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), vec2<f32>( 1.0, -1.0), vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0,  1.0), vec2<f32>( 1.0, -1.0), vec2<f32>( 1.0,  1.0)
    );
    let ndc = quad_pos[vertex_index];
    
    out.position = vec4<f32>(ndc, 0.0, 1.0);

    let unprojected = scene.inverse_projection_matrix * vec4<f32>(ndc, 0.0, 1.0);
    out.world_pos = unprojected.xy / unprojected.w;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let world_pos = in.world_pos;
    let pixel_size = fwidth(world_pos); 

    let axis_thickness = params.axis_thickness; 

    let bg_color = vec4<f32>(0.98, 0.98, 0.98, 1.0);
    let grid_color = vec4<f32>(0.85, 0.85, 0.85, 1.0);
    let axis_color = vec4<f32>(0.2, 0.2, 0.2, 1.0);

    // Grid lines
    // The shortest distance from the current fragment to the nearest grid line in physical pixels.
    let grid_dist = abs(fract(world_pos / params.spacing + 0.5) - 0.5) * params.spacing / pixel_size;
    let grid_half_width = vec2<f32>(params.thickness / 2.);

    let grid_alphas = 1. - smoothstep(grid_half_width - 0.5, grid_half_width + 0.5, grid_dist);
    let grid_alpha = max(grid_alphas.x, grid_alphas.y);
    
    // Axes
    // The shortest distance from the current fragment to the nearest axis line in physical pixels.
    let axis_dist = abs(world_pos) / pixel_size; 
    let axis_half_width = vec2<f32>(params.axis_thickness / 2.);
    
    let axis_alphas = 1. - smoothstep(axis_half_width - 0.5, axis_half_width + 0.5, axis_dist);
    let axis_alpha = max(axis_alphas.x, axis_alphas.y);

    // Combine
    var final_color = mix(bg_color, grid_color, grid_alpha);
    return mix(final_color, axis_color, axis_alpha);
}