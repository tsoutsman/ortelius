@group(0) @binding(0) var<uniform> colour: vec4<f32>;

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return colour;
}
