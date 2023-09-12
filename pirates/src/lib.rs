#![no_std]

#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

extern crate wee_alloc;
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use core::sync::atomic::{AtomicU32, Ordering};
use libm::{self, Libm};
extern crate nalgebra as na;
use nalgebra::{Vector2};

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

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let point = Vector2::new(x as f32, y as f32)* 4.0;
            let offset = Vector2::new(64120.0, 7320.0);
            let point = point + offset;
            let mut n = 0.0;
            n += (sampler::sample_map_inter(point / 64.0, &MAP)-0.5)* 0.6;
            n += noise::noise(point / 64.0, rand) / 1.0;
            n += noise::noise(point / 32.0, rand) / 2.0;
            n += noise::noise(point / 16.0, rand) / 4.0;
            n += noise::noise(point / 8.0, rand) / 8.0;
            n += noise::noise(point / 4.0, rand) / 16.0;
            n += noise::noise(point / 2.0, rand) / 32.0;
            //buffer[y*WIDTH + x] = (((n*0.5+0.5)*256.0) as u32) << 16| 0xFF005000;
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = number();
        assert_eq!(result, 64);
    }
}

