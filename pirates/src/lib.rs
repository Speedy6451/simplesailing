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

const WIDTH: usize = 80;
const HEIGHT: usize = 60;

#[no_mangle]
static mut BUFFER: [u32; WIDTH * HEIGHT] = [0; WIDTH * HEIGHT];

static FRAME: AtomicU32 = AtomicU32::new(0);

static mut RAND: noise::PerlinBuf = [0; 512];

#[no_mangle]
pub unsafe extern fn frame_entry() {
    // calling from multiple threads is ub
    render_frame(&mut BUFFER)
}

fn render_frame(buffer: &mut [u32; WIDTH*HEIGHT]) {
    let frame = FRAME.fetch_add(1, Ordering::Relaxed);

    if frame == 1 {
        unsafe {
            RAND = noise::generate();
        }
    }
    let rand = unsafe {
        RAND
    };

    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            let point = Vector2::new((x+frame as usize) as f32,y as f32)*100.0;
            let mut n = noise::noise(point / 600.0, rand) * 2.0;
            n += noise::noise(point / 300.0, rand) * 1.0;
            n += noise::noise(point / 150.0, rand) * 0.5;
            n += noise::noise(point / 75.0, rand) * 0.25;
            n += noise::noise(point / 37.5, rand) * 0.125;
            buffer[y*WIDTH + x] = (((n*0.5+0.5)*256.0) as u32) << 16| 0xFF005000;
        }
    }
    unsafe { blit_frame(); }

    draw_text("hi from rust", 0,100,3);
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

    pub fn generate() -> PerlinBuf {
        let mut rand = 42424242;
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
        let two = b[x*width + y + 256] as f32;

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
