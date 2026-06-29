#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderViewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl RenderViewport {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn fit(window_width: u32, window_height: u32, target_aspect: f32) -> Self {
        let window_width = window_width as f32;
        let window_height = window_height as f32;
        if window_width <= 0.0 || window_height <= 0.0 || target_aspect <= 0.0 {
            return Self::new(0.0, 0.0, window_width, window_height);
        }

        let window_aspect = window_width / window_height;
        if window_aspect > target_aspect {
            let width = window_height * target_aspect;
            Self::new((window_width - width) * 0.5, 0.0, width, window_height)
        } else {
            let height = window_width / target_aspect;
            Self::new(0.0, (window_height - height) * 0.5, window_width, height)
        }
    }

    pub fn contains(&self, position: [f32; 2]) -> bool {
        position[0] >= self.x
            && position[0] <= self.x + self.width
            && position[1] >= self.y
            && position[1] <= self.y + self.height
    }

    pub(crate) fn wgpu_data(&self) -> [f32; 6] {
        [self.x, self.y, self.width, self.height, 0.0, 1.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_centers_horizontal_letterbox() {
        let viewport = RenderViewport::fit(1600, 900, 4.0 / 3.0);

        assert_eq!(viewport, RenderViewport::new(200.0, 0.0, 1200.0, 900.0));
    }

    #[test]
    fn fit_centers_vertical_letterbox() {
        let viewport = RenderViewport::fit(900, 1600, 4.0 / 3.0);

        assert_eq!(viewport, RenderViewport::new(0.0, 462.5, 900.0, 675.0));
    }
}
