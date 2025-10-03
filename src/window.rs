#![allow(dead_code)]
use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

pub fn grab_cursor(mut primary_window: Single<&mut Window, With<PrimaryWindow>>) {
    set_cursor_grab_mode(&mut primary_window, CursorGrabMode::Locked);
}

pub fn release_cursor(mut primary_window: Single<&mut Window, With<PrimaryWindow>>) {
    set_cursor_grab_mode(&mut primary_window, CursorGrabMode::None);
}

pub fn hide_cursor(mut primary_window: Single<&mut Window, With<PrimaryWindow>>) {
    set_cursor_visibility(&mut primary_window, false);
}

pub fn show_cursor(mut primary_window: Single<&mut Window, With<PrimaryWindow>>) {
    set_cursor_visibility(&mut primary_window, true);
}

pub fn toggle_cursor(
    mut primary_window: Single<&mut Window, With<PrimaryWindow>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if !(input.just_pressed(KeyCode::KeyL)
        && (input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight)))
    {
        return;
    }

    if primary_window.cursor_options.visible {
        set_cursor_visibility(&mut primary_window, false);
        set_cursor_grab_mode(&mut primary_window, CursorGrabMode::Locked);
    } else {
        set_cursor_visibility(&mut primary_window, true);
        set_cursor_grab_mode(&mut primary_window, CursorGrabMode::None);
    }
}

fn set_cursor_grab_mode(window: &mut Window, mode: CursorGrabMode) {
    if mode == CursorGrabMode::Locked {
        let center = Vec2::new(window.width() / 2.0, window.height() / 2.0);
        window.set_cursor_position(Some(center));
    }

    window.cursor_options.grab_mode = mode;
}

fn set_cursor_visibility(window: &mut Window, visible: bool) {
    window.cursor_options.visible = visible;
}
