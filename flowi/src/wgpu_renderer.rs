use crate::image::Image;
use flowi_core::renderer::Texture as CoreTexture;
use flowi_core::ApplicationSettings;
use flowi_core::render::{FlowiRenderer, WindowWrapper};
use flowi_core::imgui::{DrawCmd, DrawData, DrawVert, FontAtlas, ImDrawIdx};

pub struct WgpuRenderer {
    instance: wgpu::Instance,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl FlowiRenderer for WgpuRenderer {
    fn new(settings: &ApplicationSettings, window: &WindowWrapper) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
    
        let surface = unsafe { instance.create_surface(window) }.unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None)).unwrap();

        // Set up swap chain
        let surface_desc = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: settings.width as _,
            height: settings.height as _,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![wgpu::TextureFormat::Bgra8Unorm],
        };

        surface.configure(&device, &surface_desc);

        let _font_atlas = FontAtlas::build_rgba32_texture();

        WgpuRenderer {
            instance,
            surface,
            device,
            queue,
        }
    }

    fn render(&mut self) {
        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    fn get_texture(&mut self, _image: Image) -> CoreTexture {
        CoreTexture { handle: 0 }
    }
}

/*
impl WgpuRenderer {
    // ...
    pub fn new(device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor) -> Self {
        // ...
        let render_pipeline = Self::create_render_pipeline(device, sc_desc);
        // ...
        Self {
            // ...
            render_pipeline,
            // ...
        }
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
    ) -> wgpu::RenderPipeline {
        // ...
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                // ...
            });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            // ...
            layout: Some(&render_pipeline_layout),
            // ...
        })
    }
    // ...
}
*/
