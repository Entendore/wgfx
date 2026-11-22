use anyhow::Result;
use std::time::Instant;
use wgfx::core::font::push_text;
use wgfx::math::{Camera2D, Vec2, Vec4};
use wgfx::renderer::ParticleSystem;
use winit::event::MouseButton;
use winit::event_loop::EventLoop;

fn main() -> Result<()> {
    env_logger::init();
    let event_loop = EventLoop::new()?;
    let engine = pollster::block_on(
        wgfx::Engine::builder()
            .with_title("wgfx - GPU Fireworks")
            .build(&event_loop),
    )?;
    let w = engine.config.width as f32;
    let h = engine.config.height as f32;
    let camera = Camera2D::new(w, h).with_pos(Vec2::new(w / 2.0, h / 2.0));
    let mut particles = ParticleSystem::new(
        &engine.device,
        engine.config.format,
        wgpu::TextureFormat::Depth32Float,
        100_000,
    );
    let start = Instant::now();
    let mut last_emit = 0.0;

    engine.run(event_loop, move |frame| {
        let time = start.elapsed().as_secs_f32();
        let mvp = camera.build_view_projection();

        // Auto-emit bursts
        if time - last_emit > 0.5 {
            let pos = Vec2::new(
                w / 2.0 + (time * 2.0).sin() * 300.0,
                h / 2.0 + (time * 3.0).cos() * 200.0,
            );
            let color = Vec4::new(
                0.5 + (time).sin() * 0.5,
                0.5 + (time * 1.3).cos() * 0.5,
                0.5 + (time * 0.7).sin() * 0.5,
                1.0,
            );
            particles.emit_burst(frame.ctx.queue, pos, 500, (50.0, 250.0), (1.0, 3.0), color);
            last_emit = time;
        }

        // Click to emit bursts
        let mouse_world = camera.screen_to_world(frame.input.mouse_pos);
        if frame.input.is_mouse_just_pressed(MouseButton::Left) {
            particles.emit_burst(
                frame.ctx.queue,
                mouse_world,
                1000,
                (80.0, 300.0),
                (0.5, 2.0),
                Vec4::new(1.0, 0.8, 0.2, 1.0),
            );
        }
        if frame.input.is_mouse_down(MouseButton::Right) {
            particles.emit_burst(
                frame.ctx.queue,
                mouse_world,
                50,
                (20.0, 100.0),
                (2.0, 4.0),
                Vec4::new(0.2, 0.8, 1.0, 1.0),
            );
        }

        // Update particles via compute pass (submits separately to avoid encoder borrow conflict)
        particles.update(frame.ctx.device, frame.ctx.queue, frame.ctx.dt, mvp);
        particles.flush(&mut *frame.ctx);

        frame.renderer.batch2d.reset();
        push_text(
            &mut frame.renderer.batch2d,
            "GPU COMPUTE FIREWORKS",
            10.0,
            10.0,
            4.0,
            Vec4::new(1.0, 0.5, 0.2, 1.0),
        );
        push_text(
            &mut frame.renderer.batch2d,
            "LMB: BURST  RMB: STREAM",
            10.0,
            40.0,
            2.0,
            Vec4::new(1.0, 1.0, 1.0, 1.0),
        );
        let stats = format!(
            "FPS: {:.0} | Particles: {}/{}",
            frame.ctx.fps,
            particles.count(),
            particles.capacity()
        );
        push_text(
            &mut frame.renderer.batch2d,
            &stats,
            10.0,
            60.0,
            2.0,
            Vec4::new(0.5, 1.0, 0.5, 1.0),
        );

        frame.renderer.batch2d.flush(&mut *frame.ctx, mvp);
    })?;
    Ok(())
}
