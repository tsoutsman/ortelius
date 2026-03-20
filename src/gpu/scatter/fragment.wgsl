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
    let dot_color = vec3<f32>(1.0, 1.0, 1.0);
    return vec4<f32>(dot_color, alpha);
}
