use tui::{backend::Backend, layout::Rect, Frame};

pub mod canvas;
pub mod palette;

pub trait Widget {
    /// Render contents in specified Frame's specified Rect.
    fn render(&self, f: &mut Frame<impl Backend>, rect: Rect);
}
