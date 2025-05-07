#import bevy_pbr::forward_io::VertexOutput

@group(2) @binding(0) var<uniform> material_color: vec4<f32>;
@group(2) @binding(1) var<uniform> thickness: f32;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    if mesh.uv.x < thickness || mesh.uv.y < thickness || mesh.uv.x > 1.0 - thickness || mesh.uv.y > 1.0 - thickness {
        return material_color;
    }
    return vec4<f32>(0.0, 0.0, 0.0, 0.0);
}
