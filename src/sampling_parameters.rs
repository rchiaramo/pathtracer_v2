use crate::gui::UserInput;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GPUSamplingParametersBuffer {
    samples_per_frame: u32,
    pub(crate) samples_per_pixel: u32,
    number_of_bounces: u32,
    clear_image_buffer: u32,
}

impl GPUSamplingParametersBuffer {
    pub fn new(samples_per_frame: u32, samples_per_pixel: u32, number_of_bounces: u32) -> Self {
        Self {
            samples_per_frame,
            samples_per_pixel,
            number_of_bounces,
            clear_image_buffer: 1,
        }
    }
    
    pub fn process_user_input(&mut self, user_input: &mut UserInput) {
        self.samples_per_frame = user_input.samples_per_frame();
        self.samples_per_pixel = user_input.samples_per_pixel();
        self.number_of_bounces = user_input.number_of_bounces();
    }
    
    pub fn set_clear_image_flag(&mut self, clear: bool) {
        if clear {
            self.clear_image_buffer = 1;
        } else {
            self.clear_image_buffer = 0;
        }
    }
    
    pub fn samples_per_frame(&self) -> u32 {
        self.samples_per_frame
    }
    
    pub fn samples_per_pixel(&self) -> u32 {
        self.samples_per_pixel
    }
}