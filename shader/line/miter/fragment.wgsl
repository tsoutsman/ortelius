@group(0) @binding(0) var<uniform> colour: vec4<f32>;

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    // return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    return vec4<f32>(0.3, 0.2, 0.1, 1.0);
}
