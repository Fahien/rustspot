use sdl2::{event::Event, keyboard::Keycode, mouse::MouseButton};

// Copyright © 2021
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

#[derive(Clone, Copy)]
pub struct Input {
    // Left, right, middle, x1, x2
    pub mouse_down: [bool; 5],
    pub mouse_down_updated: [bool; 5],
    pub mouse_up_updated: [bool; 5],

    pub mouse_pos: [f32; 2],

    pub ctrl_down: bool,
}

impl Input {
    pub fn new() -> Self {
        Self {
            mouse_down: [false; 5],
            mouse_down_updated: [false; 5],
            mouse_up_updated: [false; 5],
            mouse_pos: [0.0; 2],
            ctrl_down: false,
        }
    }

    fn mouse_button_as_index(mouse_btn: &MouseButton) -> usize {
        match mouse_btn {
            MouseButton::Unknown => unreachable!(),
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            MouseButton::X1 => 3,
            MouseButton::X2 => 4,
        }
    }

    #[allow(unused_variables)]
    pub fn handle(&mut self, event: &Event) {
        match event {
            Event::Quit { timestamp } => (),
            Event::AppTerminating { timestamp } => (),
            Event::AppLowMemory { timestamp } => (),
            Event::AppWillEnterBackground { timestamp } => (),
            Event::AppDidEnterBackground { timestamp } => (),
            Event::AppWillEnterForeground { timestamp } => (),
            Event::AppDidEnterForeground { timestamp } => (),
            Event::Window {
                timestamp,
                window_id,
                win_event,
            } => (),
            Event::KeyDown {
                timestamp,
                window_id,
                keycode: Some(Keycode::LCtrl),
                scancode,
                keymod,
                repeat,
            } => {
                self.ctrl_down = true;
            }
            Event::KeyUp {
                timestamp,
                window_id,
                keycode: Some(Keycode::LCtrl),
                scancode,
                keymod,
                repeat,
            } => {
                self.ctrl_down = false;
            }
            Event::TextEditing {
                timestamp,
                window_id,
                text,
                start,
                length,
            } => (),
            Event::TextInput {
                timestamp,
                window_id,
                text,
            } => (),
            Event::MouseMotion {
                timestamp,
                window_id,
                which,
                mousestate,
                x,
                y,
                xrel,
                yrel,
            } => {
                self.mouse_pos = [*x as f32, *y as f32];
            }
            Event::MouseButtonDown {
                timestamp,
                window_id,
                which,
                mouse_btn,
                clicks,
                x,
                y,
            } => {
                if *mouse_btn != MouseButton::Unknown {
                    let index = Self::mouse_button_as_index(mouse_btn);
                    // Do not update in the same frame
                    if !self.mouse_up_updated[index] {
                        self.mouse_down[index] = true;
                        self.mouse_down_updated[index] = true;
                    }
                }
            }
            Event::MouseButtonUp {
                timestamp,
                window_id,
                which,
                mouse_btn,
                clicks,
                x,
                y,
            } => {
                if *mouse_btn != MouseButton::Unknown {
                    let index = Self::mouse_button_as_index(mouse_btn);
                    // Do not update in the same frame
                    if !self.mouse_down_updated[index] {
                        self.mouse_down[index] = false;
                        self.mouse_up_updated[index] = true;
                    }
                }
            }
            Event::MouseWheel {
                timestamp,
                window_id,
                which,
                x,
                y,
                direction,
            } => (),
            Event::JoyAxisMotion {
                timestamp,
                which,
                axis_idx,
                value,
            } => (),
            Event::JoyBallMotion {
                timestamp,
                which,
                ball_idx,
                xrel,
                yrel,
            } => (),
            Event::JoyHatMotion {
                timestamp,
                which,
                hat_idx,
                state,
            } => (),
            Event::JoyButtonDown {
                timestamp,
                which,
                button_idx,
            } => (),
            Event::JoyButtonUp {
                timestamp,
                which,
                button_idx,
            } => (),
            Event::JoyDeviceAdded { timestamp, which } => (),
            Event::JoyDeviceRemoved { timestamp, which } => (),
            Event::ControllerAxisMotion {
                timestamp,
                which,
                axis,
                value,
            } => (),
            Event::ControllerButtonDown {
                timestamp,
                which,
                button,
            } => (),
            Event::ControllerButtonUp {
                timestamp,
                which,
                button,
            } => (),
            Event::ControllerDeviceAdded { timestamp, which } => (),
            Event::ControllerDeviceRemoved { timestamp, which } => (),
            Event::ControllerDeviceRemapped { timestamp, which } => (),
            Event::FingerDown {
                timestamp,
                touch_id,
                finger_id,
                x,
                y,
                dx,
                dy,
                pressure,
            } => (),
            Event::FingerUp {
                timestamp,
                touch_id,
                finger_id,
                x,
                y,
                dx,
                dy,
                pressure,
            } => (),
            Event::FingerMotion {
                timestamp,
                touch_id,
                finger_id,
                x,
                y,
                dx,
                dy,
                pressure,
            } => (),
            Event::DollarGesture {
                timestamp,
                touch_id,
                gesture_id,
                num_fingers,
                error,
                x,
                y,
            } => (),
            Event::DollarRecord {
                timestamp,
                touch_id,
                gesture_id,
                num_fingers,
                error,
                x,
                y,
            } => (),
            Event::MultiGesture {
                timestamp,
                touch_id,
                d_theta,
                d_dist,
                x,
                y,
                num_fingers,
            } => (),
            Event::ClipboardUpdate { timestamp } => (),
            Event::DropFile {
                timestamp,
                window_id,
                filename,
            } => (),
            Event::DropText {
                timestamp,
                window_id,
                filename,
            } => (),
            Event::DropBegin {
                timestamp,
                window_id,
            } => (),
            Event::DropComplete {
                timestamp,
                window_id,
            } => (),
            Event::AudioDeviceAdded {
                timestamp,
                which,
                iscapture,
            } => (),
            Event::AudioDeviceRemoved {
                timestamp,
                which,
                iscapture,
            } => (),
            Event::RenderTargetsReset { timestamp } => (),
            Event::RenderDeviceReset { timestamp } => (),
            Event::User {
                timestamp,
                window_id,
                type_,
                code,
                data1,
                data2,
            } => (),
            Event::Unknown { timestamp, type_ } => (),
            _ => (),
        }
    }

    pub fn reset(&mut self) {
        self.mouse_down_updated = [false; 5];
        self.mouse_up_updated = [false; 5];
    }
}
