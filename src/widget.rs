use tui::{backend::Backend, layout::Rect, Frame};

pub mod canvas;
pub mod palette;

pub trait Widget {
    fn render(&self, f: &mut Frame<impl Backend>, rect: Rect);
}
