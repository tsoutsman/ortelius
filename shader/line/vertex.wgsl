struct SceneParams {
    scale: vec2<f32>,
    offset: vec2<f32>,
};

@group(0) @binding(0) var<uniform> scene: SceneParams;

@group(1) @binding(0) var<storage, read> points: array<vec2<f32>>;
@group(1) @binding(1) var<uniform> thickness: f32;

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32,
) -> @builtin(position) vec4<f32> {
    let half_width = thickness / 2.0;

    // Each point in our list becomes two vertices in the strip (a top and bottom one)
    let point_index = vertex_index / 2u;
    // Determine if we are creating the top (1.0) or bottom (-1.0) vertex
    let side = f32(vertex_index % 2u) * 2.0 - 1.0;

    // Get the current, previous, and next points to determine the angle of the join.
    // Clamp indices to handle the start and end of the line gracefully.
    let p_prev = points[max(0, i32(point_index) - 1)];// * scene.scale + scene.offset;
    let p_curr = points[point_index];// * scene.scale + scene.offset;
    let p_next = points[min(arrayLength(&points) - 1u, point_index + 1u)];// * scene.scale + scene.offset;

    // Calculate direction vectors and their normals
    let dir_in = normalize(p_curr - p_prev);
    let dir_out = normalize(p_next - p_curr);
    let normal_in = vec2<f32>(-dir_in.y, dir_in.x);
    let normal_out = vec2<f32>(-dir_out.y, dir_out.x);

    // Calculate the miter vector, which bisects the angle between the two segments
    let miter_vec = normalize(normal_in + normal_out);

    // Calculate the miter length to prevent the line from getting thicker at sharp angles
    let miter_len = 1.0 / dot(miter_vec, normal_in);
    // let miter_len = min(1.0 / dot(miter_vec, normal_in), 2.5); // Clamp the miter length


    // Calculate the final position by extruding the current point along the miter vector
    let pos = p_curr + miter_vec * side * half_width * miter_len;
    // var pos = points[point_index];// + 0.3 * side;
    // pos[0] = pos[0] + 0.3 * side;

    return vec4<f32>(pos, 0.0, 1.0);
}
