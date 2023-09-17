#![cfg_attr(feature = "wasm", no_std)]

#[cfg(feature = "wasm")]
#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(feature = "wasm")]
extern crate wee_alloc;
#[cfg(feature = "wasm")]
#[cfg_attr(feature = "wasm", global_allocator)]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

extern crate alloc;
use alloc::vec::Vec;
use noise::PerlinBuf;
use num_traits::FromPrimitive;
use num_derive::FromPrimitive;
use core::sync::atomic::{AtomicU32, Ordering};
use libm;
extern crate nalgebra as na;
use nalgebra::Vector2;
use spin::Mutex;
#[cfg(feature = "rayon")]
use rayon::prelude::*;

mod sampler;
mod noise;

#[cfg(feature = "wasm")]
extern {
    fn blit_frame();
    fn blit_text(text: *const u8, len: u32, x: i32, y: i32, size: u8);
}

#[cfg(feature = "wasm")]
fn draw_text(text: &str, x: i32, y: i32, size: u8) {
    unsafe {
        blit_text(
            (*text).as_ptr(),
            text.len() as u32,
            x, y, size)
    }
}

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

#[no_mangle]
pub static mut LAST_FRAME_TIME: f32 = 0.0;

#[no_mangle]
pub static mut BUFFER: [u32; WIDTH * HEIGHT] = [0; WIDTH * HEIGHT];

// hack, js -> rust ffi in weird shape without wasm_bindgen
#[no_mangle]
pub static mut KEYCODE: [u8; 2] = [0; 2];

static INPUTS: Mutex<Vec<[u32; 2]>> = Mutex::new(Vec::new());

static FRAME: AtomicU32 = AtomicU32::new(0);

static mut RAND: noise::PerlinBuf = [0; 512];

const MAP_WIDTH: usize = 12;
const MAP_HEIGHT: usize = 10;
static MAP: [u8; MAP_WIDTH * MAP_HEIGHT] = [ // should deflate to smaller than bit-unpacking code would
    0,1,1,1,1,1,1,1,1,0,0,0,
    0,1,1,1,1,1,1,1,1,0,0,0,
    0,0,0,1,1,1,1,1,1,1,0,0,
    1,1,1,1,0,1,1,0,0,0,0,0,
    0,0,0,0,0,0,0,0,0,1,1,1,
    0,0,0,0,1,0,0,0,0,1,1,1,
    0,0,0,0,1,1,0,0,1,1,1,1,
    1,1,0,0,1,1,1,1,1,1,1,1,
    0,1,1,1,1,1,1,1,1,1,1,1,
    1,1,1,1,1,1,1,1,1,1,1,1,
];

static CAMERA: Mutex<[f32; 3]> = Mutex::new([0.0, 0.0, 0.18]);

static BOAT: Mutex<Boat> = Mutex::new(Boat {
    x: 0.0, y: 0.0,
    theta: 0.0,
    vel: 0.0,
    sail: 1.0,
});

#[no_mangle]
pub unsafe extern fn keyboard_input() {
    INPUTS.lock().push([KEYCODE[0] as u32, KEYCODE[1] as u32]);
}

#[no_mangle]
pub unsafe extern fn frame_entry() {
    // calling from multiple threads is ub
    render_frame(&mut BUFFER)
}

fn render_frame(buffer: &mut [u32; WIDTH*HEIGHT]) {
    let frame = FRAME.fetch_add(1, Ordering::Relaxed);

    if frame == 1 {
        unsafe {
            RAND = noise::generate(0xB00B5);
        }
    }
    let rand = unsafe {
        RAND
    };

    let mut camera = CAMERA.lock();
    let mut boat = BOAT.lock();

    let mut gain = unsafe { // very much an approximation of constant-velocity animation
        LAST_FRAME_TIME / (1000.0 / 20.0) // normalize to 20fps, cap simulation at 10fps
    };
    while let Some(key) = INPUTS.lock().pop() {
        use Input::*;
        match FromPrimitive::from_u32(key[0]).unwrap() { // [tag:input_handler]
            PanUp => camera[1] -= gain*10.0*camera[2], // up
            PanDown => camera[1] += gain*10.0*camera[2], // down
            PanLeft => camera[0] -= gain*10.0*camera[2], // left
            PanRight => camera[0] += gain*10.0*camera[2], // right
            191 => {
                *camera = [
                    noise::lerp(camera[0], 0.00, (gain * 0.25).min(1.0)),
                    noise::lerp(camera[1], 0.00, (gain * 0.25).min(1.0)),
                    noise::lerp(camera[2], 0.18, (gain * 0.25).min(1.0)),
                ]
            }, // reset camera (/)
            82 => boat.set_pos(Vector2::zeros()), // reset boat (r)
            ZoomIn => camera[2] *= 1.0 - 0.1*gain, // +
            ZoomOut => camera[2] *= 1.0 + 0.1*gain, // -
            RudderLeft => boat.theta -= gain*10.0, // A
            RudderRight => boat.theta += gain*10.0, // D
            AxisZoom => camera[2] *= 1.0 - (key[1] as f32 - 127.0) * 0.0004 * gain, // analog zoom
            AxisRudder => boat.theta += gain * (key[1] as f32 - 127.0) * 0.062, // analog rudder
            AxisPanY => camera[1] -= gain * (key[1] as f32 - 127.0) * 0.1 * camera[2], // pan[y]
            AxisPanX => camera[0] += gain * (key[1] as f32 - 127.0) * 0.1 * camera[2], // pan[x]
            5 => boat.sail += gain * (key[1] as f32 - 127.0) * 0.0013, // sail
            69 => boat.sail += gain * 0.062, // E
            81 => boat.sail -= gain * 0.062, // Q
        }
    } 
    boat.sail = boat.sail.clamp(0.0, 1.5);

    let wind = 0.0/RAD_TO_DEG;

    let step = 1.0; // 50ms
    while gain > 0.0 { // when draw fps is low, physics will still run at a minimum of 20fps
        gain -= step;
        let gain = gain + step;
        
        let vel = boat.get_velocity(wind);

        let depth = -sample_world(boat.get_pos()+HALF, rand);
        if depth < -0.04 {
            boat.vel = 0.0;
        } else if depth < 0.0 {
            boat.vel *= 1.0 + (depth * gain * 30.2).min(0.0);
        } 

        if depth > -0.04 {
            boat.vel = noise::lerp(boat.vel, vel * 0.82, 0.13 * gain);
            boat.go(gain);
        }
    }

    // draw sea
    const HALF: Vector2<f32> = Vector2::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);
    let camera_vec = Vector2::new(camera[0],camera[1]);

    #[cfg(feature = "rayon")]
    let buffer_iter = buffer.par_iter_mut();
    #[cfg(not(feature = "rayon"))]
    let buffer_iter = buffer.iter_mut();

    buffer_iter.enumerate().for_each(|pix| {
        let y = pix.0 / WIDTH;
        let x = pix.0 % WIDTH;
        let mut point = Vector2::new(x as f32, y as f32);
        point -= HALF;
        point *= camera[2];
        point += HALF;
        let n = sample_world(point+camera_vec+boat.get_pos(), rand);
        *pix.1 = 
        if n > 0.1 {
            let n = (n+0.1) * 300.0;
            0xFF00FF00 + (n as u32 + (n as u32) << 8)
        } else if n > 0.04 {
            let n = (0.1-n) * -300.0;
            0xFF44FF44 + (n as u32 + (n as u32) << 8)
        } else  {
            let n = (n+0.1) * 300.0;
            0xFFFF3333 + (n as u32 + (n as u32) << 8)
        }
    });

    // draw boat

    let rotation: f32 = boat.theta/RAD_TO_DEG;

    let cos = libm::cosf(rotation);
    let sin = libm::sinf(rotation);
    fn rotate(point: Vector2<f32>, cos: f32, sin: f32) -> Vector2<f32> {
        Vector2::new(
            point.x * cos - point.y * sin,
            point.x * sin + point.y * cos)
    }

    const SCALE: f32 = 2.0;

    let mut p1 = Vector2::new(-0.5, 0.7)* SCALE;
    let mut p2 = Vector2::new(0.5, 0.7)* SCALE;
    let mut p3 = Vector2::new(0.0, -0.7)* SCALE;
    p1 = (rotate(p1, cos, sin) - camera_vec) / camera[2];
    p2 = (rotate(p2, cos, sin) - camera_vec) / camera[2];
    p3 = (rotate(p3, cos, sin) - camera_vec) / camera[2];
    draw_tri(0xFF648CBA, buffer, p1+HALF, p2+HALF, p3+HALF);
    let mut p1 = Vector2::new(0.0, 0.5)* SCALE;
    let mut p2 = Vector2::new(0.4 * boat.get_intensity(wind), 0.6)* SCALE;
    let mut p3 = Vector2::new(0.0, -0.6)* SCALE;
    p1 = (rotate(p1, cos, sin) - camera_vec) / camera[2];
    p2 = (rotate(p2, cos, sin) - camera_vec) / camera[2];
    p3 = (rotate(p3, cos, sin) - camera_vec) / camera[2];
    draw_tri(0xFFDDDDDD, buffer, p1+HALF, p2+HALF, p3+HALF);


    #[cfg(feature = "wasm")]
    unsafe { blit_frame(); }

    //draw_text("hi from rust", 0,100,30);
}

fn draw_tri(color: u32, buffer: &mut [u32; WIDTH*HEIGHT], p1: Vector2<f32>, p2: Vector2<f32>, p3: Vector2<f32>) {
    let max_y = p1.y.max(p2.y).max(p3.y) as usize;
    let max_x = p1.x.max(p2.x).max(p3.x) as usize;
    let min_y = p1.y.min(p2.y).min(p3.y) as usize;
    let min_x = p1.x.min(p2.x).min(p3.x) as usize;

    fn sign(p1: Vector2<f32>, p2: Vector2<f32>, p3: Vector2<f32>) -> f32 {
        (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y)
    }

    for y in min_y.max(0)..max_y.min(HEIGHT) {
        for x in min_x.max(0)..max_x.min(WIDTH) {
            // https://stackoverflow.com/a/2049593
            let point = Vector2::new(x as f32, y as f32);

            let d1 = sign(point, p1, p2);
            let d2 = sign(point, p2, p3);
            let d3 = sign(point, p3, p1);

            let neg = (d1 < 0.0) || (d2 < 0.0) || (d3 < 0.0);
            let pos = (d1 > 0.0) || (d2 > 0.0) || (d3 > 0.0);
            if !(neg && pos) {
                buffer[y*WIDTH + x] = color;
            }
        }
    }
        
}

fn sample_world(point: Vector2<f32>, rand: PerlinBuf) -> f32 {
    let offset = Vector2::new(64492.0, 7892.0);
    let point = point + offset;
    let mut n = 0.0;
    n += (sampler::sample_map_inter(point / 64.0, &MAP)-0.5)* 0.6;
    n += noise::noise(point / 64.0, rand) / 1.0;
    n += noise::noise(point / 32.0, rand) / 2.0;
    n += noise::noise(point / 16.0, rand) / 4.0;
    n += noise::noise(point / 8.0, rand) / 8.0;
    n += noise::noise(point / 4.0, rand) / 16.0;
    n += noise::noise(point / 2.0, rand) / 32.0;
    n
}

const RAD_TO_DEG: f32 = 57.2058;
struct Boat {
    x: f32,
    y: f32,
    theta: f32,
    vel: f32,
    /// sail height 0-1
    sail: f32,
}

impl Boat {
    fn get_pos(self: &Self) -> Vector2<f32> {
        Vector2::new(self.x, self.y)
    }

    fn set_pos(self: &mut Self, pos: Vector2<f32>) {
       self.x = pos.x;
       self.y = pos.y;
    }

    fn get_intensity(self: &Self, wind_direction: f32) -> f32 {
        libm::sinf((self.theta-wind_direction)/RAD_TO_DEG) * self.sail
    }

    fn get_velocity(self: &Self, wind_direction: f32) -> f32 {
        libm::fabsf(self.get_intensity(wind_direction))
    }

    fn go(self: &mut Self, gain: f32) {
        let cos = libm::cosf((self.theta+ 45.0)/RAD_TO_DEG);
        let sin = libm::sinf((self.theta+ 45.0)/RAD_TO_DEG);
        let unit = Vector2::new(
            cos - sin,
            sin + cos);
        self.set_pos(self.get_pos() + unit * -self.vel * gain);
    }
    
}

#[derive(FromPrimitive)]
#[repr(u8)]
pub enum Input {
    /// Up Arrow
    PanUp = 38,
    /// Down Arrow
    PanDown = 40,
    /// Left Arrow
    PanLeft = 37,
    /// Right Arrow
    PanRight = 39,
    /// + 
    ZoomIn = 61,
    /// -
    ZoomOut = 173,
    /// A
    RudderLeft = 65,
    /// D
    RudderRight = 68,
    AxisZoom = 0,
    AxisRudder = 1, 
    AxisPanY = 3, 
    AxisPanX = 4, 
}
