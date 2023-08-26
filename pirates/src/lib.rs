#![no_std]

use core::sync::atomic::{AtomicU32, Ordering};

#[panic_handler]
fn handle_panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

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

#[no_mangle]
pub unsafe extern fn frame_entry() {
    // calling from multiple threads is ub
    render_frame(&mut BUFFER)
}

fn render_frame(buffer: &mut [u32; WIDTH*HEIGHT]) {
    let frame = FRAME.fetch_add(1, Ordering::Relaxed);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            buffer[y*WIDTH + x] = frame.wrapping_add((x^y) as u32) | 0xFF000000;
        }
    }
    unsafe { blit_frame(); }

    draw_text("hi from rust", 0,100,1);
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
