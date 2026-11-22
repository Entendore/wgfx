//! # wgfx - A Custom Rust WebGPU Engine
//!
//! A lightweight rendering engine built on top of wgpu and winit.
//!
//! ## Features
//! - 2D batch renderer with shapes (rect, circle, rounded rect)
//! - 3D batch renderer with instanced meshes
//! - GPU compute particle system with lifetime
//! - Line renderer for debug visualization
//! - Post-processing effects (grayscale, vignette, chromatic aberration)
//! - Bitmap font renderer
//! - Input manager (keyboard, mouse, text input)
//! - Camera system (2D and 3D)

pub mod core;
pub mod error;
pub mod math;
pub mod renderer;

use core::{Input, Mesh3D, Texture};
use error::WgfxError;
use renderer::{PostProcessConfig, PostProcessRenderer, Renderer};
use std::sync::Arc;
use std::time::Instant;
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::window::Window;

/// Context passed to the render callback. Provides access to the render pass,
/// queue, device, and frame timing info.
pub struct RenderContext<'a> {
    pub pass: &'a mut wgpu::RenderPass<'a>,
    pub queue: &'a wgpu::Queue,
    pub device: &'a wgpu::Device,
    pub dt: f32,
    pub fps: f32,
}

/// Frame data passed to the user's render callback.
pub struct Frame<'a> {
    pub renderer: &'a mut Renderer,
    pub ctx: &'a mut RenderContext<'a>,
    pub input: &'a Input,
    pub window: &'a Window,
    pub config: &'a wgpu::SurfaceConfiguration,
    pub use_post_processing: &'a mut bool,
    pub post_process_config: &'a mut PostProcessConfig,
}

/// Builder for configuring the engine before creation.
pub struct EngineBuilder {
    title: String,
    width: u32,
    height: u32,
    vsync: bool,
    present_mode: wgpu::PresentMode,
}

impl Default for EngineBuilder {
    fn default() -> Self {
        Self {
            title: "wgfx App".to_string(),
            width: 1280,
            height: 720,
            vsync: true,
            present_mode: wgpu::PresentMode::Fifo,
        }
    }
}

impl EngineBuilder {
    #[must_use]
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    #[must_use]
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Enable or disable VSync. When enabled (default), uses Fifo present mode.
    /// When disabled, uses Mailbox for uncapped framerate.
    #[must_use]
    pub fn with_vsync(mut self, vsync: bool) -> Self {
        self.vsync = vsync;
        self.present_mode = if vsync {
            wgpu::PresentMode::Fifo
        } else {
            wgpu::PresentMode::Mailbox
        };
        self
    }

    /// Set a custom present mode.
    #[must_use]
    pub fn with_present_mode(mut self, mode: wgpu::PresentMode) -> Self {
        self.present_mode = mode;
        self
    }

    pub async fn build(self, event_loop: &EventLoop<()>) -> Result<Engine, WgfxError> {
        let window = Arc::new(
            winit::window::WindowBuilder::new()
                .with_title(self.title)
                .with_inner_size(PhysicalSize::new(self.width, self.height))
                .build(event_loop)?,
        );
        Engine::new(window, self.present_mode).await
    }
}

pub struct Engine {
    pub window: Arc<Window>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
    pub depth_texture: Texture,
    pub checker_texture: Texture,
    pub offscreen_texture: Texture,
    pub post_process: PostProcessRenderer,
    use_post_processing: bool,
    post_process_config: PostProcessConfig,
    pub renderer: Renderer,
    input: Input,
    last_frame: Instant,
    fps_accum: f32,
    fps_frames: u32,
    current_fps: f32,
}

impl Engine {
    #[must_use]
    pub fn builder() -> EngineBuilder {
        EngineBuilder::default()
    }

    async fn new(window: Arc<Window>, present_mode: wgpu::PresentMode) -> Result<Self, WgfxError> {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone())?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or(WgfxError::AdapterNotFound)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::default(),
                    label: None,
                },
                None,
            )
            .await?;

        device.on_uncaptured_error(Box::new(|e| {
            eprintln!("🔥 WebGPU Uncaptured Error: {:?}", e);
            panic!("WebGPU validation error occurred.");
        }));

        let size = window.inner_size();
        let surface_caps = surface.get_capabilities(&adapter);
        let format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let depth_format = wgpu::TextureFormat::Depth32Float;
        let depth_texture =
            Texture::create_depth_texture(&device, config.width, config.height, depth_format);
        let checker_texture = Texture::create_checkerboard(&device, &queue);
        let offscreen_texture =
            Texture::create_color_texture(&device, config.width, config.height, format);
        let post_process = PostProcessRenderer::new(&device, format, &offscreen_texture);

        let cube_mesh = Mesh3D::cube();
        let renderer = Renderer::new(&device, format, depth_format, &checker_texture, &cube_mesh);

        Ok(Self {
            window,
            device,
            queue,
            surface,
            config,
            depth_texture,
            checker_texture,
            offscreen_texture,
            post_process,
            use_post_processing: false,
            post_process_config: PostProcessConfig::default(),
            renderer,
            input: Input::default(),
            last_frame: Instant::now(),
            fps_accum: 0.0,
            fps_frames: 0,
            current_fps: 0.0,
        })
    }

    pub fn run<F>(mut self, event_loop: EventLoop<()>, mut callback: F) -> Result<(), WgfxError>
    where
        F: FnMut(&mut Frame) + 'static,
    {
        if std::env::var("RUST_BACKTRACE").is_err() {
            std::env::set_var("RUST_BACKTRACE", "1");
        }

        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            eprintln!("❌ WGPU ENGINE PANIC ❌");
            if let Some(location) = info.location() {
                eprintln!("Location: {}", location);
            }
            if let Some(s) = info.payload().downcast_ref::<&str>() {
                eprintln!("Message: {}", s);
            } else if let Some(s) = info.payload().downcast_ref::<String>() {
                eprintln!("Message: {}", s);
            }
            eprintln!("Backtrace:");
            default_hook(info);
        }));

        event_loop.run(move |event, elwt| {
            if let Event::WindowEvent { ref event, .. } = event {
                match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::Resized(size) => self.resize(*size),
                    WindowEvent::RedrawRequested => {
                        let now = Instant::now();
                        let dt = now.duration_since(self.last_frame).as_secs_f32();
                        self.last_frame = now;
                        // Clamp dt to prevent huge jumps after pause/lag
                        let dt = dt.min(0.1);
                        self.fps_accum += dt;
                        self.fps_frames += 1;
                        if self.fps_accum >= 0.5 {
                            self.current_fps = self.fps_frames as f32 / self.fps_accum;
                            self.fps_accum = 0.0;
                            self.fps_frames = 0;
                        }

                        let frame_result = self.surface.get_current_texture();
                        match frame_result {
                            Ok(output) => {
                                let view = output
                                    .texture
                                    .create_view(&wgpu::TextureViewDescriptor::default());
                                let mut encoder = self.device.create_command_encoder(
                                    &wgpu::CommandEncoderDescriptor {
                                        label: Some("Render Encoder"),
                                    },
                                );
                                let target_view = if self.use_post_processing {
                                    &self.offscreen_texture.view
                                } else {
                                    &view
                                };

                                {
                                    let mut render_pass =
                                        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                            label: Some("Render Pass"),
                                            color_attachments: &[Some(
                                                wgpu::RenderPassColorAttachment {
                                                    view: target_view,
                                                    resolve_target: None,
                                                    ops: wgpu::Operations {
                                                        load: wgpu::LoadOp::Clear(wgpu::Color {
                                                            r: 0.1,
                                                            g: 0.1,
                                                            b: 0.15,
                                                            a: 1.0,
                                                        }),
                                                        store: wgpu::StoreOp::Store,
                                                    },
                                                },
                                            )],
                                            depth_stencil_attachment: Some(
                                                wgpu::RenderPassDepthStencilAttachment {
                                                    view: &self.depth_texture.view,
                                                    depth_ops: Some(wgpu::Operations {
                                                        load: wgpu::LoadOp::Clear(1.0),
                                                        store: wgpu::StoreOp::Store,
                                                    }),
                                                    stencil_ops: None,
                                                },
                                            ),
                                            timestamp_writes: None,
                                            occlusion_query_set: None,
                                        });

                                    let mut ctx = RenderContext {
                                        pass: &mut render_pass,
                                        queue: &self.queue,
                                        device: &self.device,
                                        dt,
                                        fps: self.current_fps,
                                    };

                                    let renderer = &mut self.renderer;
                                    let input = &self.input;
                                    let window = &self.window;
                                    let config = &self.config;
                                    let use_post_processing = &mut self.use_post_processing;
                                    let post_process_config = &mut self.post_process_config;

                                    let mut frame = Frame {
                                        renderer,
                                        ctx: &mut ctx,
                                        input,
                                        window,
                                        config,
                                        use_post_processing,
                                        post_process_config,
                                    };

                                    callback(&mut frame);
                                }

                                if self.use_post_processing {
                                    self.post_process.set_config(self.post_process_config);
                                    let mut post_pass =
                                        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                            label: Some("Post Process Pass"),
                                            color_attachments: &[Some(
                                                wgpu::RenderPassColorAttachment {
                                                    view: &view,
                                                    resolve_target: None,
                                                    ops: wgpu::Operations {
                                                        load: wgpu::LoadOp::Clear(
                                                            wgpu::Color::BLACK,
                                                        ),
                                                        store: wgpu::StoreOp::Store,
                                                    },
                                                },
                                            )],
                                            depth_stencil_attachment: None,
                                            timestamp_writes: None,
                                            occlusion_query_set: None,
                                        });
                                    let mut post_ctx = RenderContext {
                                        pass: &mut post_pass,
                                        queue: &self.queue,
                                        device: &self.device,
                                        dt,
                                        fps: self.current_fps,
                                    };
                                    self.post_process.flush(&mut post_ctx);
                                }

                                self.queue.submit(std::iter::once(encoder.finish()));
                                output.present();
                            }
                            Err(e) => match e {
                                wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated => {
                                    log::warn!("Surface Lost/Outdated. Reconfiguring...");
                                    self.resize(self.window.inner_size());
                                }
                                wgpu::SurfaceError::OutOfMemory => {
                                    log::error!("Fatal Error: Out of Memory");
                                    elwt.exit();
                                }
                                wgpu::SurfaceError::Timeout => {
                                    log::warn!("Surface Timeout. Skipping frame.");
                                }
                            },
                        }

                        self.input.clear_just_pressed();
                    }
                    _ => self.input.process_event(event),
                }
            } else if let Event::AboutToWait = event {
                self.window.request_redraw();
            }
        })?;
        Ok(())
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = Texture::create_depth_texture(
                &self.device,
                self.config.width,
                self.config.height,
                self.depth_texture.format,
            );
            self.offscreen_texture = Texture::create_color_texture(
                &self.device,
                self.config.width,
                self.config.height,
                self.config.format,
            );
            self.post_process =
                PostProcessRenderer::new(&self.device, self.config.format, &self.offscreen_texture);
        }
    }
}
