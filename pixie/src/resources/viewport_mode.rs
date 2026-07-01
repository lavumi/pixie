use super::RenderViewport;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub enum ViewportMode {
    /// Preserve the startup aspect ratio and letterbox the remaining window area.
    #[default]
    FixedAspect,
    /// Use the full window and update the camera aspect ratio on resize.
    Expand,
}

impl ViewportMode {
    pub fn viewport(
        self,
        window_width: u32,
        window_height: u32,
        target_aspect: f32,
    ) -> RenderViewport {
        match self {
            Self::FixedAspect => RenderViewport::fit(window_width, window_height, target_aspect),
            Self::Expand => {
                RenderViewport::new(0.0, 0.0, window_width as f32, window_height as f32)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_aspect_is_default_and_adds_letterbox() {
        let mode = ViewportMode::default();

        assert_eq!(mode, ViewportMode::FixedAspect);
        assert_eq!(
            mode.viewport(1600, 900, 4.0 / 3.0),
            RenderViewport::new(200.0, 0.0, 1200.0, 900.0)
        );
    }

    #[test]
    fn expand_uses_entire_window() {
        assert_eq!(
            ViewportMode::Expand.viewport(1600, 900, 4.0 / 3.0),
            RenderViewport::new(0.0, 0.0, 1600.0, 900.0)
        );
    }
}
