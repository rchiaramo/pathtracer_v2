const EPSILON = 0.001f;
const PI = 3.1415927f;
const FRAC_1_PI = 0.31830987f;
const FRAC_PI_2 = 1.5707964f;

struct FrameBuffer {
    width: u32,
    height: u32,
    frame: u32,
    accumulated_samples: u32
}

struct ProjectionBuffer {
    invProj: mat4x4<f32>
}

struct ViewBuffer {
    view: mat4x4<f32>
}

struct SamplingParametersBuffer {
    samples_per_frame: u32,
    samples_per_pixel: u32,
    number_of_bounces: u32,
    clear_image_buffer: u32,
}

struct CameraBuffer {
    position: vec4<f32>,
    defocus_radius: f32,
    focus_distance: f32,
}

@group(0) @binding(0) var<storage, read_write> image_buffer: array<array<f32, 3>>;
@group(0) @binding(1) var<uniform> frame_buffer: FrameBuffer;
@group(1) @binding(0) var<uniform> inv_projection_matrix: ProjectionBuffer;
@group(1) @binding(1) var<uniform> view_matrix: ViewBuffer;
@group(1) @binding(2) var<uniform> sampling_parameters: SamplingParametersBuffer;
@group(1) @binding(3) var<uniform> camera: CameraBuffer;

@compute @workgroup_size(4,4,1)
fn main(@builtin(global_invocation_id) id: vec3u) {

    let image_size = vec2(frame_buffer.width, frame_buffer.height);
    let screen_pos = id.xy;
    let idx = id.x + id.y * image_size.x;

    // load the stored pixel color
    var pixel_color: vec3f = vec3f(image_buffer[idx][0], image_buffer[idx][1], image_buffer[idx][2]);
    var rng_state:u32 = initRng(screen_pos, image_size, frame_buffer.frame);

    image_buffer[idx][0] = 0.5; // pixel_color.x;
    image_buffer[idx][1] = 0.1; //pixel_color.y;
    image_buffer[idx][2] = 0.1; //pixel_color.z;
}

fn rngNextInUnitHemisphere(state: ptr<function, u32>) -> vec3<f32> {
    let r1 = rngNextFloat(state);
    let r2 = rngNextFloat(state);

    let phi = 2.0 * PI * r1;
    let sinTheta = sqrt(1.0 - r2 * r2);

    let x = cos(phi) * sinTheta;
    let y = sin(phi) * sinTheta;
    let z = r2;

    return vec3(x, y, z);
}

fn rngNextVec3InUnitDisk(state: ptr<function, u32>) -> vec3<f32> {
    // r^2 is distributed as U(0, 1).
    let r = sqrt(rngNextFloat(state));
    let alpha = 2.0 * PI * rngNextFloat(state);

    let x = r * cos(alpha);
    let y = r * sin(alpha);

    return vec3(x, y, 0.0);
}

fn rngNextVec3InUnitSphere(state: ptr<function, u32>) -> vec3<f32> {
    // probability density is uniformly distributed over r^3
    let r = pow(rngNextFloat(state), 0.33333f);
    // and need to distribute theta according to arccos(U[-1,1])
    // let theta = acos(1.0 - 2f * rngNextFloat(state));
    let cosTheta = 1f - 2f * rngNextFloat(state);
    let sinTheta = sqrt(1 - cosTheta * cosTheta);
    let phi = 2.0 * PI * rngNextFloat(state);

    let x = r * sinTheta * cos(phi);
    let y = r * sinTheta * sin(phi);
    let z = r * cosTheta;

    return vec3(x, y, z);
}

fn rngNextUintInRange(state: ptr<function, u32>, min: u32, max: u32) -> u32 {
    let next_int = rngNextInt(state);
    return min + (next_int) % (max - min);
}

fn rngNextFloat(state: ptr<function, u32>) -> f32 {
    let next_int = rngNextInt(state);
    return f32(next_int) * 2.3283064365387e-10f;  // / f32(0xffffffffu - 1f);
}

fn initRng(pixel: vec2<u32>, resolution: vec2<u32>, frame: u32) -> u32 {
    let seed = dot(pixel, vec2<u32>(1u, resolution.x)) ^ jenkinsHash(frame);
    return jenkinsHash(seed);
}

// I've altered the code I copied to implement what I believe is now a correct
// PCG-RXS-M-XS; specifically, the state is only based on the LCG
// rngNextInt will update the state, but then return a rng via the output function
fn rngNextInt(state: ptr<function, u32>) -> u32 {
    // PCG hash RXS-M-XS
    let oldState = *state * 747796405u + 2891336453u;
    *state = oldState;
    let word = ((oldState >> ((oldState >> 28u) + 4u)) ^ oldState) * 277803737u;
    return (word >> 22u) ^ word;
}

fn advance(state: ptr<function, u32>, advance_by: u32) {
    var acc_mult = 1u;
    var acc_plus = 0u;
    var cur_mult = 747796405u;
    var cur_plus = 2891336453u;
    var delta = advance_by;
    while delta > 0 {
        if delta == 1 {
            acc_mult *= cur_mult;
            acc_plus = acc_plus * cur_mult + cur_plus;
        }
        cur_plus = (cur_mult + 1u) * cur_plus;
        cur_mult *= cur_mult;
        delta = delta >> 1;
    }
    *state = *state * acc_mult + acc_plus;
}

fn jenkinsHash(input: u32) -> u32 {
    var x = input;
    x += x << 10u;
    x ^= x >> 6u;
    x += x << 3u;
    x ^= x >> 11u;
    x += x << 15u;
    return x;
}