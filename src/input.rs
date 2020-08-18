use std::ptr;
use winapi::shared::{minwindef::DWORD, windef::POINT};
use winapi::um::winuser;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

impl MouseButton {
    pub fn press(self) {
        unsafe {
            send_mouse_event(match self {
                Self::Left => winuser::MOUSEEVENTF_LEFTDOWN,
                Self::Middle => winuser::MOUSEEVENTF_MIDDLEDOWN,
                Self::Right => winuser::MOUSEEVENTF_RIGHTDOWN,
            });
        }
    }

    pub fn release(self) {
        unsafe {
            send_mouse_event(match self {
                Self::Left => winuser::MOUSEEVENTF_LEFTUP,
                Self::Middle => winuser::MOUSEEVENTF_MIDDLEUP,
                Self::Right => winuser::MOUSEEVENTF_RIGHTUP,
            });
        }
    }

    pub fn click(self) {
        self.press();
        self.release();
    }
}

unsafe fn send_mouse_event(event: DWORD) {
    let mut input = std::mem::zeroed::<winuser::INPUT>();

    input.type_ = winuser::INPUT_MOUSE;
    input.u.mi_mut().dwFlags = event;

    winuser::SendInput(1, &mut input, std::mem::size_of::<winuser::INPUT>() as i32);
}

pub struct MouseCursor;

impl MouseCursor {
    pub fn get_pos() -> (i32, i32) {
        let mut point = POINT { x: 0, y: 0 };
        unsafe { winuser::GetCursorPos(&mut point) };
        (point.x, point.y)
    }

    pub fn set_pos(x: i32, y: i32) {
        unsafe {
            if winuser::SetCursorPos(x, y) == 0 {
                panic!("SetCursorPos failed");
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Accent,
}

impl Key {
    pub fn key_code(self) -> i32 {
        use Key::*;

        match self {
            Accent => winuser::VK_OEM_5,
        }
    }

    pub fn is_pressed(self) -> bool {
        (unsafe { winuser::GetAsyncKeyState(self.key_code()) } >> 15) != 0
    }

    pub fn register_global_hotkey(self, id: winapi::ctypes::c_int) -> Result<RegisteredHotkey, ()> {
        let result =
            unsafe { winuser::RegisterHotKey(ptr::null_mut(), id, 0, self.key_code() as u32) };

        if result == 0 {
            Err(())
        } else {
            Ok(RegisteredHotkey(id))
        }
    }
}

#[derive(Clone)]
pub struct RegisteredHotkey(winapi::ctypes::c_int);

impl RegisteredHotkey {
    pub fn unregister(self) -> Result<(), ()> {
        // SAFETY: We take ownership of self and prevent if from getting dropped so unregister_internal will not be called again
        let result = unsafe { self.unregister_internal() };

        // Prevent self from getting dropped and unregistering the hotkey again
        std::mem::ManuallyDrop::new(self);

        result
    }

    /// SAFETY: This must only be called once, either by drop or by unregister
    unsafe fn unregister_internal(&self) -> Result<(), ()> {
        let result = winuser::UnregisterHotKey(ptr::null_mut(), self.0);

        if result == 0 {
            Err(())
        } else {
            Ok(())
        }
    }
}

impl Drop for RegisteredHotkey {
    fn drop(&mut self) {
        // SAFETY: We are in drop, so unregister_internal will not be called again
        unsafe {
            self.unregister_internal()
                .expect("Failed to deregister hotkey on drop");
        }
    }
}

pub struct MessageBus {
    msg: winuser::MSG,
}

impl MessageBus {
    pub fn new() -> Self {
        MessageBus {
            msg: unsafe { std::mem::zeroed() },
        }
    }

    pub fn wait_for_hotkey(&mut self, id: &RegisteredHotkey) -> Result<(), ()> {
        loop {
            unsafe {
                let ret = winuser::GetMessageA(&mut self.msg, ptr::null_mut(), 0, 0);
                if ret == 0 || ret == -1 {
                    return Err(());
                }

                // wParam is hotkey id
                if self.msg.message == winuser::WM_HOTKEY && self.msg.wParam == id.0 as usize {
                    return Ok(());
                }
            }
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            while winuser::PeekMessageW(&mut self.msg, ptr::null_mut(), 0, 0, winuser::PM_REMOVE)
                > 0
            {}
        }
    }
}
