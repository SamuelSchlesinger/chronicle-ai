//! In-game time tracking for D&D campaigns.
//!
//! This module provides the [`GameTime`] struct for tracking the passage of time
//! within a game session, including year, month, day, hour, and minute.

use serde::{Deserialize, Serialize};

/// In-game time tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameTime {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
}

impl GameTime {
    pub fn new(year: i32, month: u8, day: u8, hour: u8, minute: u8) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
        }
    }

    pub fn advance_minutes(&mut self, minutes: u32) {
        let total_minutes = self.minute as u32 + minutes;
        self.minute = (total_minutes % 60) as u8;
        let hours_to_add = total_minutes / 60;
        self.advance_hours(hours_to_add);
    }

    pub fn advance_hours(&mut self, hours: u32) {
        let total_hours = self.hour as u32 + hours;
        self.hour = (total_hours % 24) as u8;
        let days_to_add = total_hours / 24;
        self.advance_days(days_to_add);
    }

    pub fn advance_days(&mut self, days: u32) {
        let total_days = self.day as u32 + days;
        self.day = ((total_days - 1) % 30 + 1) as u8;
        let months_to_add = (total_days - 1) / 30;
        self.advance_months(months_to_add);
    }

    pub fn advance_months(&mut self, months: u32) {
        let total_months = self.month as u32 + months;
        self.month = ((total_months - 1) % 12 + 1) as u8;
        let years_to_add = (total_months - 1) / 12;
        self.year += years_to_add as i32;
    }

    pub fn is_daytime(&self) -> bool {
        self.hour >= 6 && self.hour < 18
    }

    pub fn time_of_day(&self) -> &'static str {
        match self.hour {
            5..=7 => "dawn",
            8..=11 => "morning",
            12..=13 => "midday",
            14..=17 => "afternoon",
            18..=20 => "evening",
            _ => "night",
        }
    }
}

impl Default for GameTime {
    fn default() -> Self {
        Self::new(1492, 3, 1, 10, 0) // Day 1 of the month
    }
}
