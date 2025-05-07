#import bevy_pbr::forward_io::VertexOutput

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let centered_uv = mesh.uv - vec2<f32>(0.5, 0.5);
    return vec4<f32>(0.0, 0.0, 0.0, 1.0 - length(centered_uv) * 2.0);
}
