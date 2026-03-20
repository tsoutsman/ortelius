struct SceneParams {
    projection_matrix: mat4x4<f32>,
    inverse_projection_matrix: mat4x4<f32>,
    viewport_size: vec2<f32>,
    _padding: vec2<f32>,
};
@group(0) @binding(0) var<uniform> scene: SceneParams;

struct Params {
    colour: vec4<f32>,
    thickness: f32,
    _pad_0: f32,
    _pad_1: f32,
    _pad_2: f32,
}
@group(1) @binding(0) var<storage, read> points: array<vec2<f32>>;
@group(1) @binding(1) var<uniform> params: Params;

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
    var out: VertexOutput;
    let aspect = scene.viewport_size.x / scene.viewport_size.y;

    let radius_sq = params.thickness / scene.viewport_size.y;

    // 1. Get the start (p0) and end (p1) points
    var p0 = (scene.projection_matrix * vec4<f32>(points[instance_index], 0.0, 1.0)).xy;
    var p1 = (scene.projection_matrix * vec4<f32>(points[instance_index + 1u], 0.0, 1.0)).xy;

    // 2. Enter "Square Space"
    p0.x *= aspect;
    p1.x *= aspect;

    // 3. Create the bounding box using the CONVERTED radius
    let min_pos = min(p0, p1) - vec2<f32>(radius_sq, radius_sq);
    let max_pos = max(p0, p1) + vec2<f32>(radius_sq, radius_sq);

    // 4. Generate the 6 vertices
    var quad_pos = array<vec2<f32>, 6>(
        vec2<f32>(min_pos.x, min_pos.y),
        vec2<f32>(max_pos.x, min_pos.y),
        vec2<f32>(min_pos.x, max_pos.y),
        vec2<f32>(min_pos.x, max_pos.y),
        vec2<f32>(max_pos.x, min_pos.y),
        vec2<f32>(max_pos.x, max_pos.y) 
    );
    
    let current_pos = quad_pos[vertex_index];
    
    out.frag_pos = current_pos;
    out.p0 = p0;
    out.p1 = p1;

    // 5. Convert back to NDC
    var ndc_pos = current_pos;
    ndc_pos.x /= aspect;
    
    out.position = vec4<f32>(ndc_pos, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let radius_px = params.thickness / 2;

    let pa = in.frag_pos - in.p0;
    let ba = in.p1 - in.p0;
    
    var h = 0.0;
    let ba_len_sq = dot(ba, ba);
    
    if (ba_len_sq > 0.000001) {
        h = clamp(dot(pa, ba) / ba_len_sq, 0.0, 1.0);
    }
    
    let dist_sq = length(pa - ba * h);

    // 2. Convert to logical pixels
    let dist_px = dist_sq * (scene.viewport_size.y / 2.0);

    let aa_feather = 1.0; 
    let alpha = 1.0 - smoothstep(radius_px - 1, radius_px, dist_px);
    
    if (alpha < 0.01) {
        discard;
    }

    return vec4<f32>(params.colour.rgb, params.colour.a * alpha);
}