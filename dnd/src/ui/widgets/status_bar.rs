//! Status bar widget

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};

use dnd_core::world::GameMode;

use crate::ai_worker::WorldUpdate;
use crate::app::InputMode;
use crate::ui::theme::GameTheme;

/// Status bar widget showing quick stats
pub struct StatusBarWidget<'a> {
    hp_current: i32,
    hp_maximum: i32,
    hp_temporary: i32,
    ac: u8,
    game_mode: GameMode,
    input_mode: InputMode,
    theme: &'a GameTheme,
    message: Option<&'a str>,
}

impl<'a> StatusBarWidget<'a> {
    /// Create from a WorldUpdate (the new way)
    pub fn from_world(world: &WorldUpdate, input_mode: InputMode, theme: &'a GameTheme) -> Self {
        Self {
            hp_current: world.player_hp.current,
            hp_maximum: world.player_hp.maximum,
            hp_temporary: world.player_hp.temporary,
            ac: world.player_ac,
            game_mode: world.mode,
            input_mode,
            theme,
            message: None,
        }
    }

    pub fn message(mut self, message: Option<&'a str>) -> Self {
        self.message = message;
        self
    }
}

impl Widget for StatusBarWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let hp_ratio = if self.hp_maximum > 0 {
            self.hp_current as f32 / self.hp_maximum as f32
        } else {
            1.0
        };
        let hp_color = self.theme.hp_color(hp_ratio);

        // HP display
        let hp_text = if self.hp_temporary > 0 {
            format!(
                "HP: {}/{} (+{})",
                self.hp_current, self.hp_maximum, self.hp_temporary
            )
        } else {
            format!("HP: {}/{}", self.hp_current, self.hp_maximum)
        };

        // AC display
        let ac = self.ac;

        // Game mode indicator
        let game_mode_text = match self.game_mode {
            GameMode::Exploration => "EXPLORE",
            GameMode::Combat => "COMBAT",
            GameMode::Dialogue => "DIALOGUE",
            GameMode::Rest => "RESTING",
        };

        let game_mode_style = match self.game_mode {
            GameMode::Combat => Style::default()
                .fg(self.theme.combat_text)
                .add_modifier(Modifier::BOLD),
            GameMode::Dialogue => Style::default().fg(self.theme.npc_dialogue),
            _ => Style::default().fg(self.theme.system_text),
        };

        // Input mode indicator (vim-style)
        let (input_mode_text, input_mode_style) = match self.input_mode {
            InputMode::Normal => (
                "NORMAL",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
            InputMode::Insert => (
                "INSERT",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            InputMode::Command => (
                "COMMAND",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        };

        let mut spans = vec![
            Span::styled(format!("-- {input_mode_text} --"), input_mode_style),
            Span::raw(" | "),
            Span::styled(hp_text, Style::default().fg(hp_color)),
            Span::raw(" | "),
            Span::styled(format!("AC: {ac}"), Style::default()),
            Span::raw(" | "),
            Span::styled(game_mode_text, game_mode_style),
        ];

        // Add message if present
        if let Some(msg) = self.message {
            spans.push(Span::raw(" | "));
            spans.push(Span::styled(
                msg,
                Style::default().add_modifier(Modifier::DIM),
            ));
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line);
        paragraph.render(area, buf);
    }
}

/// Hotkey bar widget
#[allow(dead_code)]
pub struct HotkeyBarWidget<'a> {
    game_mode: GameMode,
    input_mode: InputMode,
    theme: &'a GameTheme,
}

impl<'a> HotkeyBarWidget<'a> {
    pub fn new(game_mode: GameMode, input_mode: InputMode, theme: &'a GameTheme) -> Self {
        Self {
            game_mode,
            input_mode,
            theme,
        }
    }
}

impl Widget for HotkeyBarWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let hotkeys = match self.input_mode {
            InputMode::Normal => match self.game_mode {
                GameMode::Combat => vec![
                    ("A:attack", true),
                    ("c:cast", true),
                    ("d:dash", true),
                    ("u:use", true),
                    ("I:inv", true),
                    ("1-9:target", true),
                    ("e:end", false),
                    ("?:help", false),
                ],
                GameMode::Dialogue => vec![
                    ("i:insert", true),
                    ("1-9:response", true),
                    ("Esc:leave", false),
                    ("?:help", false),
                ],
                _ => vec![
                    ("i:insert", true),
                    ("I:inv", true),
                    ("C:char", true),
                    ("Q:quest", true),
                    ("j/k:scroll", true),
                    ("?:help", false),
                ],
            },
            InputMode::Insert => vec![
                ("Esc:normal", true),
                ("Enter:send", true),
                ("↑↓:history", false),
            ],
            InputMode::Command => vec![
                ("Esc:cancel", true),
                ("Enter:execute", true),
                (":q quit", false),
                (":w save", false),
                (":load", false),
                (":roll XdY", false),
            ],
        };

        let spans: Vec<Span> = hotkeys
            .iter()
            .flat_map(|(text, primary)| {
                let style = if *primary {
                    Style::default()
                } else {
                    Style::default().add_modifier(Modifier::DIM)
                };
                vec![Span::styled(*text, style), Span::raw("  ")]
            })
            .collect();

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line);
        paragraph.render(area, buf);
    }
}
