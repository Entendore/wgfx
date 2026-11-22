use anyhow::Result;
use std::time::Instant;
use wgfx::core::font::push_text;
use wgfx::math::{Camera2D, Vec2, Vec4};
use winit::event_loop::EventLoop;

fn main() -> Result<()> {
    env_logger::init();
    let event_loop = EventLoop::new()?;
    let engine = pollster::block_on(
        wgfx::Engine::builder()
            .with_title("wgfx - Sprite Mosaic")
            .build(&event_loop),
    )?;
    let start = Instant::now();

    engine.run(event_loop, move |frame| {
        let time = start.elapsed().as_secs_f32();
        let w = frame.config.width as f32;
        let h = frame.config.height as f32;
        let camera = Camera2D::new(w, h).with_pos(Vec2::new(w / 2.0, h / 2.0));

        frame.renderer.sprite_batcher.reset();

        let cols = 200;
        let rows = 150;
        let cell_w = w / cols as f32;
        let cell_h = h / rows as f32;

        for i in 0..(cols * rows) {
            let x = (i % cols) as f32 * cell_w;
            let y = (i / cols) as f32 * cell_h;

            let wave = (x / 50.0 + time).sin() * (y / 50.0 + time).cos();
            let size = cell_w.min(cell_h) * 0.8 + wave.abs() * 3.0;

            let color = Vec4::new(1.0, (wave + 1.0) * 0.5, 0.0, 0.8);

            frame.renderer.sprite_batcher.push(
                Vec2::new(x + (cell_w - size) / 2.0, y + (cell_h - size) / 2.0),
                Vec2::new(size, size),
                color,
            );
        }

        frame.renderer.batch2d.reset();
        push_text(
            &mut frame.renderer.batch2d,
            "TEXTURED MOSAIC",
            10.0,
            10.0,
            4.0,
            Vec4::new(0.2, 1.0, 0.2, 1.0),
        );
        let fps_text = format!("FPS: {:.0}", frame.ctx.fps);
        push_text(
            &mut frame.renderer.batch2d,
            &fps_text,
            10.0,
            40.0,
            3.0,
            Vec4::new(1.0, 1.0, 1.0, 1.0),
        );

        let mvp = camera.build_view_projection();
        frame.renderer.sprite_batcher.flush(frame.ctx, mvp);
        frame.renderer.batch2d.flush(frame.ctx, mvp);
    })?;
    Ok(())
}
