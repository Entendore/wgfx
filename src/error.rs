use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum WgfxError {
    #[error(
        "Failed to find an appropriate GPU adapter. Ensure your drivers support Vulkan/Metal/DX12."
    )]
    AdapterNotFound,
    #[error("Failed to request GPU device: {0}")]
    DeviceRequestFailed(#[from] wgpu::RequestDeviceError),
    #[error("Surface creation failed: {0}")]
    SurfaceCreationFailed(#[from] wgpu::CreateSurfaceError),
    #[error("WGPU Surface Error: {0}")]
    SurfaceError(#[from] wgpu::SurfaceError),
    #[error("Window creation error: {0}")]
    WindowCreation(#[from] winit::error::OsError),
    #[error("Event Loop error: {0}")]
    EventLoop(#[from] winit::error::EventLoopError),
    #[error(
        "Buffer capacity exceeded: tried to push {requested} items but only {capacity} available"
    )]
    BufferOverflow { requested: usize, capacity: usize },
}
