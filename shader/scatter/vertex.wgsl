struct SceneParams {
    projection_matrix: mat4x4<f32>,
    xclip_bounds: vec2<f32>,
    yclip_bounds: vec2<f32>,
    viewport_size: vec2<f32>,
    _padding: vec2<f32>,
};

@group(0) @binding(0) var<uniform> scene: SceneParams;

@group(1) @binding(0) var<storage, read> points: array<vec2<f32>>;
@group(1) @binding(1) var<uniform> radius: f32;

@group(1) @binding(2) var<uniform> colour: vec4<f32>;

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

    // 1. Get the center point in NDC space (-1 to 1)
    let center = (scene.projection_matrix * vec4<f32>(points[instance_index], 0.0, 1.0)).xy;

    // 2. Generate the square quad
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

    // 3. Calculate the aspect ratio from your viewport uniform
    let aspect_ratio = scene.viewport_size.x / scene.viewport_size.y;

    // 4. Calculate the base offset
    var offset = local_coord * radius;

    // 5. Correct the X offset to counter the screen stretch
    offset.x = offset.x / aspect_ratio;

    // 6. Add the corrected offset to the NDC center
    let final_pos = center + offset;

    out.position = vec4<f32>(final_pos, 0.0, 1.0);
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 1. Calculate the distance from the center of the quad (0, 0)
    let dist = length(in.local_pos);

    // 2. Discard any pixels outside the radius of 1.0
    // Using smoothstep gives the dot nicely anti-aliased (soft) edges
    let alpha = 1.0 - smoothstep(0.95, 1.0, dist);
    
    if (alpha < 0.01) {
        discard; // Throw away the corners of the square
    }

    // 3. Draw the dot (e.g., solid white)
    return colour;
}
