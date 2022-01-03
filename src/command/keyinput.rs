use self::keyconfig::KeyConfig;
use super::{Command, CommandStream};
use crate::{image::Rgb, widget::Widget};
use crossterm::event::{self, KeyCode};
use once_cell::sync::Lazy;
use regex::Regex;
use tui::{
    layout::Alignment,
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph, Wrap},
};

mod keyconfig;

/// Fetch key event and use it as Command
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct KeyInput {
    cmd_line_content: String,
    key_config: KeyConfig,
}

impl KeyInput {
    pub fn new() -> Self {
        Self {
            cmd_line_content: String::new(),
            key_config: KeyConfig::default(),
        }
    }
}

impl KeyInput {
    /// convert self.cmd_line_content to Command.
    fn parse_cmd(&self) -> Command {
        static RE_SET_COLOR: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^: *set +(\w) +(\d+) +(\d+) +(\d+) *$").unwrap());

        if let Some(cap) = RE_SET_COLOR.captures(&self.cmd_line_content) {
            let ch = cap[1].chars().next().unwrap();
            let id = self.key_config.char2palette_id(ch);

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
                let cmd = self.parse_cmd();
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
            let msg = format!(
                "Begin input command by '{}'",
                self.key_config.cmd_line_prefix()
            );
            let msg = Paragraph::new(Text::raw(msg))
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
                let cmd_prefix = self.key_config.cmd_line_prefix();
                match op {
                    event::Event::Key(key) if key.code == KeyCode::Char(cmd_prefix) => {
                        self.cmd_line_content.push(cmd_prefix);
                        Command::Nop
                    }
                    event::Event::Key(key) => self
                        .key_config
                        .get(&key.code)
                        .copied()
                        .unwrap_or(Command::Nop),
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

#[cfg(test)]
mod tests {
    use crate::widget::palette::PaletteID;

    use super::*;

    fn new_key_input(cmd_line_content: impl Into<String>) -> KeyInput {
        KeyInput {
            cmd_line_content: cmd_line_content.into(),
            key_config: KeyConfig::default(),
        }
    }

    #[test]
    fn test_new() {
        let ki = KeyInput::new();
        assert_eq!(ki.cmd_line_content, String::new());
        assert_eq!(ki.key_config, KeyConfig::default());
    }

    #[test]
    fn test_parse_cmd() {
        let ki = new_key_input("");
        assert_eq!(ki.parse_cmd(), Command::Nop);

        let ki = new_key_input(":");
        assert_eq!(ki.parse_cmd(), Command::Nop);

        let ki = new_key_input(":set w 255 255 128");
        assert_eq!(
            ki.parse_cmd(),
            Command::SetPalette(PaletteID::ID0, Rgb(255, 255, 128))
        );

        let ki = new_key_input(":  set  w 255   255  128  ");
        assert_eq!(
            ki.parse_cmd(),
            Command::SetPalette(PaletteID::ID0, Rgb(255, 255, 128))
        );

        let ki = new_key_input(":  set  w 255   255  128  ;");
        assert_eq!(ki.parse_cmd(), Command::Nop);

        let ki = new_key_input(":set w 999 255  128");
        assert_eq!(ki.parse_cmd(), Command::Nop);

        let ki = new_key_input(":set W 275 255 128");
        assert_eq!(ki.parse_cmd(), Command::Nop);
    }

    #[test]
    fn test_process_text_command() {
        let mut ki = new_key_input(":");
        assert_eq!(ki.process_text_command(&KeyCode::Char('s')), Command::Nop);
        assert_eq!(ki.cmd_line_content, String::from(":s"));

        assert_eq!(ki.process_text_command(&KeyCode::Backspace), Command::Nop);
        assert_eq!(ki.cmd_line_content, String::from(":"));

        assert_eq!(ki.process_text_command(&KeyCode::Tab), Command::Nop);
        assert_eq!(ki.cmd_line_content, String::from(":"));

        let mut ki = new_key_input(":set w 255 255 128");
        assert_eq!(
            ki.process_text_command(&KeyCode::Enter),
            Command::SetPalette(PaletteID::ID0, Rgb(255, 255, 128))
        );
        assert_eq!(ki.cmd_line_content, String::new());
    }
}
