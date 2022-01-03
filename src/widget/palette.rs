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

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum PaletteID {
    ID0 = 0,
    ID1 = 1,
    ID2 = 2,
    ID3 = 3,
    ID4 = 4,
    ID5 = 5,
}

impl PaletteID {
    pub const NUM_COLORS: usize = 6;
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Palette {
    colors: [Rgb; PaletteID::NUM_COLORS],
}

impl Palette {
    /// Return a reference to color of palette.
    pub fn color(&self, id: PaletteID) -> &Rgb {
        &self.colors[id as usize]
    }

    /// Return a mutable reference to color of palette.
    pub fn color_mut(&mut self, id: PaletteID) -> &mut Rgb {
        &mut self.colors[id as usize]
    }
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
            vec![Span::styled("_", Style::default().fg(Color::Black))].into(),
            up.into(),
            vec![].into(),
            vec![Span::styled("_", Style::default().fg(Color::Black))].into(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color() {
        let p = Palette::default();
        assert_eq!(p.color(PaletteID::ID0), &Rgb(0, 0, 0));
        assert_eq!(p.color(PaletteID::ID3), &Rgb(255, 0, 0));
    }

    #[test]
    fn test_color_mut() {
        let mut p = Palette::default();
        let cp = p.clone();

        assert_eq!(p.color(PaletteID::ID0), &Rgb(0, 0, 0));
        *p.color_mut(PaletteID::ID0) = Rgb(3, 4, 5);
        assert_eq!(p.color(PaletteID::ID0), &Rgb(3, 4, 5));

        *p.color_mut(PaletteID::ID0) = Rgb(0, 0, 0);
        assert_eq!(p, cp);
    }
}
