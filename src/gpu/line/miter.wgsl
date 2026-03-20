struct SceneParams {
    projection_matrix: mat4x4<f32>,
    inverse_projection_matrix: mat4x4<f32>,
    viewport_size: vec2<f32>,
    _padding: vec2<f32>,
};

@group(0) @binding(0) var<uniform> scene: SceneParams;

@group(1) @binding(0) var<storage, read> points: array<vec2<f32>>;
@group(1) @binding(1) var<uniform> thickness: f32;
@group(1) @binding(2) var<uniform> colour: vec4<f32>;

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
    let p_prev = (scene.projection_matrix * vec4<f32>(points[max(0, i32(point_index) - 1)], 0.0, 1.0)).xy;
    let p_curr = (scene.projection_matrix * vec4<f32>(points[point_index], 0.0, 1.0)).xy;
    let p_next = (scene.projection_matrix * vec4<f32>(points[min(arrayLength(&points) - 1u, point_index + 1u)], 0.0, 1.0)).xy;

    // // Calculate direction vectors and their normals
    // let dir_in = normalize(p_curr - p_prev);
    // let dir_out = normalize(p_next - p_curr);
    // let normal_in = vec2<f32>(-dir_in.y, dir_in.x);
    // let normal_out = vec2<f32>(-dir_out.y, dir_out.x);

    // Calculate raw direction vectors
    var raw_dir_in = p_curr - p_prev;
    var raw_dir_out = p_next - p_curr;

    // 1. Fix the endpoints: copy the valid direction so the caps are square
    if (point_index == 0u) {
        raw_dir_in = raw_dir_out; 
    }
    if (point_index >= arrayLength(&points) - 1u) {
        raw_dir_out = raw_dir_in; 
    }

    // 2. Safely normalize to prevent crashes from duplicate points
    var dir_in = raw_dir_in;
    if (length(dir_in) > 0.0001) { 
        dir_in = normalize(dir_in); 
    } else { 
        dir_in = vec2<f32>(1.0, 0.0);
    }

    var dir_out = raw_dir_out;
    if (length(dir_out) > 0.0001) { 
        dir_out = normalize(dir_out); 
    } else { 
        dir_out = vec2<f32>(1.0, 0.0); 
    }

    let normal_in = vec2<f32>(-dir_in.y, dir_in.x);
    let normal_out = vec2<f32>(-dir_out.y, dir_out.x);

    // Calculate the miter vector, which bisects the angle between the two segments
    let miter_vec = normalize(normal_in + normal_out);

    // Calculate the miter length to prevent the line from getting thicker at sharp angles
    // let miter_len = 1.0 / dot(miter_vec, normal_in);
    let miter_len = min(1.0 / dot(miter_vec, normal_in), 2.5);


    // Calculate the final position by extruding the current point along the miter vector
    let pos = p_curr + miter_vec * side * half_width * miter_len;
    // var pos = points[point_index];// + 0.3 * side;
    // pos[0] = pos[0] + 0.3 * side;

    return vec4<f32>(pos, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return colour;
}
