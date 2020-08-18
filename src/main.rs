use std::{thread::sleep, time::Duration};

mod input;

use input::{Key, MessageBus, MouseButton, MouseCursor};

fn click_at(x: i32, y: i32) {
    MouseCursor::set_pos(x, y);
    MouseButton::Left.click();
    sleep(Duration::from_millis(50));
}

fn main() {
    let hotkey_handle = Key::Accent
        .register_global_hotkey(1)
        .expect("Failed to register hotkey");

    let mut msg_bus = MessageBus::new();

    while let Ok(()) = msg_bus.wait_for_hotkey(&hotkey_handle) {
        let mut layer = 1;
        let mut index = 0;
        let (mut x, mut y) = MouseCursor::get_pos();

        click_at(x, y);
        x -= 1;
        y -= 1;
        click_at(x, y);

        while Key::Accent.is_pressed() {
            index += 1;

            if index <= layer * 2 {
                x += 1;
            } else if index <= layer * 4 {
                y += 1;
            } else if index <= layer * 6 {
                x -= 1;
            } else if index < layer * 8 {
                y -= 1;
            } else {
                layer += 1;
                index = 0;
                x -= 1;
                y -= 2;
            }

            click_at(x, y);
        }

        msg_bus.clear();
    }
}
