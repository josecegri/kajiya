use kajiya_backend::{ash::vk, vulkan::image::*};
use kajiya_rg::{self as rg};
use rg::{RenderGraph, SimpleRenderPass};

const USE_RUST_SHADERS: bool = false;

pub fn motion_blur(
    rg: &mut RenderGraph,
    input: &rg::Handle<Image>,
    depth: &rg::Handle<Image>,
    reprojection_map: &rg::Handle<Image>,
) -> rg::Handle<Image> {
    const VELOCITY_TILE_SIZE: u32 = 16;

    let mut velocity_reduced_x = rg.create(
        reprojection_map
            .desc()
            .div_up_extent([VELOCITY_TILE_SIZE, 1, 1])
            .format(vk::Format::R16G16_SFLOAT),
    );

    if USE_RUST_SHADERS {
        SimpleRenderPass::new_compute_rust(
            rg.add_pass("velocity reduce x"),
            "motion_blur::velocity_reduce_x",
        )
        .read(reprojection_map)
        .write(&mut velocity_reduced_x)
        .dispatch(velocity_reduced_x.desc().extent);
    } else {
        SimpleRenderPass::new_compute(
            rg.add_pass("velocity reduce x"),
            "/shaders/motion_blur/velocity_reduce_x.hlsl",
        )
        .read(reprojection_map)
        .write(&mut velocity_reduced_x)
        .dispatch(velocity_reduced_x.desc().extent);
    }

    let mut velocity_reduced_y =
        rg.create(
            velocity_reduced_x
                .desc()
                .div_up_extent([1, VELOCITY_TILE_SIZE, 1]),
        );

    if USE_RUST_SHADERS {
        SimpleRenderPass::new_compute_rust(
            rg.add_pass("velocity reduce y"),
            "motion_blur::velocity_reduce_y",
        )
        .read(&velocity_reduced_x)
        .write(&mut velocity_reduced_y)
        .dispatch(velocity_reduced_x.desc().extent);
    } else {
        SimpleRenderPass::new_compute(
            rg.add_pass("velocity reduce y"),
            "/shaders/motion_blur/velocity_reduce_y.hlsl",
        )
        .read(&velocity_reduced_x)
        .write(&mut velocity_reduced_y)
        .dispatch(velocity_reduced_x.desc().extent);
    }

    let mut velocity_dilated = rg.create(*velocity_reduced_y.desc());

    if USE_RUST_SHADERS {
        SimpleRenderPass::new_compute_rust(
            rg.add_pass("velocity dilate"),
            "motion_blur::velocity_dilate",
        )
        .read(&velocity_reduced_y)
        .write(&mut velocity_dilated)
        .dispatch(velocity_dilated.desc().extent);
    } else {
        SimpleRenderPass::new_compute(
            rg.add_pass("velocity reduce y"),
            "/shaders/motion_blur/velocity_dilate.hlsl",
        )
        .read(&velocity_reduced_y)
        .write(&mut velocity_dilated)
        .dispatch(velocity_dilated.desc().extent);
    }

    let mut output = rg.create(*input.desc());

    // TODO: account for framerate like the HLSL version did
    let motion_blur_scale: f32 = 1.0;

    if USE_RUST_SHADERS {
        SimpleRenderPass::new_compute_rust(rg.add_pass("motion blur"), "motion_blur::motion_blur")
            .read(input)
            .read(reprojection_map)
            .read(&velocity_dilated)
            .read_aspect(depth, vk::ImageAspectFlags::DEPTH)
            .write(&mut output)
            .constants((
                depth.desc().extent_inv_extent_2d(),
                output.desc().extent_inv_extent_2d(),
                motion_blur_scale,
            ))
            .dispatch(output.desc().extent);
    } else {
        // TODO: the hlsl and rust implementations differ a bit, possibly only in the motion_blur_scale const
        SimpleRenderPass::new_compute(
            rg.add_pass("motion blur"),
            "/shaders/motion_blur/motion_blur.hlsl",
        )
        .read(input)
        .read(reprojection_map)
        .read(&velocity_dilated)
        .read_aspect(depth, vk::ImageAspectFlags::DEPTH)
        .write(&mut output)
        .constants((
            depth.desc().extent_inv_extent_2d(),
            output.desc().extent_inv_extent_2d(),
            //motion_blur_scale,
        ))
        .dispatch(output.desc().extent);
    }

    output
}
