@group(1) @binding(2) var<uniform> colour: vec4<f32>;

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    // return vec4<f32>(1.0, 0.0, 0.0, 1.0);
    return colour;
}
