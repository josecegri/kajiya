use kajiya::world_renderer::AddMeshOptions;
use kajiya_simple::*;

fn main() -> anyhow::Result<()> {
    let mut kajiya = SimpleMainLoop::builder().resolution([1600, 900]).build(
        WindowAttributes::default()
            .with_position(PhysicalPosition::new(50, 50))
            .with_title("hello-kajiya")
            .with_resizable(false),
    )?;

    let camera = (
        Vec3::new(0.0, 1.0, 2.5),
        Quat::from_rotation_x(-18.0f32.to_radians()),
    );

    let lens = CameraLens {
        aspect_ratio: kajiya.window_aspect_ratio(),
        ..Default::default()
    };

    let car_mesh = kajiya
        .world_renderer
        .add_baked_mesh("/cache/f8044ed9f35d18a0.mesh", AddMeshOptions::new())?;

    let car_inst = kajiya.world_renderer.add_instance(
        car_mesh,
        Affine3A::from_rotation_translation(Quat::IDENTITY, Vec3::ZERO),
    );

    let mut car_rot = 0.0f32;

    kajiya.run(move |ctx| {
        car_rot += 0.5 * ctx.dt_filtered;
        ctx.world_renderer.set_instance_transform(
            car_inst,
            Affine3A::from_rotation_translation(Quat::from_rotation_y(car_rot), Vec3::ZERO),
        );

        WorldFrameDesc {
            camera_matrices: camera.through(&lens),
            render_extent: ctx.render_extent,
            sun_direction: Vec3::new(4.0, 1.0, 1.0).normalize(),
        }
    })
}
