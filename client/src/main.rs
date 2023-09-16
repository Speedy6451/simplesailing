use minifb::{Key, ScaleMode, Window, WindowOptions, Scale};
extern crate pirates;
use pirates::{WIDTH, HEIGHT, Input};
#[cfg(feature = "gilrs")]
use gilrs::{Axis, Gilrs, Button};
use pirates::Input::*;

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

    fn keyboard_input(key: Input) {
        unsafe {
            pirates::KEYCODE[0] = key as u8;
            pirates::keyboard_input();
        }
    }

    fn analog_input(chan: Input, val: f32) {
        let val = (val * 127.0) as i8;
        unsafe {
            pirates::KEYCODE[0] = chan as u8;
            pirates::KEYCODE[1] = (val + 127) as u8;
            pirates::keyboard_input();
        }
    }

    #[cfg(feature = "gamepad")]
    let mut gamepad_handle = None;

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

        #[cfg(feature = "gamepad")]
        while let Some(event) = gilrs.next_event() {
            gamepad_handle = Some(event.id);
        }

        #[cfg(feature = "gamepad")]
        if let Some(gamepad) = gamepad_handle.map(|h| gilrs.gamepad(h)) {
            gamepad.axis_data(Axis::LeftStickX).map(|axis| {
                analog_input(AxisRudder, axis.value());
            });
            if gamepad.is_pressed(Button::RightTrigger) {
                gamepad.axis_data(Axis::RightStickY).map(|axis| {
                    analog_input(AxisPanY, axis.value());
                });
                gamepad.axis_data(Axis::RightStickX).map(|axis| {
                    analog_input(AxisPanX, axis.value());
                });
            } else {
                gamepad.axis_data(Axis::RightStickY).map(|axis| {
                    analog_input(AxisZoom, axis.value());
                });
            }
            if gamepad.is_pressed(Button::DPadLeft) {
                keyboard_input(PanLeft)
            }
            if gamepad.is_pressed(Button::DPadRight) {
                keyboard_input(PanRight)
            }
            if gamepad.is_pressed(Button::DPadUp) {
                keyboard_input(ZoomIn)
            }
            if gamepad.is_pressed(Button::DPadDown) {
                keyboard_input(ZoomOut)
            }
        }

        window.get_keys().iter().for_each(|key| match key {
            Key::A => keyboard_input(RudderLeft),
            Key::D => keyboard_input(RudderRight),
            Key::Equal => keyboard_input(ZoomIn),
            Key::Minus => keyboard_input(ZoomOut),
            Key::Up => keyboard_input(PanUp),
            Key::Down => keyboard_input(PanDown),
            Key::Left => keyboard_input(PanLeft),
            Key::Right => keyboard_input(PanRight),
            _ => (),
        });

        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
