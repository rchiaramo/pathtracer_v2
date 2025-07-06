use std::f32::consts::PI;
use glam::{Mat4, Vec3, Vec4};
use crate::gui::UserInput;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GPUCamera {
    camera_position: Vec4,
    defocus_radius: f32,
    focus_distance: f32,
    buffering: [f32; 2],
}

impl GPUCamera {
    pub fn new(camera_position: Vec3, defocus_angle_rad: f32, focus_distance: f32) -> GPUCamera {
        let defocus_radius = focus_distance * (0.5 * defocus_angle_rad).tan();

        GPUCamera {
            camera_position: camera_position.extend(0.0),
            defocus_radius,
            focus_distance,
            buffering: [0.0; 2],
        }
    }
}

pub struct CameraController {
    position: Vec3,
    pitch: f32,
    yaw: f32,
    vfov_rad: f32,
    defocus_angle_rad: f32,
    focus_distance: f32,
    z_near: f32,
    z_far: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_right: f32,
    amount_left: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    speed: f32,
    sensitivity: f32,
    updated: bool,
}

impl CameraController {
    const SAFE_FRAC_PI:f32 = PI - 0.001;

    pub fn new(look_from: Vec3, look_at: Vec3, vfov: f32, defocus_angle: f32, focus_distance: f32,
               z_near:f32, z_far: f32, speed: f32, sensitivity: f32) -> Self {
        
        let position = look_from;
        let forwards = (look_at - position).normalize();

        let pitch = forwards.y.acos();
        let yaw = forwards.x.atan2(-forwards.z);
        
        Self {
            position,
            pitch,
            yaw,
            vfov_rad: vfov.to_radians(),
            defocus_angle_rad: defocus_angle.to_radians(),
            focus_distance,
            z_near,
            z_far,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_right: 0.0,
            amount_left: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            speed,
            sensitivity,
            updated: true
        }
    }
    
    pub fn updated(&self) -> bool { self.updated }
    
    pub fn reset(&mut self) { self.updated = false; }
    
    pub fn process_user_input(&mut self, input: &mut UserInput) {
        // process keyboard
        let key = input.key();
        let is_pressed = input.key_pressed();
        let is_released = input.key_released();
        match key {
            imgui::Key::W => {
                if is_pressed { self.amount_forward = 1.0; }
                if is_released { self.amount_forward = 0.0; }
            },
            imgui::Key::A => {
                if is_pressed { self.amount_left = 1.0; }
                if is_released { self.amount_left = 0.0; }
            },
            imgui::Key::S => {
                if is_pressed { self.amount_backward = 1.0; }
                if is_released { self.amount_backward = 0.0; }
            },
            imgui::Key::D => {
                if is_pressed { self.amount_right = 1.0; }
                if is_released { self.amount_right = 0.0; }
            },
            imgui::Key::E => {
                if is_pressed { self.amount_up = 1.0; }
                if is_released { self.amount_up = 0.0; }
            },
            imgui::Key::Q => {
                if is_pressed { self.amount_down = 1.0; }
                if is_released { self.amount_down = 0.0; }
            },
            _ => {}
        }
        
        // process mouse drag when right button down
        self.rotate_horizontal = input.mouse_delta()[0];
        self.rotate_vertical = input.mouse_delta()[1];
        
        // process input from UI controls
        self.vfov_rad = input.vfov().to_radians();
        self.defocus_angle_rad = input.defocus_angle().to_radians();
        self.focus_distance = input.focus_distance();
        
        input.reset_state();
        self.updated = true;
    }

    pub fn update_camera(&mut self, dt: f32) {
        // Move forward/backward and left/right
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

        let forward = Vec3::new(-sin_yaw, 0.0, -cos_yaw);
        let right = Vec3::new(cos_yaw, 0.0, -sin_yaw);

        self.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        self.position += right * (self.amount_right - self.amount_left) * self.speed * dt;


        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        self.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        // Rotate
        self.yaw -= self.rotate_horizontal * self.sensitivity * dt;
        self.pitch -= self.rotate_vertical * self.sensitivity * dt;

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non-cardinal direction.
        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        if self.pitch < -Self::SAFE_FRAC_PI {
            self.pitch = -Self::SAFE_FRAC_PI;
        } else if self.pitch > Self::SAFE_FRAC_PI {
            self.pitch = Self::SAFE_FRAC_PI;
        }

        // after everything has been updated, set the updated flag back to false
        self.reset();
    }
    
    pub fn get_inv_projection_matrix(&self, aspect_ratio: f32) -> [[f32; 4]; 4] {
        // I will use the variables w, h, and r in standard fashion
        // w = h/AR where h = cot(fov/2); r = zfar/(znear - zfar)
        // P = [w, 0,  0,       0]
        //     [0, h,  0,       0]
        //     [0, 0,  r, r*znear]
        //     [0, 0, -1,       0]
        // This is the same matrix as glam::perspective_rh
        let h = 1.0 / (self.vfov_rad / 2.0).tan();
        let w = h / aspect_ratio;
        let r = self.z_far / (self.z_near - self.z_far);

        // for the raytracer I need the inverse of the projection matrix
        let p_inv = [
            [1.0/ w, 0.0, 0.0, 0.0],
            [0.0, 1.0 / h, 0.0, 0.0],
            [0.0, 0.0, 0.0, 1.0 / (r * self.z_near)],
            [0.0, 0.0, -1.0, 1.0 / self.z_near]
        ];

        p_inv
    }

    pub fn get_view_transform(&self) -> [[f32; 4]; 4]
    {
        // the view matrix is the world_from_camera transformation
        // the key "issue" that crops up and confuses everything is that the dir direction is
        // pointing in the -z_camera direction for a rh-coordinate system camera
        // the convention I like is pitch=90deg, yaw=0deg is pointing in -z_camera direction
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        let dir = Vec3::new(- sin_pitch * sin_yaw, cos_pitch, - sin_pitch * cos_yaw);
        let right = dir.cross(Vec3::new(0.0, 1.0, 0.0));
        let up = right.cross(dir);
        let center = self.position;

        let world_from_camera = [
            [right.x, right.y, right.z, 0.0],
            [up.x, up.y, up.z, 0.0],
            [dir.x, dir.y, dir.z, 0.0],
            [center.x, center.y, center.z, 1.0]
        ];

        // if wfc is of form T*R, then inv is inv(T)*inv(T), which is why we have the dot
        // product now in the fourth column
        // let camera_from_world = Mat4::from_cols(
        //     Vec4::new(-right.x, new_up.x, dir.x, 0.0),
        //     Vec4::new(-right.y, new_up.y, dir.y, 0.0),
        //     Vec4::new(-right.z, new_up.z, dir.z, 0.0),
        //     Vec4::new(center.dot(right), -center.dot(new_up), -center.dot(dir), 1.0)
        // );

        world_from_camera
    }
    
    pub fn get_gpu_camera(&self) -> GPUCamera {
        GPUCamera::new(self.position, self.defocus_angle_rad, self.focus_distance)
    }
}