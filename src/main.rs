use std::{ptr, thread::sleep, time::Duration};

use winapi::um::winuser;

fn click() {
    unsafe {
        let mut input = std::mem::zeroed::<winuser::INPUT>();
        input.type_ = winuser::INPUT_MOUSE;

        input.u.mi_mut().dwFlags = winuser::MOUSEEVENTF_LEFTDOWN;
        winuser::SendInput(1, &mut input, std::mem::size_of::<winuser::INPUT>() as i32);

        input.u.mi_mut().dwFlags = winuser::MOUSEEVENTF_LEFTUP;
        winuser::SendInput(1, &mut input, std::mem::size_of::<winuser::INPUT>() as i32);
    }

    sleep(Duration::from_millis(50));
}

fn get_mouse_pos() -> (i32, i32) {
    use winapi::shared::windef::POINT;
    let mut point = POINT { x: 0, y: 0 };
    unsafe { winuser::GetCursorPos(&mut point) };
    (point.x, point.y)
}

fn set_mouse_pos(x: i32, y: i32) {
    unsafe {
        if winuser::SetCursorPos(x, y) == 0 {
            panic!("SetCursorPos failed");
        }
    }
}

fn key_pressed(key: u32) -> bool {
    (unsafe { winuser::GetAsyncKeyState(key as i32) } >> 15) != 0
}

const TRIGGER_KEY: u32 = winuser::VK_OEM_5 as u32; // ^ key on German keyboard

fn main() {
    unsafe {
        // register as hotkey id 1
        if winuser::RegisterHotKey(ptr::null_mut(), 1, 0, TRIGGER_KEY) == 0 {
            panic!("RegisterHotKey failed");
        }

        let mut msg = std::mem::zeroed();

        loop {
            let ret = winuser::GetMessageA(&mut msg, ptr::null_mut(), 0, 0);
            if ret == 0 {
                break;
            }
            if ret == -1 {
                println!("GetMessageA failed");
                break;
            }

            // wParam is hotkey id
            if msg.message == winuser::WM_HOTKEY && msg.wParam == 1 {
                let mut layer = 1;
                let mut index = 0;
                let (mut x, mut y) = get_mouse_pos();

                click();
                x -= 1;
                y -= 1;
                set_mouse_pos(x, y);
                click();

                while key_pressed(TRIGGER_KEY) {
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

                    set_mouse_pos(x, y);
                    click();
                }

                while winuser::PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, winuser::PM_REMOVE) > 0
                {
                }
            }
        }

        if winuser::UnregisterHotKey(ptr::null_mut(), 1) == 0 {
            panic!("UnregisterHotKey failed");
        }
    }
}
