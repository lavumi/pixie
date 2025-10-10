use std::cmp::{max, min};
use std::collections::HashMap;
use fontdue::Metrics;
use crate::renderer::mesh::InstanceColorTileRaw;
use crate::renderer::TextRenderData;

const RENDER_CHARACTER_ARRAY: [char; 64] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', ':', '.',
];

struct Glyph {
    bitmap: Vec<u8>,
    metrics: Metrics,
}

pub struct RasterizedFont {
    glyphs: Vec<Glyph>,
    size: [usize; 2],
    max_y_min: i32,
}

struct FontRenderData {
    uv: [f32; 4],
    advance: f32,
}

pub struct FontManager {
    font_map: HashMap<char, FontRenderData>,
}

impl Default for FontManager {
    fn default() -> Self {
        FontManager {
            font_map: Default::default(),
        }
    }
}

impl FontManager {
    pub fn font_rasterize(&mut self, font_size: f32) -> RasterizedFont {
        let font = include_bytes!("../../assets/font/ZEN-SERIF.otf") as &[u8];
        // let font = include_bytes!("../../assets/font/Gameplay.ttf") as &[u8];
        let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();

        let mut size = [0, 0];
        let mut max_y_min = 0;
        let mut glyphs = vec![];

        for (_, character) in RENDER_CHARACTER_ARRAY.iter().enumerate() {
            let (metrics, bitmap) = font.rasterize(*character, font_size);
            glyphs.push(Glyph { bitmap, metrics });
            size[0] = max(size[0], metrics.width);
            size[1] = max(size[1], metrics.height);
            max_y_min = min(max_y_min, metrics.ymin);
        }

        let atlas_size = 512;
        let char_in_row = atlas_size / size[0];

        for (index, character) in RENDER_CHARACTER_ARRAY.iter().enumerate() {
            let uv = [
                (index % char_in_row) as f32 * size[0] as f32 / atlas_size as f32,
                ((index % char_in_row) + 1) as f32 * size[0] as f32 / atlas_size as f32,
                (index / char_in_row) as f32 * size[1] as f32 / atlas_size as f32,
                ((index / char_in_row) + 1) as f32 * size[1] as f32 / atlas_size as f32,
            ];

            // let _metrics = glyphs[index].metrics;

            let metrics = glyphs[index].metrics;
            self.font_map.insert(
                character.clone(),
                FontRenderData {
                    uv,
                    advance: metrics.advance_width,
                },
            );
        }

        return RasterizedFont {
            glyphs,
            size,
            max_y_min,
        };
    }

    pub fn make_font_buffer(
        &mut self,
        rasterized_font: RasterizedFont,
        output_buffer: wgpu::Buffer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<wgpu::Buffer, wgpu::SurfaceError> {
        let max_size = rasterized_font.size;

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("font buffer command encoder"),
        });

        let atlas_size = 512;
        let char_in_row = atlas_size / max_size[0];
        for (index, font_datum) in rasterized_font.glyphs.iter().enumerate() {
            let metrics = font_datum.metrics;
            let bitmap = &font_datum.bitmap;

            // Convert grayscale to RGBA
            let rgba_data: Vec<u8> = bitmap
                .iter()
                .flat_map(|&gray| [255, 255, 255, gray].into_iter())
                .collect();

            let size = wgpu::Extent3d {
                width: metrics.width as u32,
                height: metrics.height as u32,
                depth_or_array_layers: 1,
            };
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("single-font texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &rgba_data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(size.width * 4),
                    rows_per_image: Some(size.height),
                },
                size,
            );
            
            // 안전한 오프셋 계산 - 오버플로우 방지
            let char_x = index % char_in_row;
            let char_y = index / char_in_row;
            
            // 각 컴포넌트를 개별적으로 계산하여 오버플로우 방지
            let x_offset = char_x * max_size[0];
            let y_offset = char_y * atlas_size * max_size[1];
            let metrics_x = metrics.xmin.max(0) as usize; // 음수 방지
            
            // Y 위치 계산 - 안전한 방식
            let glyph_height = metrics.height as i32;
            let glyph_ymin = metrics.ymin;
            let y_adjustment = glyph_height + glyph_ymin - rasterized_font.max_y_min;
            let y_position = if y_adjustment >= 0 {
                max_size[1].saturating_sub(y_adjustment as usize)
            } else {
                max_size[1] // 음수인 경우 기본값 사용
            };
            
            let final_y_offset = atlas_size * y_position;
            
            // 최종 오프셋 계산 - 각 단계에서 오버플로우 체크
            let total_offset = x_offset
                .saturating_add(y_offset)
                .saturating_add(metrics_x)
                .saturating_add(final_y_offset);
            
            let offset = (total_offset * 4) as wgpu::BufferAddress;

            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    aspect: wgpu::TextureAspect::All,
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: &output_buffer,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset,
                        bytes_per_row: Some((atlas_size * 4).try_into().unwrap()),
                        rows_per_image: Some((atlas_size).try_into().unwrap()),
                    },
                },
                size,
            );
        }
        queue.submit(Some(encoder.finish()));
        return Ok(output_buffer);
    }

    pub async fn make_font_atlas_rgba(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        font_size: f32,
    ) -> Result<wgpu::Texture, wgpu::SurfaceError> {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("font atlasing command encoder"),
        });
        let rasterized_font = self.font_rasterize(font_size);

        let atlas_size = 512;
        let u8_size = std::mem::size_of::<u8>() as u32;
        let output_buffer_size = (u8_size * atlas_size * atlas_size * 4) as wgpu::BufferAddress;
        let output_buffer_desc = wgpu::BufferDescriptor {
            size: output_buffer_size,
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            label: Some("font atlas buffer"),
            mapped_at_creation: false,
        };
        let output_buffer = device.create_buffer(&output_buffer_desc);
        let output_buffer = self
            .make_font_buffer(rasterized_font, output_buffer, device, queue)
            .unwrap();

        // Make Font Atlas Texture
        let size = wgpu::Extent3d {
            width: atlas_size,
            height: atlas_size,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("font_atlas"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        encoder.copy_buffer_to_texture(
            wgpu::TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(atlas_size * 4),
                    rows_per_image: None,
                },
            },
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: Default::default(),
            },
            size,
        );
        queue.submit(Some(encoder.finish()));

        Ok(texture)
    }


    fn get_render_data(&self, char_key: char) -> &FontRenderData {
        match self.font_map.get(&char_key) {
            Some(value) => value,
            None => panic!("try to use unloaded font {}", char_key),
        }
    }

    pub fn make_instance_buffer(&self, text: &TextRenderData) -> Vec<InstanceColorTileRaw> {
        let line_space = 0.1;
        let mut result = Vec::new();
        let mut position = cgmath::Vector3 {
            x: text.position[0],
            y: text.position[1],
            z: text.position[2],
        };

        for txt in text.content.chars() {
            if txt == ' ' {
                // Use space advance - estimate as average character width
                position.x += text.size[0] * 0.5;
                continue;
            }
            if txt == '\n' {
                position.y -= text.size[1] + line_space;
                position.x = text.position[0];
                continue;
            }

            let render_data = self.get_render_data(txt);

            // Use consistent scaling for now - just use advance width for proportional spacing
            let scale_matrix = cgmath::Matrix4::from_nonuniform_scale(text.size[0], text.size[1], 1.0);
            let translation_matrix = cgmath::Matrix4::from_translation(position);
            let color = text.color;

            let model = (translation_matrix * scale_matrix).into();
            result.push(InstanceColorTileRaw {
                uv: render_data.uv,
                model,
                color,
            });

            // Advance cursor by actual advance width
            position.x += render_data.advance * text.size[0] / 24.0;
        }

        return result;
    }
}
