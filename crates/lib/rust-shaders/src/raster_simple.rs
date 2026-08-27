use spirv_std::glam::{Vec2, Vec3, Vec4, Vec4Swizzles};
use spirv_std::spirv;

use rust_shaders_shared::frame_constants::FrameConstants;
use rust_shaders_shared::mesh::{MeshDescriptor, VertexPacked, unpack_vertex};
use rust_shaders_shared::raster_simple::{InstanceTransform, RasterConstants, transform_point};

#[spirv(vertex)]
pub fn raster_simple_vs(
    #[spirv(vertex_index)] vid: u32,
    #[spirv(instance_index)] _instance_index: u32,

    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)]
    instance_transforms_dyn: &[InstanceTransform],

    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] meshes: &[MeshDescriptor],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)] vertices: &[u32],

    #[spirv(uniform, descriptor_set = 2, binding = 0)] frame_constants: &FrameConstants,

    #[spirv(push_constant)] push_constants: &RasterConstants,

    #[spirv(position)] out_position: &mut Vec4,
    #[spirv(location = 0)] out_color: &mut Vec4,
    #[spirv(location = 1)] out_uv: &mut Vec2,
    #[spirv(location = 2)] out_normal: &mut Vec3,
    #[spirv(flat, location = 3)] out_material_id: &mut u32,
    #[spirv(location = 4)] out_tangent: &mut Vec3,
    #[spirv(location = 5)] out_bitangent: &mut Vec3,
    #[spirv(location = 6)] out_vs_pos: &mut Vec3,
    #[spirv(location = 7)] out_prev_vs_pos: &mut Vec3,
) {
    let mesh = meshes[push_constants.mesh_index as usize];

    let packed = VertexPacked {
        data0: load4(
            vertices,
            vid * size_of::<Vec4>() as u32 + mesh.vertex_core_offset,
        ),
    };
    let v = unpack_vertex(packed);

    let v_color = if mesh.vertex_aux_offset != 0 {
        load4(
            vertices,
            vid * size_of::<Vec4>() as u32 + mesh.vertex_aux_offset,
        )
    } else {
        Vec4::splat(1.0)
    };

    let v_tangent_packed = if mesh.vertex_tangent_offset != 0 {
        load4(
            vertices,
            vid * size_of::<Vec4>() as u32 + mesh.vertex_tangent_offset,
        )
    } else {
        Vec4::new(1.0, 0.0, 0.0, 1.0)
    };

    let uv = load2(
        vertices,
        vid * size_of::<Vec2>() as u32 + mesh.vertex_uv_offset,
    );

    let material_offset = (vid * size_of::<u32>() as u32 + mesh.vertex_mat_offset) / 4;
    let material_id = vertices[material_offset as usize];

    let instance = &instance_transforms_dyn[push_constants.draw_index as usize];

    //float3 ws_pos = v.position + float3(push_constants.instance_position);
    let ws_pos = transform_point(&instance.current, v.position);

    let vs_pos = frame_constants.view_constants.world_to_view * ws_pos.extend(1.0);
    let cs_pos = frame_constants.view_constants.view_to_sample * vs_pos;

    let prev_ws_pos = transform_point(&instance.previous, v.position);
    let prev_vs_pos = frame_constants.view_constants.world_to_view * prev_ws_pos.extend(1.0);
    //float4 prev_cs_pos = mul(frame_constants.view_constants.view_to_sample, prev_vs_pos);

    let tangent = v_tangent_packed.xyz();
    let bitangent = (v.normal.cross(tangent) * v_tangent_packed.w).normalize();

    *out_position = cs_pos;
    *out_color = v_color;
    *out_uv = uv;
    *out_normal = v.normal;
    *out_material_id = material_id;
    *out_tangent = tangent;
    *out_bitangent = bitangent;

    *out_vs_pos = vs_pos.xyz() / vs_pos.w;
    *out_prev_vs_pos = prev_vs_pos.xyz() / prev_vs_pos.w;
}

// TODO: if sensible, consider instead using spirv_std::ByteAddressableBuffer for accessing the vertex buffer data
#[inline]
fn load4(vertices: &[u32], byte_offset: u32) -> Vec4 {
    let offset = (byte_offset / 4) as usize;

    Vec4::new(
        f32::from_bits(vertices[offset]),
        f32::from_bits(vertices[offset + 1]),
        f32::from_bits(vertices[offset + 2]),
        f32::from_bits(vertices[offset + 3]),
    )
}

#[inline]
fn load2(vertices: &[u32], byte_offset: u32) -> Vec2 {
    let offset = (byte_offset / 4) as usize;

    Vec2::new(
        f32::from_bits(vertices[offset]),
        f32::from_bits(vertices[offset + 1]),
    )
}
