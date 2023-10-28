use crate::{PrimaryWindow, SurfaceToken, Window, WindowCloseRequested};

use bevy_app::AppExit;
use bevy_ecs::prelude::*;
use bevy_input::{keyboard::KeyCode, Input};
use bevy_utils::HashSet;

/// Exit the application when there are no open windows.
///
/// This system is added by the [`WindowPlugin`] in the default configuration.
/// To disable this behavior, set `close_when_requested` (on the [`WindowPlugin`]) to `false`.
/// Ensure that you read the caveats documented on that field if doing so.
///
/// [`WindowPlugin`]: crate::WindowPlugin
pub fn exit_on_all_closed(mut app_exit_events: EventWriter<AppExit>, windows: Query<&Window>) {
    if windows.is_empty() {
        bevy_utils::tracing::info!("No windows are open, exiting");
        app_exit_events.send(AppExit);
    }
}

/// Exit the application when the primary window has been closed
///
/// This system is added by the [`WindowPlugin`]
///
/// [`WindowPlugin`]: crate::WindowPlugin
pub fn exit_on_primary_closed(
    mut app_exit_events: EventWriter<AppExit>,
    windows: Query<(), (With<Window>, With<PrimaryWindow>)>,
) {
    if windows.is_empty() {
        bevy_utils::tracing::info!("Primary window was closed, exiting");
        app_exit_events.send(AppExit);
    }
}

/// Close windows in response to [`WindowCloseRequested`] (e.g.  when the close button is pressed).
///
/// This system is added by the [`WindowPlugin`] in the default configuration.
/// To disable this behavior, set `close_when_requested` (on the [`WindowPlugin`]) to `false`.
/// Ensure that you read the caveats documented on that field if doing so.
///
/// [`WindowPlugin`]: crate::WindowPlugin
pub fn close_when_requested(
    mut commands: Commands,
    tokens: Query<&SurfaceToken>,
    mut closed: EventReader<WindowCloseRequested>,
    mut waiting_to_close: Local<HashSet<Entity>>,
) {
    for event in closed.read() {
        if let Ok(token) = tokens.get(event.window) {
            // Check if that is okay
            if token.is_safe_to_close_window() {
                commands.entity(event.window).despawn();
            } else {
                // Stash for later when the renderer cleans up the surface
                waiting_to_close.insert(event.window);
            }
        }
    }

    waiting_to_close.retain(|window_entity| {
        if let Ok(token) = tokens.get(*window_entity) {
            if token.is_safe_to_close_window() {
                commands.entity(*window_entity).despawn();
                return false;
            }
        }

        true
    })
}

/// Close the focused window whenever the escape key (<kbd>Esc</kbd>) is pressed
///
/// This is useful for examples or prototyping.
pub fn close_on_esc(
    mut commands: Commands,
    focused_windows: Query<(Entity, &Window)>,
    input: Res<Input<KeyCode>>,
) {
    for (window, focus) in focused_windows.iter() {
        if !focus.focused {
            continue;
        }

        if input.just_pressed(KeyCode::Escape) {
            commands.entity(window).despawn();
        }
    }
}

/// Windows need to hold on to a unique [SurfaceToken] to know if they are able
/// to be despawned
pub fn fixup_window_surface(
    mut commands: Commands,
    missing_surface_token: Query<Entity, (With<Window>, Without<SurfaceToken>)>,
) {
    missing_surface_token.for_each(|entity| {
        commands.entity(entity).insert(SurfaceToken::default());
    });
}
