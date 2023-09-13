use minifb::{Key, ScaleMode, Window, WindowOptions, Scale};
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
            scale: Scale::X4,
            ..WindowOptions::default()
        },
    )
    .expect("Unable to create window");

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut buffer: Vec<u32> = Vec::with_capacity(WIDTH * HEIGHT);

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
            Key::Equal => keyboard_input(61),
            Key::Minus => keyboard_input(173),
            Key::Up => keyboard_input(38),
            Key::Down => keyboard_input(40),
            Key::Left => keyboard_input(37),
            Key::Right => keyboard_input(39),
            _ => (),
        });

        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
