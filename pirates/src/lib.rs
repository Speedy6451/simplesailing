#![no_std]

#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate wee_alloc;
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use lazy_static::lazy_static;
use noise::PerlinBuf;
use core::sync::atomic::{AtomicU32, Ordering};
use libm::{self, Libm};
extern crate nalgebra as na;
use nalgebra::{Vector2};
use thingbuf::mpsc::{self, errors::TryRecvError};
use spin::Mutex;
use keycode::KeyMap;

mod sampler;
mod noise;

extern {
    fn blit_frame();
    fn blit_text(text: *const u8, len: u32, x: i32, y: i32, size: u8);
}

fn draw_text(text: &str, x: i32, y: i32, size: u8) {
    unsafe {
        blit_text(
            (*text).as_ptr(),
            text.len() as u32,
            x, y, size)
    }
}

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

#[no_mangle]
static mut BUFFER: [u32; WIDTH * HEIGHT] = [0; WIDTH * HEIGHT];

// hack, js -> rust ffi in weird shape without wasm_bindgen
#[no_mangle]
static mut KEYCODE: [u8; 2] = [0; 2];

static LAST_KEY: Mutex<Option<[u32; 2]>> = Mutex::new(None);

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

static CAMERA: Mutex<[f32; 3]> = Mutex::new([0.0, 0.0, 4.0]);

#[no_mangle]
pub unsafe extern fn keyboard_input() {
    let keycode = KEYCODE[0];
    LAST_KEY.lock().insert([keycode as u32, 0]);
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

    //camera[0] += 1.0;

    if let Some(key) = LAST_KEY.lock().take() {
        match key[0] {
            38 => camera[1] -= 10.0, // up
            40 => camera[1] += 10.0, // down
            37 => camera[0] -= 10.0, // left
            39 => camera[0] += 10.0, // right
            61 => camera[2] *= 0.9, // +
            173 => camera[2] *= 1.1, // -
            _ => {}
        }
    } 

    let camera_vec = Vector2::new(camera[0],camera[1]);

    const half: Vector2<f32> = Vector2::new(WIDTH as f32 / 2.0, HEIGHT as f32 / 2.0);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let mut point = Vector2::new(x as f32, y as f32);
            point -= half;
            point *= camera[2];
            point += half;
            let n = sample_world(point+camera_vec, rand);
            buffer[y*WIDTH + x] = 
            if n > 0.1 {
                0xFF00FF00
            } else if n > 0.04 {
                0xFF44FF44
            } else if n > -0.03 {
                0xFFFF1111 
            } else {
                0xFFFF0000
            }
        }
    }
    unsafe { blit_frame(); }

    //draw_text("hi from rust", 0,100,30);
}

fn sample_world(point: Vector2<f32>, rand: PerlinBuf) -> f32 {
    let offset = Vector2::new(64480.0, 7870.0);
    //240.0,240.0
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

#[repr(u32)]
enum Keys {
    Up = 38,
    Down = 40,
    Left = 37,
    Right = 39
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = number();
        assert_eq!(result, 64);
    }
}

