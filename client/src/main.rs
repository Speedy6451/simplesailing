use std::time::SystemTime;
use minifb::{Key, ScaleMode, Window, WindowOptions, Scale};
extern crate pirates;
use pirates::{WIDTH, HEIGHT};
#[cfg(feature = "gilrs")]
use gilrs::{Axis, Gilrs, Button};

fn main() {
    #[cfg(feature = "gilrs")]
    let mut gilrs = Gilrs::new().unwrap();

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

    fn analog_input(chan: u8, val: f32) {
        let val = (val * 127.0) as i8;
        unsafe {
            pirates::KEYCODE[0] = chan;
            pirates::KEYCODE[1] = (val + 127) as u8;
            pirates::keyboard_input();
        }
    }

    #[cfg(feature = "gamepad")]
    let mut gamepad_handle = None;

    let mut frame_start: SystemTime = SystemTime::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let last_frame = frame_start.elapsed().unwrap().as_micros();
        frame_start = SystemTime::now();

        buffer.clear();
        unsafe {
            pirates::LAST_FRAME_TIME = last_frame as f32 / 1000.0;

            pirates::frame_entry();
            for pix in pirates::BUFFER {
                // AABBGGRR to 00RRGGBB 
                let arr: [u8; 4] = std::mem::transmute(pix);
                let arr = [arr[3],arr[0],arr[1],arr[2]];
                let pix: u32 = std::mem::transmute(arr);
                buffer.push(pix.swap_bytes());
            }
        }

        #[cfg(feature = "gamepad")]
        while let Some(event) = gilrs.next_event() {
            gamepad_handle = Some(event.id);
        }

        #[cfg(feature = "gamepad")]
        if let Some(gamepad) = gamepad_handle.map(|h| gilrs.gamepad(h)) {
            // see [ref:input_handler] for mapping info
            gamepad.axis_data(Axis::LeftStickX).map(|axis| {
                analog_input(1, axis.value());
            });
            if gamepad.is_pressed(Button::LeftTrigger) {
                gamepad.axis_data(Axis::LeftStickY).map(|axis| {
                    analog_input(5, axis.value());
                });
            }
            if gamepad.is_pressed(Button::South) {
                keyboard_input(69)
            }
            if gamepad.is_pressed(Button::East) {
                keyboard_input(81)
            }
            if gamepad.is_pressed(Button::RightTrigger) {
                gamepad.axis_data(Axis::RightStickY).map(|axis| {
                    analog_input(3, axis.value());
                });
                gamepad.axis_data(Axis::RightStickX).map(|axis| {
                    analog_input(4, axis.value());
                });
            } else {
                gamepad.axis_data(Axis::RightStickY).map(|axis| {
                    analog_input(0, axis.value());
                });
            }
            if gamepad.is_pressed(Button::West) {
                keyboard_input(191)
            }
            if gamepad.is_pressed(Button::North) {
                keyboard_input(82)
            }
            if gamepad.is_pressed(Button::DPadLeft) {
                keyboard_input(65)
            }
            if gamepad.is_pressed(Button::DPadRight) {
                keyboard_input(68)
            }
            if gamepad.is_pressed(Button::DPadUp) {
                keyboard_input(61)
            }
            if gamepad.is_pressed(Button::DPadDown) {
                keyboard_input(173)
            }
        }

        // see [ref:input_handler] for mapping info
        window.get_keys().iter().for_each(|key| match key {
            Key::A => keyboard_input(65),
            Key::D => keyboard_input(68),
            Key::Equal => keyboard_input(61),
            Key::Minus => keyboard_input(173),
            Key::Up => keyboard_input(38),
            Key::Down => keyboard_input(40),
            Key::Left => keyboard_input(37),
            Key::Right => keyboard_input(39),
            Key::R => keyboard_input(82),
            Key::Slash => keyboard_input(191),
            Key::E => keyboard_input(69),
            Key::Q => keyboard_input(81),
            _ => (),
        });

        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
