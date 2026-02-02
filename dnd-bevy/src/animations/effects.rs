//! Screen shake effect.
//!
//! Provides camera shake feedback for combat events.

use bevy::prelude::*;

use super::AnimationLifetime;

/// Component for screen shake animation.
#[derive(Component)]
pub struct ScreenShake {
    /// Shake intensity (0.0 to 1.0).
    pub intensity: f32,
    /// Animation duration.
    pub duration: f32,
    /// Time elapsed.
    pub elapsed: f32,
}

/// Spawn a screen shake effect.
pub fn spawn_screen_shake(commands: &mut Commands, intensity: f32) {
    let duration = 0.3;

    commands.spawn((
        ScreenShake {
            intensity: intensity.clamp(0.0, 1.0),
            duration,
            elapsed: 0.0,
        },
        AnimationLifetime {
            remaining: duration + 0.1,
        },
    ));
}

/// System to animate screen shake.
pub fn animate_screen_shake(
    time: Res<Time>,
    mut query: Query<&mut ScreenShake>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
) {
    // Accumulate screen shake from all active shake effects
    let mut total_shake = Vec2::ZERO;

    for mut shake in query.iter_mut() {
        shake.elapsed += time.delta_secs();
        let progress = (shake.elapsed / shake.duration).min(1.0);

        // Decaying screen shake
        let shake_amount = shake.intensity * 10.0 * (1.0 - progress);
        let shake_x = (shake.elapsed * 50.0).sin() * shake_amount;
        let shake_y = (shake.elapsed * 43.0).cos() * shake_amount;
        total_shake += Vec2::new(shake_x, shake_y);
    }

    // Apply screen shake to camera
    if total_shake != Vec2::ZERO {
        for mut transform in camera_query.iter_mut() {
            transform.translation.x = total_shake.x;
            transform.translation.y = total_shake.y;
        }
    } else {
        // Reset camera position when no shake
        for mut transform in camera_query.iter_mut() {
            transform.translation.x = 0.0;
            transform.translation.y = 0.0;
        }
    }
}
