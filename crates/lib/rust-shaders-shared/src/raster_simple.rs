#[cfg(not(target_arch = "spirv"))]
use glam::{Vec3, Vec4};
#[cfg(target_arch = "spirv")]
use spirv_std::glam::{Vec3, Vec4};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RasterConstants {
    pub draw_index: u32,
    pub mesh_index: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct InstanceTransform {
    pub current: [Vec4; 3],
    pub previous: [Vec4; 3],
}

pub fn transform_point(m: &[Vec4; 3], p: Vec3) -> Vec3 {
    Vec3::new(
        m[0].dot(p.extend(1.0)),
        m[1].dot(p.extend(1.0)),
        m[2].dot(p.extend(1.0)),
    )
}

pub fn transform_vector(m: &[Vec4; 3], v: Vec3) -> Vec3 {
    Vec3::new(
        m[0].dot(v.extend(0.0)),
        m[1].dot(v.extend(0.0)),
        m[2].dot(v.extend(0.0)),
    )
}
