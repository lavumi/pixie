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
use crate::renderer::vertex::DebugLineVertex;
use crate::renderer::RenderError;
use crate::{AtlasId, DebugLine, RenderViewport, UiRoot, ViewportMode};

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

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

#[derive(Default)]
struct DebugLineBuffers {
    vertices: Vec<DebugLineVertex>,
    vertex_buffer: Option<wgpu::Buffer>,
    vertex_capacity: usize,
    vertex_count: u32,
}

impl DebugLineBuffers {
    fn update(&mut self, lines: &[DebugLine], device: &wgpu::Device, queue: &wgpu::Queue) {
        self.vertices.clear();
        for line in lines {
            append_debug_line_vertices(&mut self.vertices, line);
        }
        self.vertex_count = self.vertices.len() as u32;

        if self.vertices.is_empty() {
            return;
        }

        if self.vertices.len() > self.vertex_capacity {
            self.vertex_capacity = self.vertices.len().next_power_of_two().max(384);
            self.vertex_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Debug Line Vertex Buffer"),
                size: (self.vertex_capacity * std::mem::size_of::<DebugLineVertex>())
                    as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

        queue.write_buffer(
            self.vertex_buffer
                .as_ref()
                .expect("debug line vertex buffer must be allocated"),
            0,
            bytemuck::cast_slice(&self.vertices),
        );
    }
}

fn append_debug_line_vertices(vertices: &mut Vec<DebugLineVertex>, line: &DebugLine) {
    let delta = [line.end[0] - line.start[0], line.end[1] - line.start[1]];
    let length = delta[0].hypot(delta[1]);
    if length <= f32::EPSILON {
        return;
    }

    let half_thickness = line.thickness * 0.5;
    let normal = [
        -delta[1] / length * half_thickness,
        delta[0] / length * half_thickness,
    ];
    let start_left = DebugLineVertex {
        position: [
            line.start[0] + normal[0],
            line.start[1] + normal[1],
            line.start[2],
        ],
        color: line.color,
    };
    let start_right = DebugLineVertex {
        position: [
            line.start[0] - normal[0],
            line.start[1] - normal[1],
            line.start[2],
        ],
        color: line.color,
    };
    let end_left = DebugLineVertex {
        position: [
            line.end[0] + normal[0],
            line.end[1] + normal[1],
            line.end[2],
        ],
        color: line.color,
    };
    let end_right = DebugLineVertex {
        position: [
            line.end[0] - normal[0],
            line.end[1] - normal[1],
            line.end[2],
        ],
        color: line.color,
    };

    vertices.extend_from_slice(&[
        start_left,
        start_right,
        end_left,
        end_left,
        start_right,
        end_right,
    ]);
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
    debug_line_buffers: DebugLineBuffers,

    color: wgpu::Color,
    depth_texture: texture::Texture,

    aspect_ratio: f32,
    viewport_data: [f32; 6],
}
impl RenderState {
    pub async fn new(window: Arc<Window>, width: u32, height: u32) -> Result<Self, RenderError> {
        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());

        // # Safety
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = instance.create_surface(window)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;
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
            .await?;

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
            .or_else(|| surface_caps.formats.first().copied())
            .ok_or(RenderError::SurfaceConfiguration(
                "adapter reported no supported surface formats",
            ))?;
        let present_mode = surface_caps.present_modes.first().copied().ok_or(
            RenderError::SurfaceConfiguration("adapter reported no supported present modes"),
        )?;
        let alpha_mode =
            surface_caps
                .alpha_modes
                .first()
                .copied()
                .ok_or(RenderError::SurfaceConfiguration(
                    "adapter reported no supported alpha modes",
                ))?;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode,
            desired_maximum_frame_latency: 2,
            alpha_mode,
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");
        let color = wgpu::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };

        let aspect_ratio = width as f32 / height as f32;
        let viewport_data = RenderViewport::new(0.0, 0.0, width as f32, height as f32).wgpu_data();

        let mut gpu_resource_manager = GPUResourceManager::default();
        gpu_resource_manager.initialize(&device);
        let mut pipeline_manager = PipelineManager::default();
        pipeline_manager.init_pipelines(&device, config.format, &gpu_resource_manager);

        let font_manager = FontManager::new()?;

        Ok(Self {
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
            debug_line_buffers: DebugLineBuffers::default(),
        })
    }

    pub async fn init_resources(&mut self) -> Result<(), RenderError> {
        // Initialize UI resources (font system)
        // Generate font atlas from TTF at runtime
        let font_texture = self
            .font_manager
            .make_font_atlas_rgba(&self.device, &self.queue, RASTER_SIZE)
            .await?;
        self.gpu_resource_manager
            .init_ui_atlas_from_texture(font_texture, &self.device)
            .await;
        self.gpu_resource_manager.init_ui_meshes(&self.device);
        Ok(())
    }

    /// Load a texture atlas and auto-create a quad mesh for rendering
    pub(crate) fn load_texture_atlas(
        &mut self,
        name: &crate::AtlasId,
        image_bytes: &[u8],
    ) -> Result<(), RenderError> {
        self.gpu_resource_manager.load_texture_atlas(
            name,
            image_bytes,
            &self.device,
            &self.queue,
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn set_clear_color(&mut self, color: wgpu::Color) {
        self.color = color;
    }
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>, viewport_mode: ViewportMode) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            self.surface.configure(&self.device, &self.config);

            self.viewport_data = viewport_mode
                .viewport(new_size.width, new_size.height, self.aspect_ratio)
                .wgpu_data();
        }
    }

    pub fn viewport(&self) -> RenderViewport {
        RenderViewport::new(
            self.viewport_data[0],
            self.viewport_data[1],
            self.viewport_data[2],
            self.viewport_data[3],
        )
    }
    fn update_camera_buffer(&self, camera_uniform: [[f32; 4]; 4]) -> Result<(), RenderError> {
        self.update_named_camera_buffer("camera_matrix", camera_uniform)
    }

    fn update_ui_camera_buffer(&self) -> Result<(), RenderError> {
        let half_width = self.config.width as f32 * 0.5;
        let half_height = self.config.height as f32 * 0.5;
        let camera_uniform = (OPENGL_TO_WGPU_MATRIX
            * cgmath::ortho(
                -half_width,
                half_width,
                -half_height,
                half_height,
                -100.0,
                100.0,
            ))
        .into();

        self.update_named_camera_buffer("ui_camera_matrix", camera_uniform)
    }

    fn update_named_camera_buffer(
        &self,
        name: &str,
        camera_uniform: [[f32; 4]; 4],
    ) -> Result<(), RenderError> {
        let camera_buffer = self.gpu_resource_manager.get_buffer(name).ok_or_else(|| {
            RenderError::MissingGpuResource {
                resource_type: "buffer",
                name: name.to_string(),
            }
        })?;
        self.queue
            .write_buffer(&camera_buffer, 0, bytemuck::cast_slice(&[camera_uniform]));
        Ok(())
    }

    fn update_frame(&mut self, frame: &RenderFrame<'_>) -> Result<(), RenderError> {
        self.update_camera_buffer(frame.camera_uniform())?;
        self.update_ui_camera_buffer()?;
        self.update_sprite_instances(frame)?;
        self.debug_line_buffers
            .update(frame.debug_lines(), &self.device, &self.queue);
        self.update_text_instance("world_text", frame.world_texts());
        self.update_screen_text_instance(frame.screen_texts());
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

    fn update_text_instance(&mut self, mesh_name: &str, texts: &[TextRenderData]) {
        let sprite_instances = texts
            .iter()
            .flat_map(|text| self.font_manager.make_instance_buffer(text))
            .collect::<Vec<_>>();

        self.gpu_resource_manager.update_color_sprite_instances(
            mesh_name,
            &self.device,
            &self.queue,
            sprite_instances,
        );
    }

    fn update_screen_text_instance(&mut self, texts: &[TextRenderData]) {
        let window_size = [self.config.width as f32, self.config.height as f32];
        let window = RenderViewport::new(0.0, 0.0, window_size[0], window_size[1]);
        let viewport = self.viewport();
        let sprite_instances = texts
            .iter()
            .flat_map(|text| {
                let ui_transform = text.ui_transform.unwrap_or_default();
                let root = match ui_transform.root {
                    UiRoot::RenderViewport => viewport,
                    UiRoot::Window => window,
                };
                self.font_manager
                    .make_ui_instance_buffer(text, ui_transform, root, window_size)
            })
            .collect::<Vec<_>>();

        self.gpu_resource_manager.update_color_sprite_instances(
            "screen_text",
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

            if self.debug_line_buffers.vertex_count > 0 {
                let render_pipeline = self.pipeline_manager.get_pipeline("debug_line_pl");
                render_pass.set_pipeline(render_pipeline);
                self.gpu_resource_manager
                    .set_bind_group(&mut render_pass, "camera");
                render_pass.set_vertex_buffer(
                    0,
                    self.debug_line_buffers
                        .vertex_buffer
                        .as_ref()
                        .expect("debug line vertex buffer must be allocated")
                        .slice(..),
                );
                render_pass.draw(0..self.debug_line_buffers.vertex_count, 0..1);
            }

            let render_pipeline = self.pipeline_manager.get_pipeline("font_pl");
            render_pass.set_pipeline(render_pipeline);
            self.gpu_resource_manager
                .set_bind_group(&mut render_pass, "camera");
            self.gpu_resource_manager
                .render_text(&mut render_pass, "world_text");
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("UI Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_viewport(
                0.0,
                0.0,
                self.config.width as f32,
                self.config.height as f32,
                0.0,
                1.0,
            );
            let render_pipeline = self.pipeline_manager.get_pipeline("ui_font_pl");
            render_pass.set_pipeline(render_pipeline);
            self.gpu_resource_manager
                .set_bind_group(&mut render_pass, "ui_camera");
            self.gpu_resource_manager
                .render_text(&mut render_pass, "screen_text");
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

    #[test]
    fn horizontal_debug_line_expands_to_two_triangles() {
        let line = DebugLine::new([0.0, 0.0, 0.5], [2.0, 0.0, 0.5], [1.0, 0.0, 0.0, 0.75], 0.2);
        let mut vertices = Vec::new();

        append_debug_line_vertices(&mut vertices, &line);

        assert_eq!(vertices.len(), 6);
        assert_eq!(vertices[0].position, [0.0, 0.1, 0.5]);
        assert_eq!(vertices[1].position, [0.0, -0.1, 0.5]);
        assert_eq!(vertices[2].position, [2.0, 0.1, 0.5]);
        assert_eq!(vertices[5].position, [2.0, -0.1, 0.5]);
        assert!(vertices.iter().all(|vertex| vertex.color == line.color));
    }

    #[test]
    fn zero_length_debug_line_produces_no_geometry() {
        let line = DebugLine::new([1.0, 1.0, 0.0], [1.0, 1.0, 0.0], [1.0, 1.0, 1.0, 1.0], 0.1);
        let mut vertices = Vec::new();

        append_debug_line_vertices(&mut vertices, &line);

        assert!(vertices.is_empty());
    }
}
