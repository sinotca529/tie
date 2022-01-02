use tui::{
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::image::{Image, Rgb};

use super::Widget;

#[derive(Clone, PartialEq, Debug)]
pub struct Canvas {
    image: Image,
    cursor_coord: (usize, usize),
}

impl Canvas {
    pub fn new(image: Image) -> Self {
        Self {
            image,
            cursor_coord: (0, 0),
        }
    }

    pub fn move_cursor(&mut self, dir: crate::command::Direction) {
        match dir {
            crate::command::Direction::Up => {
                self.cursor_coord.1 = self.cursor_coord.1.saturating_sub(1);
            }
            crate::command::Direction::Down => {
                self.cursor_coord.1 = self
                    .cursor_coord
                    .1
                    .saturating_add(1)
                    .min(self.image.height() as usize - 1);
            }
            crate::command::Direction::Left => {
                self.cursor_coord.0 = self.cursor_coord.0.saturating_sub(1);
            }
            crate::command::Direction::Right => {
                self.cursor_coord.0 = self
                    .cursor_coord
                    .0
                    .saturating_add(1)
                    .min(self.image.width() as usize - 1);
            }
        }
    }

    pub fn edit(&mut self, color: Rgb) {
        self.image.edit(color, &self.cursor_coord);
    }
}

impl Widget for Canvas {
    fn render(&self, f: &mut tui::Frame<impl tui::backend::Backend>, rect: tui::layout::Rect) {
        let canvas = Block::default().title("Canvas").borders(Borders::ALL);
        let img = Paragraph::new(self.image.clone().into_text_with_cursor(&self.cursor_coord))
            .block(canvas)
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        f.render_widget(img, rect);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Direction;

    #[test]
    fn test_move_cursor() {
        let img = Image::read_from_file("tests/image/00.png").unwrap();
        let (w, h) = (img.width() as usize, img.height() as usize);
        let mut canvas = Canvas::new(img);

        assert_eq!(canvas.cursor_coord, (0, 0));
        for _ in 0..w {
            canvas.move_cursor(Direction::Right);
        }
        for _ in 0..h {
            canvas.move_cursor(Direction::Down);
        }
        assert_eq!(canvas.cursor_coord, (w - 1, h - 1));
    }

    #[test]
    fn boundary_test_move_cursor_left() {
        let img = Image::read_from_file("tests/image/00.png").unwrap();
        let mut canvas = Canvas::new(img);

        assert_eq!(canvas.cursor_coord, (0, 0));
        canvas.move_cursor(Direction::Left);
        assert_eq!(canvas.cursor_coord, (0, 0));
    }

    #[test]
    fn boundary_test_move_cursor_right() {
        let img = Image::read_from_file("tests/image/00.png").unwrap();
        let w = img.width() as usize;
        let mut canvas = Canvas::new(img);

        assert_eq!(canvas.cursor_coord, (0, 0));
        for _ in 0..w + 10 {
            canvas.move_cursor(Direction::Right);
        }
        assert_eq!(canvas.cursor_coord, (w - 1, 0));
    }

    #[test]
    fn boundary_test_move_cursor_up() {
        let img = Image::read_from_file("tests/image/00.png").unwrap();
        let mut canvas = Canvas::new(img);

        assert_eq!(canvas.cursor_coord, (0, 0));
        canvas.move_cursor(Direction::Up);
        assert_eq!(canvas.cursor_coord, (0, 0));
    }

    #[test]
    fn boundary_test_move_cursor_down() {
        let img = Image::read_from_file("tests/image/00.png").unwrap();
        let h = img.height() as usize;
        let mut canvas = Canvas::new(img);

        assert_eq!(canvas.cursor_coord, (0, 0));
        for _ in 0..h + 10 {
            canvas.move_cursor(Direction::Down);
        }
        assert_eq!(canvas.cursor_coord, (0, h - 1));
    }
}
