#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DebugLine {
    pub start: [f32; 3],
    pub end: [f32; 3],
    pub color: [f32; 4],
    pub thickness: f32,
}

impl DebugLine {
    pub fn new(start: [f32; 3], end: [f32; 3], color: [f32; 4], thickness: f32) -> Self {
        assert!(
            start.into_iter().chain(end).all(f32::is_finite),
            "debug line positions must be finite"
        );
        assert!(
            color.into_iter().all(f32::is_finite),
            "debug line color must be finite"
        );
        assert!(
            thickness.is_finite() && thickness > 0.0,
            "debug line thickness must be finite and positive"
        );

        Self {
            start,
            end,
            color,
            thickness,
        }
    }
}

#[derive(Debug, Default)]
pub struct DebugDraw {
    lines: Vec<DebugLine>,
}

impl DebugDraw {
    pub fn with_capacity(line_count: usize) -> Self {
        Self {
            lines: Vec::with_capacity(line_count),
        }
    }

    pub fn line(&mut self, start: [f32; 3], end: [f32; 3], color: [f32; 4], thickness: f32) {
        self.lines
            .push(DebugLine::new(start, end, color, thickness));
    }

    pub fn lines(&self) -> &[DebugLine] {
        &self.lines
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn submissions_are_cleared_without_losing_capacity() {
        let mut debug_draw = DebugDraw::with_capacity(4);
        debug_draw.line([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 0.0, 0.0, 1.0], 0.1);
        let capacity = debug_draw.lines.capacity();

        debug_draw.clear();

        assert!(debug_draw.lines().is_empty());
        assert_eq!(debug_draw.lines.capacity(), capacity);
    }

    #[test]
    #[should_panic(expected = "thickness must be finite and positive")]
    fn rejects_zero_thickness() {
        DebugLine::new([0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [1.0, 1.0, 1.0, 1.0], 0.0);
    }
}
