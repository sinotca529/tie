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
pub enum PaletteCellId {
    Id0 = 0,
    Id1 = 1,
    Id2 = 2,
    Id3 = 3,
    Id4 = 4,
    Id5 = 5,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Palette {
    cells: [Rgb; Self::NUM_CELL],
}

impl Palette {
    pub const NUM_CELL: usize = 6;

    /// Return a reference to color of palette.
    pub fn color(&self, id: PaletteCellId) -> &Rgb {
        &self.cells[id as usize]
    }

    /// Set color of palette's specified cell.
    pub fn set_color(&mut self, id: PaletteCellId, color: Rgb) {
        self.cells[id as usize] = color;
    }
}

impl Widget for Palette {
    fn render(&self, f: &mut Frame<impl Backend>, rect: Rect) {
        let up = (0..3)
            .map(|i| {
                let color = self.cells[i].into();
                vec![
                    Span::styled("[]", Style::default().fg(color).bg(color)),
                    Span::raw(" "),
                ]
            })
            .flatten()
            .collect::<Vec<Span<'static>>>();

        let down = (3..6)
            .map(|i| {
                let color = self.cells[i].into();
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
            cells: [
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
        assert_eq!(p.color(PaletteCellId::Id0), &Rgb(0, 0, 0));
        assert_eq!(p.color(PaletteCellId::Id3), &Rgb(255, 0, 0));
    }

    #[test]
    fn test_color_mut() {
        let mut p = Palette::default();
        let cp = p.clone();

        assert_eq!(p.color(PaletteCellId::Id0), &Rgb(0, 0, 0));
        p.set_color(PaletteCellId::Id0, Rgb(3, 4, 5));
        assert_eq!(p.color(PaletteCellId::Id0), &Rgb(3, 4, 5));

        p.set_color(PaletteCellId::Id0, Rgb(0, 0, 0));
        assert_eq!(p, cp);
    }
}
