use cgmath::Point3;

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

    perspective : bool,

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
    pub fn init_perspective(aspect_ratio : f32)-> Self {
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

    pub fn init_orthographic(height: u32, aspect_ratio: f32) -> Self {
        let height = height as f32;
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
    pub fn move_camera_delta(&mut self, delta: [f32;2]) -> [f32;2]{
        if delta[0] != 0. && delta[1] != 0. {
            let normalize = 0.4472135955;
            self.eye.x += delta[0] * 2. * normalize;
            self.eye.y += delta[1] * normalize;
            self.target.x += delta[0] * 2. * normalize;
            self.target.y += delta[1] * normalize;
        }
        else {
            self.eye.x += delta[0];
            self.eye.y += delta[1];
            self.target.x += delta[0];
            self.target.y += delta[1];
        }


        [self.eye.x, self.eye.y]
    }

    #[allow(unused)]
    pub fn move_camera(&mut self, position: [f32;2]) -> [f32;2]{

        self.eye.x = position[0];
        self.eye.y = position[1];
        self.target.x = position[0];
        self.target.y = position[1];

        [self.eye.x, self.eye.y]
    }

    pub fn get_view_proj(&self) -> [[f32; 4]; 4]{
        let vp = self.build_view_projection_matrix();
        vp.into()
    }

    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        // 1.
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        // 2.
        if self.perspective {
            let proj = cgmath::perspective(cgmath::Deg(self.fov_y), self.aspect, self.z_near, self.z_far);
            OPENGL_TO_WGPU_MATRIX * proj * view

        }
        else {
            let proj = cgmath::ortho(-self.right, self.right,-self.top, self.top, self.z_near, self.z_far);
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
