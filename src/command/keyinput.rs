use crossterm::event::{self, KeyCode};
use once_cell::sync::Lazy;
use regex::Regex;
use tui::{
    layout::Alignment,
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{
    image::Rgb,
    widget::{palette::PaletteID, Widget},
};

use super::{Command, CommandStream, Direction};

/// Fetch key event and use it as Command
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct KeyInput {
    cmd_line_content: String,
}

impl KeyInput {
    pub fn new() -> Self {
        Self {
            cmd_line_content: String::new(),
        }
    }
}

impl KeyInput {
    /// convert KeyCode to Command.
    fn keycode2command(keycode: &KeyCode) -> Command {
        match keycode {
            KeyCode::Char('q') => Command::Quit,
            KeyCode::Char('h') => Command::Direction(Direction::Left),
            KeyCode::Char('j') => Command::Direction(Direction::Down),
            KeyCode::Char('k') => Command::Direction(Direction::Up),
            KeyCode::Char('l') => Command::Direction(Direction::Right),
            KeyCode::Char('w') => Command::Palette(PaletteID::ID0),
            KeyCode::Char('e') => Command::Palette(PaletteID::ID1),
            KeyCode::Char('r') => Command::Palette(PaletteID::ID2),
            KeyCode::Char('s') => Command::Palette(PaletteID::ID3),
            KeyCode::Char('d') => Command::Palette(PaletteID::ID4),
            KeyCode::Char('f') => Command::Palette(PaletteID::ID5),
            _ => Command::Nop,
        }
    }

    /// convert Into<&str> to Command.
    fn str2command(s: impl AsRef<str>) -> Command {
        static RE_SET_COLOR: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^: *set +(\w) +(\d+) +(\d+) +(\d+) *$").unwrap());

        if let Some(cap) = RE_SET_COLOR.captures(s.as_ref()) {
            let id = match &cap[1] {
                "w" => Some(PaletteID::ID0),
                "e" => Some(PaletteID::ID1),
                "r" => Some(PaletteID::ID2),
                "a" => Some(PaletteID::ID3),
                "s" => Some(PaletteID::ID4),
                "d" => Some(PaletteID::ID5),
                _ => None,
            };
            if let (Some(id), Ok(r), Ok(g), Ok(b)) =
                (id, cap[2].parse(), cap[3].parse(), cap[4].parse())
            {
                let rgb = Rgb(r, g, b);
                return Command::SetPalette(id, rgb);
            }
        }
        Command::Nop
    }

    fn process_text_command(&mut self, keycode: &KeyCode) -> Command {
        match keycode {
            KeyCode::Enter => {
                let cmd = Self::str2command(&self.cmd_line_content);
                self.cmd_line_content.clear();
                cmd
            }
            KeyCode::Char(ch) => {
                self.cmd_line_content.push(*ch);
                Command::Nop
            }
            KeyCode::Backspace => {
                self.cmd_line_content.pop();
                Command::Nop
            }
            _ => Command::Nop,
        }
    }
}

impl Widget for KeyInput {
    fn render(&self, f: &mut tui::Frame<impl tui::backend::Backend>, rect: tui::layout::Rect) {
        if self.cmd_line_content.is_empty() {
            let cmd_line = Block::default().borders(Borders::ALL);
            let msg = Paragraph::new(Text::raw("Begin input command by ':'"))
                .block(cmd_line)
                .style(
                    Style::default()
                        .fg(Color::Rgb(128, 128, 128))
                        .bg(Color::Black),
                )
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: false });

            f.render_widget(msg, rect);
        } else {
            let cmd_line = Block::default().borders(Borders::ALL);
            let msg = Paragraph::new(Text::raw(&self.cmd_line_content))
                .block(cmd_line)
                .style(Style::default().fg(Color::White).bg(Color::Black))
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: false });

            f.render_widget(msg, rect);
        };
    }
}

impl CommandStream for KeyInput {
    type Error = std::io::Error;

    fn read(&mut self) -> Result<Command, Self::Error> {
        event::read().map(|op| {
            if self.cmd_line_content.is_empty() {
                match op {
                    event::Event::Key(key) if key.code == KeyCode::Char(':') => {
                        self.cmd_line_content.push(':');
                        Command::Nop
                    }
                    event::Event::Key(key) => Self::keycode2command(&key.code),
                    _ => Command::Nop,
                }
            } else {
                match op {
                    event::Event::Key(key) => self.process_text_command(&key.code),
                    _ => Command::Nop,
                }
            }
        })
    }
}
