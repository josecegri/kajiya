use spirv_std::arch::{Derivative, kill};
use spirv_std::glam::{FloatExt, Mat3, Vec2, Vec3, Vec3Swizzles, Vec4, Vec4Swizzles};
use spirv_std::image::Image2d;
#[cfg(target_arch = "spirv")]
use spirv_std::num_traits::Float;
use spirv_std::{RuntimeArray, Sampler, spirv};

use rust_shaders_shared::frame_constants::{FrameConstants, InstanceDynamicConstants};
use rust_shaders_shared::gbuffer::{GbufferData, perceptual_roughness_to_roughness};
use rust_shaders_shared::mesh::{
    MESH_MATERIAL_SIZE, MaterialDescriptor, MeshDescriptor, VertexPacked, load_vec2, load_vec4,
    unpack_vertex,
};
use rust_shaders_shared::raster_simple::{
    InstanceTransform, RasterConstants, transform_point, transform_vector,
};
use rust_shaders_shared::render_overrides::RenderOverrideFlags;

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
        data0: load_vec4(
            vertices,
            (vid * size_of::<Vec4>() as u32 + mesh.vertex_core_offset) as usize >> 2,
        ),
    };
    let v = unpack_vertex(packed);

    let v_color = if mesh.vertex_aux_offset != 0 {
        load_vec4(
            vertices,
            (vid * size_of::<Vec4>() as u32 + mesh.vertex_aux_offset) as usize >> 2,
        )
    } else {
        Vec4::splat(1.0)
    };

    let v_tangent_packed = if mesh.vertex_tangent_offset != 0 {
        load_vec4(
            vertices,
            (vid * size_of::<Vec4>() as u32 + mesh.vertex_tangent_offset) as usize >> 2,
        )
    } else {
        Vec4::new(1.0, 0.0, 0.0, 1.0)
    };

    let uv = load_vec2(
        vertices,
        (vid * size_of::<Vec2>() as u32 + mesh.vertex_uv_offset) as usize >> 2,
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

#[spirv(fragment)]
pub fn raster_simple_fs(
    #[spirv(location = 0)] color: Vec4,
    #[spirv(location = 1)] uv: Vec2,
    #[spirv(location = 2)] normal: Vec3,
    #[spirv(flat, location = 3)] material_id: u32,
    #[spirv(location = 4)] tangent: Vec3,
    #[spirv(location = 5)] bitangent: Vec3,
    #[spirv(location = 6)] vs_pos: Vec3,
    #[spirv(location = 7)] prev_vs_pos: Vec3,

    #[spirv(push_constant)] push_constants: &RasterConstants,

    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)]
    instance_transforms_dyn: &[InstanceTransform],
    #[spirv(descriptor_set = 0, binding = 32)] _sampler_lnc: &Sampler,
    #[spirv(descriptor_set = 0, binding = 33)] sampler_llr: &Sampler,
    #[spirv(descriptor_set = 0, binding = 34)] _sampler_nnc: &Sampler,
    #[spirv(descriptor_set = 0, binding = 35)] _sampler_llc: &Sampler,

    #[spirv(storage_buffer, descriptor_set = 1, binding = 0)] meshes: &[MeshDescriptor],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 1)] vertices: &[u32],
    #[spirv(storage_buffer, descriptor_set = 1, binding = 2)] _bindless_texture_sizes: &[Vec4],
    #[spirv(descriptor_set = 1, binding = 3)] bindless_textures: &RuntimeArray<Image2d>,

    #[spirv(uniform, descriptor_set = 2, binding = 0)] frame_constants: &FrameConstants,
    #[spirv(storage_buffer, descriptor_set = 2, binding = 1)]
    instance_dynamic_parameters_dyn: &[InstanceDynamicConstants],

    #[spirv(location = 0)] out_geometric_normal: &mut Vec3,
    #[spirv(location = 1)] out_gbuffer: &mut Vec4,
    #[spirv(location = 2)] out_velocity: &mut Vec4,
) {
    let mesh = meshes[push_constants.mesh_index as usize];

    let material = MaterialDescriptor::load(
        vertices,
        mesh.mat_data_offset + material_id * MESH_MATERIAL_SIZE,
    );

    const LOD_BIAS: f32 = -0.5;

    let albedo_uv: Vec2 = material.transform_uv(uv, 0);
    let albedo_tex = unsafe { bindless_textures.index(material.maps.albedo()) };
    let albedo_texel = albedo_tex.sample_bias(*sampler_llr, albedo_uv, LOD_BIAS);
    if albedo_texel.w < 0.5 {
        kill();
    }

    let albedo = albedo_texel.xyz() * material.base_color_mult.xyz() * color.xyz();

    let spec_uv = material.transform_uv(uv, 2);
    let spec_tex = unsafe { bindless_textures.index(material.maps.metallic_roughness()) };
    let metalness_roughness = spec_tex.sample_bias(*sampler_llr, spec_uv, LOD_BIAS);
    let perceptual_roughness = material.roughness_mult * metalness_roughness.x;
    let mut roughness = perceptual_roughness_to_roughness(perceptual_roughness).clamp(1e-4, 1.0);
    let mut metalness = metalness_roughness.y * material.metalness_factor;

    if frame_constants
        .render_overrides
        .has_flag(RenderOverrideFlags::NO_METAL)
    {
        metalness = 0.0;
    }

    if frame_constants.render_overrides.material_roughness_scale <= 1.0 {
        roughness *= frame_constants.render_overrides.material_roughness_scale;
    } else {
        let value = roughness.sqrt().lerp(
            1.0,
            1.0 - 1.0 / frame_constants.render_overrides.material_roughness_scale,
        );
        roughness = value * value;
    }

    let mut normal_os = normal;

    if !frame_constants
        .render_overrides
        .has_flag(RenderOverrideFlags::NO_NORMAL_MAPS)
    {
        let normal_tex = unsafe { bindless_textures.index(material.maps.normal()) };

        let mut ts_normal =
            (normal_tex.sample_bias(*sampler_llr, uv, LOD_BIAS).xy() * 2.0 - 1.0).extend(0.0);
        ts_normal.z = 0.01_f32
            .max(1.0 - ts_normal.xy().dot(ts_normal.xy()))
            .sqrt();
        //float3 ts_normal = normal_tex.SampleBias(sampler_llr, ps.uv, lod_bias).xyz * 2.0 - 1.0;

        if frame_constants
            .render_overrides
            .has_flag(RenderOverrideFlags::FLIP_NORMAL_MAP_YZ)
        {
            ts_normal.z *= -1.0;
            ts_normal.y *= -1.0;
        }

        if bitangent.dot(bitangent) > 0.0 {
            let tbn = Mat3::from_cols(tangent, bitangent, normal);
            normal_os = tbn * ts_normal;
        }
    }

    // Transform to world space
    let mut normal_ws = transform_vector(
        &instance_transforms_dyn[push_constants.draw_index as usize].current,
        normal_os,
    )
    .normalize();

    // Derive normal from depth
    let geometric_normal_vs = {
        let d1 = vs_pos.dfdx();
        let d2 = vs_pos.dfdy();
        d2.cross(d1).normalize()
    };
    let geometric_normal_ws = frame_constants.direction_view_to_world(geometric_normal_vs);

    // Fix invalid normals
    if normal_ws.dot(geometric_normal_ws) < 0.0 {
        normal_ws *= -1.0;
    }

    if frame_constants
        .render_overrides
        .has_flag(RenderOverrideFlags::FORCE_FACE_NORMALS)
    {
        normal_ws = geometric_normal_ws;
    }

    let emissive_uv = material.transform_uv(uv, 3);
    let emissive_tex = unsafe { bindless_textures.index(material.maps.emissive()) };
    let emissive = Vec3::splat(1.0)
        * emissive_tex
            .sample_bias(*sampler_llr, emissive_uv, LOD_BIAS)
            .xyz()
        * Vec3::from_array(material.emissive)
        * instance_dynamic_parameters_dyn[push_constants.draw_index as usize].emissive_multiplier
        * frame_constants.pre_exposure;

    //albedo = float3(0.966653, 0.802156, 0.323968); // Au from Mitsuba

    let gbuffer = GbufferData {
        albedo,
        emissive,
        normal: normal_ws,
        roughness,
        metalness,
    };
    //gbuffer.roughness = lerp(0.05, 0.15, roughness);  // kitchen hack

    *out_geometric_normal = geometric_normal_vs * 0.5 + 0.5;
    *out_gbuffer = gbuffer.pack().to_vec4();
    *out_velocity = (prev_vs_pos - vs_pos).extend(0.0);
}
