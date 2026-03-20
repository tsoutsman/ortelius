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

    // 1. Generate the full-screen NDC quad
    var quad_pos = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), vec2<f32>( 1.0, -1.0), vec2<f32>(-1.0,  1.0),
        vec2<f32>(-1.0,  1.0), vec2<f32>( 1.0, -1.0), vec2<f32>( 1.0,  1.0)
    );
    let ndc = quad_pos[vertex_index];
    
    // Output the vertex to the screen
    out.position = vec4<f32>(ndc, 0.0, 1.0);

    // 2. Un-project the NDC coordinate back into World Space
    // We multiply the inverse matrix by the NDC coordinate
    let unprojected = scene.inverse_projection_matrix * vec4<f32>(ndc, 0.0, 1.0);

    // 3. The "Perspective Divide"
    // When doing matrix multiplication like this, we must divide by 'w' 
    // to complete the transformation (even if you are just doing 2D math!)
    out.world_pos = unprojected.xy / unprojected.w;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let world_pos = in.world_pos;
    let pixel_size = fwidth(world_pos); 

    let grid_spacing = params.spacing; 
    let grid_thickness = params.thickness; 
    let axis_thickness = params.axis_thickness; 

    let bg_color = vec4<f32>(0.98, 0.98, 0.98, 1.0);
    let grid_color = vec4<f32>(0.85, 0.85, 0.85, 1.0);
    let axis_color = vec4<f32>(0.2, 0.2, 0.2, 1.0);

    // 1. Calculate Standard Grid Lines
    // Because world_pos and grid_spacing are BOTH vec2, WGSL applies this math 
    // independently to the X and Y axes all at once. No extra code needed!
    let coord = world_pos / grid_spacing;
    let grid_dist = abs(fract(coord + 0.5) - 0.5) * grid_spacing;
    
    let grid_half_width = (grid_thickness * pixel_size) / 2.0;
    
    let grid_alpha_x = 1.0 - smoothstep(grid_half_width.x - pixel_size.x, grid_half_width.x + pixel_size.x, grid_dist.x);
    let grid_alpha_y = 1.0 - smoothstep(grid_half_width.y - pixel_size.y, grid_half_width.y + pixel_size.y, grid_dist.y);
    let grid_alpha = max(grid_alpha_x, grid_alpha_y);

    // 2. Calculate Main Axes (X=0 and Y=0)
    let axis_half_width = (axis_thickness * pixel_size) / 2.0;
    let axis_dist = abs(world_pos); 
    
    let axis_alpha_x = 1.0 - smoothstep(axis_half_width.x - pixel_size.x, axis_half_width.x + pixel_size.x, axis_dist.x);
    let axis_alpha_y = 1.0 - smoothstep(axis_half_width.y - pixel_size.y, axis_half_width.y + pixel_size.y, axis_dist.y);
    let axis_alpha = max(axis_alpha_x, axis_alpha_y);

    // 3. Blend it all together
    var final_color = mix(bg_color, grid_color, grid_alpha);
    final_color = mix(final_color, axis_color, axis_alpha);

    return final_color;
}