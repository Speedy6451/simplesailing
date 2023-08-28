#![no_std]

#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}


use core::sync::atomic::{AtomicU32, Ordering};
use libm::{self, Libm};
extern crate nalgebra as na;
use nalgebra::{Vector2};

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

const WIDTH: usize = 256;
const HEIGHT: usize = 224;

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
            n += (sample_map_inter(point / 64.0, &MAP)-0.5)* 0.6;
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

fn sample_map_inter(point: Vector2<f32>, map: &[u8]) -> f32 {
    let x = libm::floorf(point.x) as usize % MAP_WIDTH;
    let y = libm::floorf(point.y) as usize % MAP_HEIGHT;
    let p0 = Vector2::new(libm::floorf(point.x), libm::floorf(point.y));
    let p1 = p0 + Vector2::new(1.0, 0.0);
    let p2 = p0 + Vector2::new(0.0, 1.0);
    let p3 = p0 + Vector2::new(1.0, 1.0);

    let tx = point.x - libm::floorf(point.x);
    let ty = point.y - libm::floorf(point.y);

    let top = (1.0 - tx) * sample_map(p0, map) as f32 + tx * sample_map(p1, map) as f32;
    let bot = (1.0 - tx) * sample_map(p2, map) as f32 + tx * sample_map(p3, map) as f32;

    (1.0-ty) * top + ty * bot
}

fn sample_map(point: Vector2<f32>, map: &[u8]) -> u8 {
    const MARGIN: usize = 3;
    let x = libm::floorf(point.x) as usize % (MAP_WIDTH + MARGIN);
    let y = libm::floorf(point.y) as usize % (MAP_HEIGHT + MARGIN);

    if x >= MAP_WIDTH || y >= MAP_HEIGHT { 
        if x-y > 12 {
            return 1;
        }
        return 0; 
    }

    map[y*MAP_WIDTH + x]

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

mod noise {
    use libm::Libm;
    use nalgebra::Vector2;

    pub type PerlinBuf = [u32; 512];

    pub fn generate(seed: u32) -> PerlinBuf {
        let mut rand = seed;
        let mut data: PerlinBuf = [0; 512];
        for item in data.iter_mut() {
            rand = xorshift(rand);
            *item = rand;
        }

        data
    }

    fn xorshift(state: u32) -> u32 {
        // impl from wikipedia
        let mut state = state;
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        state
    }

    fn fade(t: f32) -> f32 {
        t*t*t*(t*(t*6.0-15.0)+10.0)
    }

    fn lerp(a: f32, b:f32, d:f32) -> f32 {
        a * (1.0-d) + b * d
    }

    fn grad(p: Vector2<f32>, b: PerlinBuf) -> Vector2<f32> {
        const width: usize = 16;


        let x = p.x as usize % width;
        let y = p.y as usize % width;

        let one = b[x*width + y] as f32;
        let two = b[(x*width + y + 1)%512] as f32;

        Vector2::new(one, two).normalize()
    }

    pub fn noise(p: Vector2<f32>, b: PerlinBuf) -> f32 {
        let p0 = Vector2::new(libm::floorf(p.x), libm::floorf(p.y));
        let p1 = p0 + Vector2::new(1.0, 0.0);
        let p2 = p0 + Vector2::new(0.0, 1.0);
        let p3 = p0 + Vector2::new(1.0, 1.0);

        let g0 = grad(p0, b);
        let g1 = grad(p1, b);
        let g2 = grad(p2, b);
        let g3 = grad(p3, b);

        let tx = p.x - p0.x;
        let ftx = fade(tx);
        let ty = p.y - p0.y;
        let fty = fade(ty);

        let p0p1 = (1.0 - ftx) * g0.dot(&(p-p0)) + ftx * g1.dot(&(p-p1));
        let p2p3 = (1.0 - ftx) * g2.dot(&(p-p2)) + ftx * g3.dot(&(p-p3));


        (1.0 - fty) * p0p1 + fty * p2p3
    }
}
