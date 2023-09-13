use minifb::{Key, ScaleMode, Window, WindowOptions};
extern crate pirates;
use pirates::{WIDTH, HEIGHT};

fn main() {
    let mut window = Window::new(
        "Simple Sailing Simulator",
        WIDTH,
        HEIGHT,
        WindowOptions {
            resize: true,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    )
    .expect("Unable to create window");

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut buffer: Vec<u32> = Vec::with_capacity(WIDTH * HEIGHT);

    let mut size = (0, 0);

    fn keyboard_input(key: u8) {
        unsafe {
            pirates::KEYCODE[0] = key;
            pirates::keyboard_input();
        }
    }

    while window.is_open() && !window.is_key_down(Key::Escape) {
        buffer.clear();
        unsafe {
            pirates::frame_entry();
            for pix in pirates::BUFFER {
                // AABBGGRR to 00RRGGBB 
                let arr: [u8; 4] = std::mem::transmute(pix);
                let arr = [arr[3],arr[0],arr[1],arr[2]];
                let pix: u32 = std::mem::transmute(arr);
                buffer.push(pix.swap_bytes());
            }
        }

        window.get_keys().iter().for_each(|key| match key {
            Key::A => keyboard_input(65),
            Key::D => keyboard_input(68),
            _ => (),
        });

        window.get_keys_released().iter().for_each(|key| match key {
            Key::W => println!("released w!"),
            Key::T => println!("released t!"),
            _ => (),
        });

        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
