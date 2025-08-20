struct SceneParams {
    projection_matrix: mat4x4<f32>,
    xclip_bounds: vec2<f32>,
    yclip_bounds: vec2<f32>,
    viewport_size: vec2<f32>,
    _padding: vec2<f32>,
};

// Data passed from the vertex to the fragment shader
struct Varyings {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) p1: vec2<f32>,
    @location(1) @interpolate(flat) p2: vec2<f32>,
    @location(2) clip_pos: vec2<f32>,
};

@group(0) @binding(0) var<uniform> scene: SceneParams;

@group(1) @binding(0) var<storage, read> points: array<vec2<f32>>;
@group(1) @binding(1) var<uniform> thickness: f32;


// @vertex
// fn vs_main(
//     @builtin(vertex_index) vertex_index: u32
// ) -> Varyings {
//     // Each segment needs 4 vertices to make a quad (2 triangles)
//     let segment_index = vertex_index / 4u;
//     let corner_index = vertex_index % 4u;
// 
//     // Get the two points for this segment
//     let p1_raw = scene.projection_matrix * vec4<f32>(points[segment_index], 0.0, 1.0);
//     let p2_raw = scene.projection_matrix * vec4<f32>(points[segment_index + 1u], 0.0, 1.0);
// 
//     // Determine the normal and extrusion direction
//     let dir = normalize(p2_raw - p1_raw);
//     var normal = vec2<f32>(-dir.y, dir.x);
//     // Correct the normal for non-uniform scaling
//     normal = normal / length(normal);
// 
//     // s is -1 if corner_index is 0 or 2, 1 otherwise
//     // let s = (corner_index % 2u) * 2. - 1.;
// 
//     let half = thickness / 2.0;
// 
//     // Determine the vertex position for the quad
//     var point_pos: vec2<f32>;
//     if (corner_index == 0u) { // Top-left
//         point_pos = p1_raw - normal * half;
//     } else if (corner_index == 1u) { // Bottom-left
//         point_pos = p1_raw + normal * half;
//     } else if (corner_index == 2u) { // Top-right
//         point_pos = p2_raw - normal * half;
//     } else { // Bottom-right
//         point_pos = p2_raw + normal * half;
//     }
//     
//     // Create the final clip-space position
//     let final_pos = point_pos;// * scene.scale + scene.offset;
// 
//     var out: Varyings;
//     out.position = scene.projection_matrix * vec4<f32>(final_pos, 0.0, 1.0);
//     // Pass the UN-SCALED segment points to the fragment shader
//     out.p1 = p1_raw;// * scene.scale + scene.offset;
//     out.p2 = p2_raw;// * scene.scale + scene.offset;
//     out.clip_pos = final_pos;
//     return out;
// }

@vertex
fn vs_main(
    @builtin(vertex_index) vertex_index: u32
) -> Varyings {
    let segment_index = vertex_index / 4u;
    let corner_index = vertex_index % 4u;

    // Get the two points for this segment in data space
    let p1_raw = points[segment_index];
    let p2_raw = points[segment_index + 1u];

    // --- MODIFICATION START ---

    // 1. Transform endpoints into 4D clip space
    let p1_clip = scene.projection_matrix * vec4<f32>(p1_raw, 0.0, 1.0);
    let p2_clip = scene.projection_matrix * vec4<f32>(p2_raw, 0.0, 1.0);

    // 2. Convert to 2D Normalized Device Coordinates (NDC) by perspective divide
    // This gives us a uniform coordinate system from -1.0 to 1.0.
    let p1_ndc = p1_clip.xy / p1_clip.w;
    let p2_ndc = p2_clip.xy / p2_clip.w;

    // 3. Calculate direction and normal IN SCREEN-RELATIVE SPACE
    // We multiply by viewport_size to correct for aspect ratio before normalizing.
    let dir_screen = normalize((p2_ndc - p1_ndc) * scene.viewport_size);
    let normal_screen = vec2<f32>(-dir_screen.y, dir_screen.x);

    // 4. Calculate the thickness offset in NDC units
    let thickness_pixels = 5.;
    let half_thickness_ndc = (normal_screen * thickness_pixels) / scene.viewport_size;

    // 5. Determine which point to extrude and in which direction
    let base_ndc = select(p2_ndc, p1_ndc, corner_index < 2u); // p1 for corners 0,1; p2 for 2,3
    let extrusion_sign = select(1.0, -1.0, (corner_index & 1u) == 0u); // -1 for top, 1 for bottom

    let final_ndc = base_ndc + extrusion_sign * half_thickness_ndc;

    // 6. Reconstruct the final 4D clip space position
    // Use the z and w from the original transformed point to maintain depth.
    let base_clip = select(p2_clip, p1_clip, corner_index < 2u);
    let final_clip_pos = vec4<f32>(final_ndc * base_clip.w, base_clip.z, base_clip.w);
    
    // --- MODIFICATION END ---

    var out: Varyings;
    out.position = final_clip_pos;
    
    // Pass original data-space points to fragment shader as before
    out.p1 = p1_raw;
    out.p2 = p2_raw;
    // Pass the final data_pos for this vertex (calculated via inverse projection if needed,
    // or just leave it as is if the fragment shader doesn't need it transformed)
    out.clip_pos = vec2(0.0); // Or calculate appropriately if needed
    
    return out;
}

// This function calculates the shortest distance from a point `p`
// to a line segment defined by `a` and `b`.
fn distance_to_segment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let ab = b - a;
    let ap = p - a;
    let t = saturate(dot(ap, ab) / dot(ab, ab));
    let projected_point = a + t * ab;
    return distance(p, projected_point);
}

@fragment
fn fs_main(in: Varyings) -> @location(0) vec4<f32> {
    // let aspect_ratio = scene.viewport_size.x / scene.viewport_size.y;
    // let aspect_ratio = 8./6.;

    // // Create a new, uniform coordinate system for distance measurement
    // var p_screen = in.clip_pos;
    // p_screen.x *= aspect_ratio;

    // var p1_screen = in.p1;
    // p1_screen.x *= aspect_ratio;
    // 
    // var p2_screen = in.p2;
    // p2_screen.x *= aspect_ratio;

    // // Calculate distance from the fragment to the line segment
    // let d = distance_to_segment(p_screen, p1_screen, p2_screen);

    // // Calculate line radius in clip space
    // // NOTE: This assumes a square aspect ratio. For non-square,
    // // this calculation needs to be adjusted by the aspect ratio.
    // let radius = thickness / 2.0;

    // // Smooth the edge using the pixel derivative (for anti-aliasing)
    // let smooth_width = fwidth(d);
    // let alpha = 1.0 - smoothstep(radius - smooth_width, radius, d);

    // if (alpha <= 0.0) {
    //     //discard;
    // }

    // Return your line color with the calculated alpha for a smooth edge
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);//alpha);
}
