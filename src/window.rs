#![allow(dead_code)]
use bevy::{
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

pub fn grab_cursor(window_query: Single<(&mut Window, &mut CursorOptions), With<PrimaryWindow>>) {
    let (mut window, mut cursor_options) = window_query.into_inner();
    set_cursor_grab_mode(&mut window, &mut cursor_options, CursorGrabMode::Locked);
}

pub fn release_cursor(
    window_query: Single<(&mut Window, &mut CursorOptions), With<PrimaryWindow>>,
) {
    let (mut window, mut cursor_options) = window_query.into_inner();
    set_cursor_grab_mode(&mut window, &mut cursor_options, CursorGrabMode::None);
}

pub fn hide_cursor(cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    set_cursor_visibility(&mut cursor_options.into_inner(), false);
}

pub fn show_cursor(cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    set_cursor_visibility(&mut cursor_options.into_inner(), true);
}

pub fn toggle_cursor(
    window_query: Single<(&mut Window, &mut CursorOptions), With<PrimaryWindow>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if !(input.just_pressed(KeyCode::KeyL)
        && (input.pressed(KeyCode::ShiftLeft) || input.pressed(KeyCode::ShiftRight)))
    {
        return;
    }

    let (mut window, mut cursor_options) = window_query.into_inner();

    if cursor_options.visible {
        set_cursor_visibility(&mut cursor_options, false);
        set_cursor_grab_mode(&mut window, &mut cursor_options, CursorGrabMode::Locked);
    } else {
        set_cursor_visibility(&mut cursor_options, true);
        set_cursor_grab_mode(&mut window, &mut cursor_options, CursorGrabMode::None);
    }
}

fn set_cursor_grab_mode(
    window: &mut Window,
    cursor_options: &mut CursorOptions,
    mode: CursorGrabMode,
) {
    if mode == CursorGrabMode::Locked {
        let center = Vec2::new(window.width() / 2.0, window.height() / 2.0);
        window.set_cursor_position(Some(center));
    }

    cursor_options.grab_mode = mode;
}

fn set_cursor_visibility(cursor_options: &mut CursorOptions, visible: bool) {
    cursor_options.visible = visible;
}
