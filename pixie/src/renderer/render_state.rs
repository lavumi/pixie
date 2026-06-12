use std::collections::HashMap;
use std::iter;
use std::sync::Arc;

use winit::window::Window;

use crate::renderer::font_manager::FontManager;
use crate::renderer::font_manager::RASTER_SIZE;
use crate::renderer::gpu_resource_manager::GPUResourceManager;
use crate::renderer::mesh::SpriteInstanceRaw;
use crate::renderer::pipeline_manager::PipelineManager;
use crate::renderer::render_input_data::*;
use crate::renderer::texture;
use crate::renderer::RenderError;
use crate::AtlasId;

#[derive(Default)]
struct SpriteInstanceBuffers {
    by_atlas: HashMap<AtlasId, Vec<SpriteInstanceRaw>>,
}

impl SpriteInstanceBuffers {
    fn update(&mut self, atlas: &AtlasId, sprites: &[SpriteRenderData]) -> &[SpriteInstanceRaw] {
        let instances = self.by_atlas.entry(atlas.clone()).or_default();
        instances.clear();
        instances.extend(sprites.iter().map(SpriteRenderData::get_instance_matrix));
        instances
    }
}

pub struct RenderState {
    pub device: wgpu::Device,
    surface: wgpu::Surface<'static>,

    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,

    pub gpu_resource_manager: GPUResourceManager,
    pub pipeline_manager: PipelineManager,

    font_manager: FontManager,
    sprite_instance_buffers: SpriteInstanceBuffers,

    color: wgpu::Color,
    depth_texture: texture::Texture,

    aspect_ratio: f32,
    viewport_data: [f32; 6],
}
impl RenderState {
    pub async fn new(window: Arc<Window>, width: u32, height: u32) -> Self {
        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

        // # Safety
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu`s features, so if
                // we're building for the web we'll have to disable some.
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| {
                matches!(
                    f,
                    wgpu::TextureFormat::Rgba8UnormSrgb | wgpu::TextureFormat::Bgra8UnormSrgb
                )
            })
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");
        let color = wgpu::Color {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        };

        let aspect_ratio = width as f32 / height as f32;
        let viewport_data = [0., 0., width as f32, height as f32, 0., 1.];

        let mut gpu_resource_manager = GPUResourceManager::default();
        gpu_resource_manager.initialize(&device);
        let mut pipeline_manager = PipelineManager::default();
        pipeline_manager.init_pipelines(&device, config.format, &gpu_resource_manager);

        let font_manager = FontManager::default();

        Self {
            device,
            surface,
            queue,
            config,
            gpu_resource_manager,
            pipeline_manager,
            color,
            depth_texture,
            aspect_ratio,
            viewport_data,
            font_manager,
            sprite_instance_buffers: SpriteInstanceBuffers::default(),
        }
    }

    pub async fn init_resources(&mut self) {
        // Initialize UI resources (font system)
        // Generate font atlas from TTF at runtime
        let font_texture = self
            .font_manager
            .make_font_atlas_rgba(&self.device, &self.queue, RASTER_SIZE)
            .await
            .unwrap();
        self.gpu_resource_manager
            .init_ui_atlas_from_texture(font_texture, &self.device)
            .await;
        self.gpu_resource_manager.init_ui_meshes(&self.device);
    }

    /// Load a texture atlas and auto-create a quad mesh for rendering
    pub(crate) fn load_texture_atlas(
        &mut self,
        name: &crate::AtlasId,
        image_bytes: &[u8],
    ) -> Result<(), crate::AtlasError> {
        self.gpu_resource_manager
            .load_texture_atlas(name, image_bytes, &self.device, &self.queue)
    }

    #[allow(dead_code)]
    pub fn set_clear_color(&mut self, color: wgpu::Color) {
        self.color = color;
    }
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            self.surface.configure(&self.device, &self.config);

            let aspect_ratio = new_size.width as f32 / new_size.height as f32;

            if (self.aspect_ratio - aspect_ratio).abs() > 0.02 {
                if self.aspect_ratio < aspect_ratio {
                    //width is bigger
                    let adjust_width = new_size.height as f32 * self.aspect_ratio;
                    let x_offset = (new_size.width as f32 - adjust_width) * 0.5;

                    self.viewport_data =
                        [x_offset, 0., adjust_width, new_size.height as f32, 0., 1.];
                } else {
                    let adjust_height = new_size.width as f32 / self.aspect_ratio;
                    self.viewport_data = [0., 0., new_size.width as f32, adjust_height, 0., 1.];
                }
            } else {
                self.viewport_data = [
                    0.,
                    0.,
                    new_size.width as f32,
                    new_size.height as f32,
                    0.,
                    1.,
                ];
            }
        }
    }
    fn update_camera_buffer(&self, camera_uniform: [[f32; 4]; 4]) {
        let camera_buffer = self.gpu_resource_manager.get_buffer("camera_matrix");
        self.queue
            .write_buffer(&camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));
    }

    fn update_frame(&mut self, frame: &RenderFrame<'_>) -> Result<(), crate::AtlasError> {
        self.update_camera_buffer(frame.camera_uniform());
        self.update_sprite_instances(frame)?;
        self.update_text_instance(frame.texts());
        Ok(())
    }

    fn update_sprite_instances(
        &mut self,
        frame: &RenderFrame<'_>,
    ) -> Result<(), crate::AtlasError> {
        for (atlas, sprite_data) in frame.sprite_batches() {
            let instance_data = self.sprite_instance_buffers.update(atlas, sprite_data);

            self.gpu_resource_manager.update_sprite_instances(
                atlas,
                &self.device,
                &self.queue,
                instance_data,
            )?;
        }
        Ok(())
    }

    fn update_text_instance(&mut self, texts: &[TextRenderData]) {
        let sprite_instances = texts
            .iter()
            .flat_map(|text| self.font_manager.make_instance_buffer(text))
            .collect::<Vec<_>>();

        self.gpu_resource_manager.update_color_sprite_instances(
            &self.device,
            &self.queue,
            sprite_instances,
        );
    }

    fn render(&self, frame: &RenderFrame<'_>) -> Result<(), RenderError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.color),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_viewport(
                self.viewport_data[0],
                self.viewport_data[1],
                self.viewport_data[2],
                self.viewport_data[3],
                self.viewport_data[4],
                self.viewport_data[5],
            );

            let render_pipeline = self.pipeline_manager.get_pipeline("sprite_pl");
            render_pass.set_pipeline(render_pipeline);
            self.gpu_resource_manager
                .render(&mut render_pass, frame.sprite_atlases())?;

            let render_pipeline = self.pipeline_manager.get_pipeline("font_pl");
            render_pass.set_pipeline(render_pipeline);
            self.gpu_resource_manager.render_ui(&mut render_pass);
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    pub fn render_frame(&mut self, frame: &RenderFrame<'_>) -> Result<(), RenderError> {
        self.update_frame(frame)?;
        self.render(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sprite_data(count: usize) -> Vec<SpriteRenderData> {
        (0..count)
            .map(|index| SpriteRenderData {
                uv: [0.0, 1.0, 0.0, 1.0],
                position: [index as f32, 0.0, 0.0],
                size: [1.0, 1.0],
                rotation: 0.0,
            })
            .collect()
    }

    #[test]
    fn sprite_instance_cpu_capacity_is_reused() {
        let atlas = AtlasId::from("main");
        let mut buffers = SpriteInstanceBuffers::default();

        buffers.update(&atlas, &sprite_data(5));
        let capacity = buffers.by_atlas[&atlas].capacity();
        buffers.update(&atlas, &sprite_data(2));

        assert_eq!(buffers.by_atlas[&atlas].len(), 2);
        assert_eq!(buffers.by_atlas[&atlas].capacity(), capacity);
    }
}
