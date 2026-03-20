struct SceneParams {
    projection_matrix: mat4x4<f32>,
    inverse_projection_matrix: mat4x4<f32>,
    viewport_size: vec2<f32>,
    _padding: vec2<f32>,
};
@group(0) @binding(0) var<uniform> scene: SceneParams;

struct PerLineParams {
    colour: vec4<f32>,
    radius: f32,
    _pad_0: f32,
    _pad_1: f32,
    _pad_2: f32,
}
@group(1) @binding(0) var<storage, read> points: array<vec2<f32>>;
@group(1) @binding(1) var<uniform> line: PerLineParams;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) frag_pos: vec2<f32>,
    @location(1) p0: vec2<f32>,
    @location(2) p1: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
    @builtin(instance_index) instance_index: u32
) -> VertexOutput {
    let radius = line.radius;
    var out: VertexOutput;

    let aspect = scene.viewport_size.x / scene.viewport_size.y;

    // 1. Get the start (p0) and end (p1) points of this specific segment
    var p0 = (scene.projection_matrix * vec4<f32>(points[instance_index], 0.0, 1.0)).xy;
    var p1 = (scene.projection_matrix * vec4<f32>(points[instance_index + 1u], 0.0, 1.0)).xy;

    // 2. Multiply X by aspect ratio to enter "Square Space" so our radius math is perfectly circular
    p0.x *= aspect;
    p1.x *= aspect;

    // 3. Create a bounding box that covers the line segment PLUS the radius padding on all sides
    let min_pos = min(p0, p1) - vec2<f32>(radius, radius);
    let max_pos = max(p0, p1) + vec2<f32>(radius, radius);

    // 4. Generate the 6 vertices for the bounding quad
    var quad_pos = array<vec2<f32>, 6>(
        vec2<f32>(min_pos.x, min_pos.y), // Bottom-left
        vec2<f32>(max_pos.x, min_pos.y), // Bottom-right
        vec2<f32>(min_pos.x, max_pos.y), // Top-left
        vec2<f32>(min_pos.x, max_pos.y), // Top-left
        vec2<f32>(max_pos.x, min_pos.y), // Bottom-right
        vec2<f32>(max_pos.x, max_pos.y)  // Top-right
    );
    
    let current_pos = quad_pos[vertex_index];
    
    // Pass the square-space coordinates to the fragment shader
    out.frag_pos = current_pos;
    out.p0 = p0;
    out.p1 = p1;

    // 5. Convert the vertex position back to screen stretch (NDC) before outputting
    var ndc_pos = current_pos;
    ndc_pos.x /= aspect;
    
    out.position = vec4<f32>(ndc_pos, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let radius = line.radius;

    // 1. Calculate the shortest distance from the current pixel to the line segment
    let pa = in.frag_pos - in.p0;
    let ba = in.p1 - in.p0;
    
    var h = 0.0;
    let ba_len_sq = dot(ba, ba);
    
    // Safety check: Prevent division by zero if two points in the array are exactly the same
    if (ba_len_sq > 0.000001) {
        h = clamp(dot(pa, ba) / ba_len_sq, 0.0, 1.0);
    }
    
    let dist = length(pa - ba * h);

    let pixel_size = fwidth(dist);
    let alpha = 1.0 - smoothstep(radius - pixel_size, radius, dist);
    
    if (alpha < 0.01) {
        discard;
    }

    return vec4<f32>(line.colour.rgb, line.colour.a * alpha);
}
