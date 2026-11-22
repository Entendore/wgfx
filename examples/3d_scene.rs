use anyhow::Result;
use std::time::Instant;
use wgfx::core::font::{measure_text, push_text};
use wgfx::core::Mesh3D;
use wgfx::math::{Camera3D, Quat, Transform, Vec3, Vec4, AABB};
use wgfx::renderer::PostProcessConfig;
use winit::event::MouseButton;
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;

fn main() -> Result<()> {
    env_logger::init();
    let event_loop = EventLoop::new()?;
    let engine = pollster::block_on(
        wgfx::Engine::builder()
            .with_title("wgfx - 3D Scene")
            .build(&event_loop),
    )?;

    let aspect = engine.config.width as f32 / engine.config.height as f32;
    let mut camera = Camera3D::new(aspect).with_pos(Vec3::new(0.0, 15.0, -25.0));
    let cube_mesh = Mesh3D::cube();
    let sphere_mesh = Mesh3D::sphere(2.0, 32, 16);
    let mut sphere_batch = wgfx::renderer::Batch3D::new(
        &engine.device,
        engine.config.format,
        wgpu::TextureFormat::Depth32Float,
        &engine.checker_texture,
        &sphere_mesh,
    );
    let start = Instant::now();
    let mut last_mouse = (0.0, 0.0);
    let mut wireframe = false;
    let mut post_process_idx: usize = 0;
    let post_process_configs = [
        PostProcessConfig::default(),
        PostProcessConfig {
            grayscale: true,
            vignette_intensity: 0.8,
            ..Default::default()
        },
        PostProcessConfig {
            chromatic_aberration: 0.005,
            vignette_intensity: 0.6,
            ..Default::default()
        },
        PostProcessConfig {
            brightness: 1.3,
            contrast: 1.2,
            vignette_intensity: 0.3,
            ..Default::default()
        },
    ];

    engine.run(event_loop, move |frame| {
        let time = start.elapsed().as_secs_f32();

        if frame.input.is_key_just_pressed(KeyCode::KeyF) {
            wireframe = !wireframe;
        }
        if frame.input.is_key_just_pressed(KeyCode::KeyP) {
            *frame.use_post_processing = !*frame.use_post_processing;
        }
        if frame.input.is_key_just_pressed(KeyCode::KeyC) {
            post_process_idx = (post_process_idx + 1) % post_process_configs.len();
            *frame.post_process_config = post_process_configs[post_process_idx];
        }

        // Camera zoom via scroll
        camera.pos += camera.forward() * frame.input.scroll * 5.0;

        let forward = camera.forward();
        let right = camera.right();
        let speed = 20.0 * frame.ctx.dt;
        if frame.input.is_key_down(KeyCode::KeyW) { camera.pos += forward * speed; }
        if frame.input.is_key_down(KeyCode::KeyS) { camera.pos -= forward * speed; }
        if frame.input.is_key_down(KeyCode::KeyA) { camera.pos -= right * speed; }
        if frame.input.is_key_down(KeyCode::KeyD) { camera.pos += right * speed; }
        if frame.input.is_key_down(KeyCode::Space) { camera.pos += Vec3::Y * speed; }
        if frame.input.is_key_down(KeyCode::ShiftLeft) { camera.pos -= Vec3::Y * speed; }

        if frame.input.is_mouse_down(MouseButton::Right) {
            let dx = frame.input.mouse_pos.x - last_mouse.0;
            let dy = frame.input.mouse_pos.y - last_mouse.1;
            camera.yaw -= dx * 0.005; 
            camera.pitch = (camera.pitch - dy * 0.005).clamp(-1.5, 1.5)
        }
        last_mouse = (frame.input.mouse_pos.x, frame.input.mouse_pos.y);

        frame.renderer.batch3d.reset();
        sphere_batch.reset();
        frame.renderer.line3d.push_grid(10, 5.0, Vec4::new(0.3, 0.3, 0.3, 1.0));
        frame.renderer.line3d.push_axes(Vec3::ZERO, 3.0);

        // Dynamic waving terrain (15x15 grid of cubes)
        for x in -7..8 {
            for z in -7..8 {
                let y = (x as f32 * 0.5 + time).sin() * (z as f32 * 0.5 + time).cos() * 3.0;
                let pos = Vec3::new(x as f32 * 3.0, y, z as f32 * 3.0);
                let model = Transform::from_pos(pos).to_matrix();

                let height_factor = (y + 3.0) / 6.0;
                let color = Vec4::new(height_factor, 0.2, 1.0 - height_factor, 1.0);

                if !wireframe {
                    frame.renderer.batch3d.push(wgfx::renderer::Shape3D::new(model, color));
                } else {
                    frame.renderer.line3d.push_wireframe(&cube_mesh, model, color);
                }
            }
        }

        // 4 Orbiting Spheres
        for i in 0..4 {
            let angle = time * 0.5 + (i as f32 * std::f32::consts::PI / 2.0);
            let radius = 15.0;
            let pos = Vec3::new(
                angle.cos() * radius,
                10.0 + (time + i as f32).sin() * 2.0,
                angle.sin() * radius,
            );
            let model = Transform::from_pos(pos)
                .with_rot(Quat::from_rotation_y(time + i as f32))
                .to_matrix();
            let color = Vec4::new(1.0, 0.8, 0.2, 1.0);

            if !wireframe {
                sphere_batch.push(wgfx::renderer::Shape3D::new(model, color));
            } else {
                frame.renderer.line3d.push_wireframe(&sphere_mesh, model, color);
            }
        }

        // Day/Night cycle light
        let light_angle = time * 0.2;
        let light_dir = Vec3::new(light_angle.cos(), -0.5, light_angle.sin()).normalize();

        let view_proj = camera.build_view_projection();
        if !wireframe {
            frame.renderer.batch3d.flush(frame.ctx, view_proj, light_dir, time);
            sphere_batch.flush(frame.ctx, view_proj, light_dir, time);
        }
        frame.renderer.line3d.flush(frame.ctx, view_proj);

        // Render Text UI
        frame.renderer.batch2d.reset();
        let mode_text = if wireframe { "MODE: WIREFRAME" } else { "MODE: SOLID" };
        let pp_text = if *frame.use_post_processing { "POST: ON" } else { "POST: OFF" };
        push_text(&mut frame.renderer.batch2d, "3D SCENE", 10.0, 10.0, 4.0, Vec4::new(1.0, 1.0, 0.0, 1.0));
        push_text(&mut frame.renderer.batch2d, mode_text, 10.0, 40.0, 2.0, Vec4::new(1.0, 1.0, 1.0, 1.0));
        push_text(&mut frame.renderer.batch2d, pp_text, 10.0, 60.0, 2.0, Vec4::new(1.0, 1.0, 1.0, 1.0));
        push_text(
            &mut frame.renderer.batch2d,
            "[F] WIRE   [P] POST   [C] CYCLE FX   [SCROLL] ZOOM   [RMB] LOOK   [WASD+SPACE/SHIFT] MOVE",
            10.0, 80.0, 2.0, Vec4::new(0.8, 0.8, 0.8, 1.0),
        );
        let fps_text = format!("FPS: {:.0}", frame.ctx.fps);
        push_text(&mut frame.renderer.batch2d, &fps_text, 10.0, 100.0, 2.0, Vec4::new(0.5, 1.0, 0.5, 1.0));

        let ui_cam = wgfx::math::Camera2D::new(
            frame.config.width as f32,
            frame.config.height as f32,
        );
        frame.renderer.batch2d.flush(frame.ctx, ui_cam.build_view_projection());
    })?;
    Ok(())
}
