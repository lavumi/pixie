use crate::renderer::mesh::ColorSpriteInstanceRaw;
use crate::renderer::TextRenderData;
use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle as FontdueTextStyle};
use fontdue::{Font, Metrics};
use std::cmp::max;
use std::collections::HashMap;

const ATLAS_SIZE: usize = 512;
const ATLAS_PADDING: usize = 1;
pub(crate) const RASTER_SIZE: f32 = 48.0;
const RENDER_CHARACTER_ARRAY: [char; 64] = [
    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's',
    't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L',
    'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9', ':', '.',
];

struct Glyph {
    bitmap: Vec<u8>,
    metrics: Metrics,
    atlas_origin: [usize; 2],
}

pub struct RasterizedFont {
    glyphs: Vec<Glyph>,
}

struct FontRenderData {
    uv: [f32; 4],
}

pub struct FontManager {
    font: Font,
    font_map: HashMap<char, FontRenderData>,
}

impl Default for FontManager {
    fn default() -> Self {
        let bytes = include_bytes!("../../assets/font/ZEN-SERIF.otf") as &[u8];
        let font = Font::from_bytes(bytes, fontdue::FontSettings::default())
            .expect("embedded font must be valid");

        Self {
            font,
            font_map: HashMap::new(),
        }
    }
}

impl FontManager {
    pub fn font_rasterize(&mut self, font_size: f32) -> RasterizedFont {
        let mut max_size = [0, 0];
        let mut rasterized = Vec::with_capacity(RENDER_CHARACTER_ARRAY.len());

        for character in RENDER_CHARACTER_ARRAY {
            let (metrics, bitmap) = self.font.rasterize(character, font_size);
            max_size[0] = max(max_size[0], metrics.width);
            max_size[1] = max(max_size[1], metrics.height);
            rasterized.push((character, metrics, bitmap));
        }

        let cell_size = [
            max_size[0] + ATLAS_PADDING * 2,
            max_size[1] + ATLAS_PADDING * 2,
        ];
        let characters_per_row = ATLAS_SIZE / cell_size[0];
        assert!(
            characters_per_row > 0,
            "font glyphs do not fit in the atlas"
        );
        let required_rows = RENDER_CHARACTER_ARRAY.len().div_ceil(characters_per_row);
        assert!(
            required_rows * cell_size[1] <= ATLAS_SIZE,
            "font glyphs do not fit in the atlas"
        );

        self.font_map.clear();
        let mut glyphs = Vec::with_capacity(rasterized.len());

        for (index, (character, metrics, bitmap)) in rasterized.into_iter().enumerate() {
            let atlas_origin = [
                (index % characters_per_row) * cell_size[0] + ATLAS_PADDING,
                (index / characters_per_row) * cell_size[1] + ATLAS_PADDING,
            ];
            let uv = [
                (atlas_origin[0] as f32 + 0.5) / ATLAS_SIZE as f32,
                (atlas_origin[0] as f32 + metrics.width as f32 - 0.5) / ATLAS_SIZE as f32,
                (atlas_origin[1] as f32 + 0.5) / ATLAS_SIZE as f32,
                (atlas_origin[1] as f32 + metrics.height as f32 - 0.5) / ATLAS_SIZE as f32,
            ];

            self.font_map.insert(character, FontRenderData { uv });
            glyphs.push(Glyph {
                bitmap,
                metrics,
                atlas_origin,
            });
        }

        RasterizedFont { glyphs }
    }

    pub fn make_font_buffer(
        &mut self,
        rasterized_font: RasterizedFont,
        output_buffer: wgpu::Buffer,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<wgpu::Buffer, wgpu::SurfaceError> {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("font buffer command encoder"),
        });

        for glyph in &rasterized_font.glyphs {
            if glyph.metrics.width == 0 || glyph.metrics.height == 0 {
                continue;
            }

            let rgba_data: Vec<u8> = glyph
                .bitmap
                .iter()
                .flat_map(|&gray| [255, 255, 255, gray])
                .collect();
            let size = wgpu::Extent3d {
                width: glyph.metrics.width as u32,
                height: glyph.metrics.height as u32,
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

            let offset = ((glyph.atlas_origin[1] * ATLAS_SIZE + glyph.atlas_origin[0]) * 4) as u64;
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
                        bytes_per_row: Some((ATLAS_SIZE * 4) as u32),
                        rows_per_image: Some(ATLAS_SIZE as u32),
                    },
                },
                size,
            );
        }

        queue.submit(Some(encoder.finish()));
        Ok(output_buffer)
    }

    pub async fn make_font_atlas_rgba(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        font_size: f32,
    ) -> Result<wgpu::Texture, wgpu::SurfaceError> {
        assert!(
            (font_size - RASTER_SIZE).abs() < f32::EPSILON,
            "text layout and atlas raster size must match"
        );

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("font atlasing command encoder"),
        });
        let rasterized_font = self.font_rasterize(font_size);
        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            size: (ATLAS_SIZE * ATLAS_SIZE * 4) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
            label: Some("font atlas buffer"),
            mapped_at_creation: false,
        });
        let output_buffer = self
            .make_font_buffer(rasterized_font, output_buffer, device, queue)
            .unwrap();

        let size = wgpu::Extent3d {
            width: ATLAS_SIZE as u32,
            height: ATLAS_SIZE as u32,
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
                    bytes_per_row: Some((ATLAS_SIZE * 4) as u32),
                    rows_per_image: None,
                },
            },
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            size,
        );
        queue.submit(Some(encoder.finish()));

        Ok(texture)
    }

    fn get_render_data(&self, character: char) -> &FontRenderData {
        self.font_map
            .get(&character)
            .unwrap_or_else(|| panic!("try to use unloaded font {}", character))
    }

    pub fn make_instance_buffer(&self, text: &TextRenderData) -> Vec<ColorSpriteInstanceRaw> {
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&LayoutSettings {
            x: 0.0,
            y: 0.0,
            ..LayoutSettings::default()
        });
        layout.append(
            &[&self.font],
            &FontdueTextStyle::new(text.content.as_str(), RASTER_SIZE, 0),
        );

        let pixel_scale = [text.size[0] / RASTER_SIZE, text.size[1] / RASTER_SIZE];
        let mut result = Vec::with_capacity(layout.glyphs().len());
        let mut previous = None;
        let mut kerning_offset = 0.0;

        for glyph in layout.glyphs() {
            if glyph.parent == '\n' || glyph.parent == '\r' {
                previous = None;
                kerning_offset = 0.0;
                continue;
            }

            if let Some(previous) = previous {
                kerning_offset += self
                    .font
                    .horizontal_kern(previous, glyph.parent, RASTER_SIZE)
                    .unwrap_or(0.0);
            }
            previous = Some(glyph.parent);

            if glyph.width == 0 || glyph.height == 0 {
                continue;
            }

            let render_data = self.get_render_data(glyph.parent);
            let glyph_size = [
                glyph.width as f32 * pixel_scale[0],
                glyph.height as f32 * pixel_scale[1],
            ];
            let position = cgmath::Vector3 {
                x: text.position[0]
                    + (glyph.x + kerning_offset + glyph.width as f32 * 0.5) * pixel_scale[0],
                y: text.position[1] - (glyph.y + glyph.height as f32 * 0.5) * pixel_scale[1],
                z: text.position[2],
            };
            let model = (cgmath::Matrix4::from_translation(position)
                * cgmath::Matrix4::from_nonuniform_scale(glyph_size[0], glyph_size[1], 1.0))
            .into();

            result.push(ColorSpriteInstanceRaw {
                uv: render_data.uv,
                model,
                color: text.color,
            });
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn manager() -> FontManager {
        let mut manager = FontManager::default();
        manager.font_rasterize(RASTER_SIZE);
        manager
    }

    fn text(content: &str) -> TextRenderData {
        TextRenderData {
            content: Arc::new(content.to_string()),
            color: [1.0, 1.0, 1.0],
            position: [10.0, 20.0, 0.5],
            size: [RASTER_SIZE, RASTER_SIZE],
        }
    }

    fn translation(instance: &ColorSpriteInstanceRaw) -> [f32; 2] {
        [instance.model[3][0], instance.model[3][1]]
    }

    fn scale(instance: &ColorSpriteInstanceRaw) -> [f32; 2] {
        [instance.model[0][0], instance.model[1][1]]
    }

    #[test]
    fn applies_pair_kerning() {
        let manager = manager();
        let instances = manager.make_instance_buffer(&text("AV"));
        let a_metrics = manager.font.metrics('A', RASTER_SIZE);
        let v_metrics = manager.font.metrics('V', RASTER_SIZE);
        let kern = manager
            .font
            .horizontal_kern('A', 'V', RASTER_SIZE)
            .unwrap_or(0.0);
        let expected_v_center = 10.0
            + a_metrics.advance_width.ceil()
            + v_metrics.bounds.xmin.floor()
            + kern
            + v_metrics.width as f32 * 0.5;

        assert_eq!(instances.len(), 2);
        assert!((translation(&instances[1])[0] - expected_v_center).abs() < 0.001);
    }

    #[test]
    fn uses_actual_glyph_bitmap_dimensions() {
        let manager = manager();
        let instances = manager.make_instance_buffer(&text("iW"));
        let i_size = scale(&instances[0]);
        let w_size = scale(&instances[1]);

        assert!(i_size[0] < w_size[0]);
        assert_eq!(
            i_size[0],
            manager.font.metrics('i', RASTER_SIZE).width as f32
        );
        assert_eq!(
            w_size[0],
            manager.font.metrics('W', RASTER_SIZE).width as f32
        );
    }

    #[test]
    fn aligns_capital_and_descender_to_the_same_baseline() {
        let manager = manager();
        let instances = manager.make_instance_buffer(&text("Ag"));
        let a_metrics = manager.font.metrics('A', RASTER_SIZE);
        let g_metrics = manager.font.metrics('g', RASTER_SIZE);
        let a_top = translation(&instances[0])[1] + scale(&instances[0])[1] * 0.5;
        let g_top = translation(&instances[1])[1] + scale(&instances[1])[1] * 0.5;
        let a_baseline = 20.0 - a_top + a_metrics.height as f32 + a_metrics.ymin as f32;
        let g_baseline = 20.0 - g_top + g_metrics.height as f32 + g_metrics.ymin as f32;

        assert!((a_baseline - g_baseline).abs() < 0.001);
    }

    #[test]
    fn uses_font_metrics_for_spaces_and_new_lines() {
        let manager = manager();
        let spaced = manager.make_instance_buffer(&text("A A"));
        let lines = manager.make_instance_buffer(&text("A\nA"));
        let space_advance = manager.font.metrics(' ', RASTER_SIZE).advance_width.ceil();
        let a_advance = manager.font.metrics('A', RASTER_SIZE).advance_width.ceil();
        let line_height = manager
            .font
            .horizontal_line_metrics(RASTER_SIZE)
            .unwrap()
            .new_line_size
            .ceil();

        assert!(
            (translation(&spaced[1])[0] - translation(&spaced[0])[0] - a_advance - space_advance)
                .abs()
                < 0.001
        );
        assert!((translation(&lines[0])[0] - translation(&lines[1])[0]).abs() < 0.001);
        assert!(
            (translation(&lines[0])[1] - translation(&lines[1])[1] - line_height).abs() < 0.001
        );
    }
}
