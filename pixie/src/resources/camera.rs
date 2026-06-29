use cgmath::Point3;
use cgmath::SquareMatrix;

use super::RenderViewport;

const MIN_ORTHOGRAPHIC_HALF_HEIGHT: f32 = 0.1;
const MIN_PERSPECTIVE_FOV_Y: f32 = 1.0;
const MAX_PERSPECTIVE_FOV_Y: f32 = 179.0;

pub struct Camera {
    eye: Point3<f32>,
    target: Point3<f32>,
    up: cgmath::Vector3<f32>,

    aspect: f32,
    fov_y: f32,

    right: f32,
    top: f32,

    z_near: f32,
    z_far: f32,

    perspective: bool,
    // uniform: CameraUniform
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            // position the camera one unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 0.0, 15.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: 1.44,
            fov_y: 45.0,
            right: 0.0,
            top: 0.0,
            z_near: 0.1,
            z_far: 100.0,
            perspective: true,
            // uniform : CameraUniform::new(),
        }
    }
}

impl Camera {
    #[allow(unused)]
    pub fn init_perspective(aspect_ratio: f32) -> Self {
        Self {
            // position the camera one unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 0.0, 30.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: aspect_ratio,
            fov_y: 45.0,
            right: 0.0,
            top: 0.0,
            z_near: 0.1,
            z_far: 100.0,
            perspective: true,
            // uniform : CameraUniform::new(),
        }
    }

    pub fn init_orthographic(height: f32, aspect_ratio: f32) -> Self {
        let width = aspect_ratio * height;
        Self {
            // position the camera one unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 0.0, 30.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: 0.0,
            fov_y: 0.0,
            right: width,
            top: height,
            z_near: 0.0,
            z_far: 100.0,
            perspective: false,
            // uniform: CameraUniform::new(),
        }
    }

    #[allow(unused)]
    pub fn move_camera_delta(&mut self, delta: [f32; 2]) -> [f32; 2] {
        if delta[0] != 0. && delta[1] != 0. {
            let normalize = 0.447_213_6;
            self.eye.x += delta[0] * 2. * normalize;
            self.eye.y += delta[1] * normalize;
            self.target.x += delta[0] * 2. * normalize;
            self.target.y += delta[1] * normalize;
        } else {
            self.eye.x += delta[0];
            self.eye.y += delta[1];
            self.target.x += delta[0];
            self.target.y += delta[1];
        }

        [self.eye.x, self.eye.y]
    }

    #[allow(unused)]
    pub fn move_camera(&mut self, position: [f32; 2]) -> [f32; 2] {
        self.eye.x = position[0];
        self.eye.y = position[1];
        self.target.x = position[0];
        self.target.y = position[1];

        [self.eye.x, self.eye.y]
    }

    #[allow(unused)]
    pub fn set_zoom(&mut self, height: f32) {
        let height = height.max(MIN_ORTHOGRAPHIC_HALF_HEIGHT);
        let width = if self.aspect != 0.0 {
            self.aspect * height
        } else {
            // For orthographic cameras, recalculate width based on current aspect ratio
            let current_aspect = self.right / self.top;
            current_aspect * height
        };
        self.right = width;
        self.top = height;
    }

    pub fn zoom(&self) -> f32 {
        if self.perspective {
            self.fov_y
        } else {
            self.top
        }
    }

    pub fn zoom_in(&mut self, factor: f32) {
        self.zoom_by(1.0 / factor);
    }

    pub fn zoom_out(&mut self, factor: f32) {
        self.zoom_by(factor);
    }

    pub fn zoom_by(&mut self, factor: f32) {
        let factor = factor.max(f32::EPSILON);
        if self.perspective {
            self.fov_y = (self.fov_y * factor).clamp(MIN_PERSPECTIVE_FOV_Y, MAX_PERSPECTIVE_FOV_Y);
        } else {
            self.set_zoom(self.top * factor);
        }
    }

    pub fn get_view_proj(&self) -> [[f32; 4]; 4] {
        let vp = self.build_view_projection_matrix();
        vp.into()
    }

    pub fn screen_to_world(
        &self,
        screen_position: [f32; 2],
        viewport: RenderViewport,
    ) -> Option<[f32; 2]> {
        if self.perspective
            || viewport.width <= 0.0
            || viewport.height <= 0.0
            || !viewport.contains(screen_position)
        {
            return None;
        }

        let ndc = cgmath::Vector4::new(
            (screen_position[0] - viewport.x) / viewport.width * 2.0 - 1.0,
            1.0 - (screen_position[1] - viewport.y) / viewport.height * 2.0,
            0.0,
            1.0,
        );
        let inverse = self.build_view_projection_matrix().invert()?;
        let world = inverse * ndc;
        if world.w.abs() <= f32::EPSILON {
            return None;
        }

        Some([world.x / world.w, world.y / world.w])
    }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // 1.
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        // 2.
        if self.perspective {
            let proj = cgmath::perspective(
                cgmath::Deg(self.fov_y),
                self.aspect,
                self.z_near,
                self.z_far,
            );
            OPENGL_TO_WGPU_MATRIX * proj * view
        } else {
            let proj = cgmath::ortho(
                -self.right,
                self.right,
                -self.top,
                self.top,
                self.z_near,
                self.z_far,
            );
            OPENGL_TO_WGPU_MATRIX * proj * view
        }
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_position_close(actual: Option<[f32; 2]>, expected: [f32; 2]) {
        let actual = actual.expect("expected a world position");
        assert!((actual[0] - expected[0]).abs() < 0.0001);
        assert!((actual[1] - expected[1]).abs() < 0.0001);
    }

    #[test]
    fn orthographic_zoom_in_reduces_visible_height() {
        let mut camera = Camera::init_orthographic(20.0, 16.0 / 9.0);

        camera.zoom_in(2.0);

        assert_eq!(camera.zoom(), 10.0);
    }

    #[test]
    fn orthographic_zoom_out_increases_visible_height() {
        let mut camera = Camera::init_orthographic(20.0, 16.0 / 9.0);

        camera.zoom_out(2.0);

        assert_eq!(camera.zoom(), 40.0);
    }

    #[test]
    fn orthographic_zoom_is_clamped_above_zero() {
        let mut camera = Camera::init_orthographic(20.0, 16.0 / 9.0);

        camera.set_zoom(0.0);

        assert_eq!(camera.zoom(), MIN_ORTHOGRAPHIC_HALF_HEIGHT);
    }

    #[test]
    fn perspective_zoom_is_clamped_to_valid_fov() {
        let mut camera = Camera::init_perspective(16.0 / 9.0);

        camera.zoom_in(1_000.0);
        assert_eq!(camera.zoom(), MIN_PERSPECTIVE_FOV_Y);

        camera.zoom_out(1_000.0);
        assert_eq!(camera.zoom(), MAX_PERSPECTIVE_FOV_Y);
    }

    #[test]
    fn screen_to_world_uses_letterboxed_viewport() {
        let camera = Camera::init_orthographic(10.0, 2.0);
        let viewport = RenderViewport::new(100.0, 50.0, 800.0, 400.0);

        assert_eq!(
            camera.screen_to_world([500.0, 250.0], viewport),
            Some([0.0, 0.0])
        );
        assert_eq!(
            camera.screen_to_world([100.0, 50.0], viewport),
            Some([-20.0, 10.0])
        );
        assert_eq!(camera.screen_to_world([50.0, 250.0], viewport), None);
    }

    #[test]
    fn screen_to_world_accounts_for_camera_position() {
        let mut camera = Camera::init_orthographic(10.0, 1.0);
        camera.move_camera([4.0, -3.0]);
        let viewport = RenderViewport::new(0.0, 0.0, 200.0, 200.0);

        assert_position_close(
            camera.screen_to_world([100.0, 100.0], viewport),
            [4.0, -3.0],
        );
    }
}
