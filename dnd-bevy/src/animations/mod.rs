//! Animation systems for visual feedback.
//!
//! This module provides screen shake effects for combat feedback.

pub mod effects;

pub use effects::{animate_screen_shake, spawn_screen_shake};

use bevy::prelude::*;

/// Marker component for animations that should be cleaned up when finished.
#[derive(Component)]
pub struct AnimationLifetime {
    pub remaining: f32,
}

/// System to clean up finished animations.
pub fn cleanup_finished_animations(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut AnimationLifetime)>,
) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.remaining -= time.delta_secs();
        if lifetime.remaining <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}
