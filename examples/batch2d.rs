use anyhow::Result;
use std::time::Instant;
use wgfx::core::font::push_text;
use wgfx::math::{Camera2D, Vec2, Vec4};
use wgfx::renderer::{Shape2D, ShapeKind};
use winit::event_loop::EventLoop;

fn main() -> Result<()> {
    env_logger::init();
    let event_loop = EventLoop::new()?;
    let engine = pollster::block_on(
        wgfx::Engine::builder()
            .with_title("wgfx - 2D Vortex")
            .build(&event_loop),
    )?;
    let start = Instant::now();

    engine.run(event_loop, move |frame| {
        let time = start.elapsed().as_secs_f32();
        let w = frame.config.width as f32;
        let h = frame.config.height as f32;
        let camera = Camera2D::new(w, h).with_pos(Vec2::new(w / 2.0, h / 2.0));

        frame.renderer.batch2d.reset();

        for i in 0..50000 {
            let angle = i as f32 * 0.1 + time;
            let radius = 50.0 + (i as f32 * 0.05) % 300.0;
            let x = w / 2.0 + angle.cos() * radius;
            let y = h / 2.0 + angle.sin() * radius;

            let size = 2.0 + (i as f32 % 5.0);
            let color = Vec4::new(
                (angle.sin() + 1.0) * 0.5,
                (angle.cos() + 1.0) * 0.5,
                0.8,
                0.9,
            );

            frame.renderer.batch2d.push(Shape2D {
                pos: Vec2::new(x, y),
                size: Vec2::new(size, size),
                color,
                kind: if i % 3 == 0 {
                    ShapeKind::Circle
                } else {
                    ShapeKind::Rect
                },
                rot: angle + time,
            });
        }

        let fps_text = format!(
            "FPS: {:.0} | Shapes: {}",
            frame.ctx.fps,
            frame.renderer.batch2d.count()
        );
        push_text(
            &mut frame.renderer.batch2d,
            "2D VORTEX",
            10.0,
            10.0,
            4.0,
            Vec4::new(1.0, 0.2, 0.2, 1.0),
        );
        push_text(
            &mut frame.renderer.batch2d,
            &fps_text,
            10.0,
            40.0,
            3.0,
            Vec4::new(1.0, 1.0, 1.0, 1.0),
        );

        let mvp = camera.build_view_projection();
        frame.renderer.batch2d.flush(frame.ctx, mvp);
    })?;
    Ok(())
}
