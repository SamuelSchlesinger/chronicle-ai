//! Game mechanics for D&D 5e.
//!
//! This module contains the business logic for game mechanics, separated from
//! the data storage in [`GameWorld`]. This separation allows for:
//! - Cleaner code organization
//! - Easier testing of mechanics in isolation
//! - Reusable mechanics functions
//!
//! ## Submodules
//!
//! - [`rest`]: Short rest and long rest recovery mechanics
//! - [`combat`]: Combat state transitions and turn management

mod combat;
mod rest;

pub use combat::{end_combat, next_turn, start_combat};
pub use rest::{apply_long_rest, apply_short_rest};
