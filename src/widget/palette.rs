use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Span, Text},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::image::Rgb;

use super::Widget;

const NUM_COLORS: usize = 6;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum PaletteID {
    ID1,
    ID2,
    ID3,
    ID4,
    ID5,
    ID6,
}

pub struct Palette {
    colors: [Rgb; NUM_COLORS],
}

impl Widget for Palette {
    fn render(&self, f: &mut Frame<impl Backend>, rect: Rect) {
        let up = (0..3)
            .map(|i| {
                let color = self.colors[i].into();
                vec![
                    Span::styled("[]", Style::default().fg(color).bg(color)),
                    Span::raw(" "),
                ]
            })
            .flatten()
            .collect::<Vec<Span<'static>>>();

        let down = (3..6)
            .map(|i| {
                let color = self.colors[i].into();
                vec![
                    Span::styled("[]", Style::default().fg(color).bg(color)),
                    Span::raw(" "),
                ]
            })
            .flatten()
            .collect::<Vec<Span<'static>>>();

        let text: Text<'static> = vec![
            vec![
                Span::raw("W  E  R"),
                Span::styled("_", Style::default().fg(Color::Black)),
            ]
            .into(),
            up.into(),
            vec![].into(),
            vec![
                Span::raw("S  D  F"),
                Span::styled("_", Style::default().fg(Color::Black)),
            ]
            .into(),
            down.into(),
        ]
        .into();

        let palette = Paragraph::new(text)
            .block(Block::default().title("Palette").borders(Borders::ALL))
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        f.render_widget(palette, rect);
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            colors: [
                Rgb(0, 0, 0),
                Rgb(127, 127, 127),
                Rgb(255, 255, 255),
                Rgb(255, 0, 0),
                Rgb(0, 255, 0),
                Rgb(0, 0, 255),
            ],
        }
    }
}
