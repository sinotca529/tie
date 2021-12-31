use std::{fs::File, path::Path};
use thiserror::Error;
use tui::{
    style::{Color, Style},
    text::{Span, Spans, Text},
};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    fn opposite(&self) -> Self {
        Self(255 - self.0, 255 - self.1, 255 - self.2)
    }
}

impl From<Rgb> for tui::style::Color {
    fn from(rgb: Rgb) -> Self {
        Self::Rgb(rgb.0, rgb.1, rgb.2)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Image {
    width: u32,
    height: u32,
    data: Text<'static>,
}

#[derive(Error, Debug)]
pub enum ImageError {
    #[error("IO error.")]
    IO(#[source] std::io::Error),
    #[error("This image type is not supported.")]
    UnsupportedImgType,
    #[error("Failed to decode.")]
    Decode(#[source] png::DecodingError),
}

impl Image {
    const CURSOR_STR: &'static str = "[]";

    /// Read image from file.
    pub fn read_from_file(path: impl AsRef<Path>) -> Result<Image, ImageError> {
        dbg!(path.as_ref());

        let file = File::open(path).map_err(ImageError::IO)?;
        let decoder = png::Decoder::new(&file);
        let mut reader = decoder.read_info().map_err(ImageError::Decode)?;
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).unwrap();

        if (info.color_type != png::ColorType::Rgb) || (info.bit_depth != png::BitDepth::Eight) {
            dbg!(info.color_type, info.bit_depth);
            return Err(ImageError::UnsupportedImgType);
        }

        let (width, height) = (info.width, info.height);
        dbg!(width, height);

        let bytes = &buf[..info.buffer_size()];

        assert_eq!((width * height * 3) as usize, bytes.len());

        let data: Text<'static> = bytes
            .chunks(3 * width as usize)
            .map(|rgbs: &[u8]| {
                let mut line = Vec::with_capacity(width as usize);
                for i in 0..(width as usize) {
                    let base = 3 * i;
                    let color = Color::Rgb(rgbs[base], rgbs[base + 1], rgbs[base + 2]);
                    let style = Style::default().bg(color).fg(color);
                    let span = Span::styled(Self::CURSOR_STR, style);
                    line.push(span);
                }
                Into::<Spans<'static>>::into(line)
            })
            .collect::<Vec<Spans<'static>>>()
            .into();

        file.sync_all().map_err(ImageError::IO)?;

        Ok(Image {
            width,
            height,
            data,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn into_text_with_cursor(self, cursor_coord: (usize, usize)) -> Text<'static> {
        let mut img_txt = self.data;

        let span = &mut img_txt.lines[cursor_coord.1].0[cursor_coord.0];

        if let Some(Color::Rgb(r, g, b)) = span.style.bg {
            let opposite_color = Rgb(r, g, b).opposite().into();
            span.style = span.style.fg(opposite_color);
            img_txt
        } else {
            unreachable!()
        }
    }
}

impl From<Image> for Text<'static> {
    fn from(img: Image) -> Self {
        img.data
    }
}

#[cfg(test)]
mod tests {
    use tui::style::Color;

    use super::*;

    /// This test checks whether `Image::read_from_file` can parse `./tests/image/01.png`.
    #[test]
    fn test_read_from() {
        let img = Image::read_from_file("./tests/image/00.png");
        assert!(img.is_ok());
        let img = img.unwrap();

        let expected_colors = vec![
            vec![
                Rgb(237, 28, 36),
                Rgb(63, 72, 204),
                Rgb(255, 255, 255),
                Rgb(255, 255, 255),
                Rgb(255, 127, 39),
            ],
            vec![
                Rgb(255, 255, 255),
                Rgb(255, 255, 255),
                Rgb(255, 255, 255),
                Rgb(255, 255, 255),
                Rgb(255, 242, 0),
            ],
        ];

        let (width, height) = (5, 2);
        for y in 0..height {
            for x in 0..width {
                let expected_color: Color = expected_colors[y as usize][x as usize].into();
                let span = &img.data.lines[y as usize].0[x as usize];
                assert_eq!(span.content.to_string(), Image::CURSOR_STR);
                assert_eq!(span.style.fg, Some(expected_color));
                assert_eq!(span.style.bg, Some(expected_color));
            }
        }
    }

    /// This test checks whether `Image::read_from_file` return `ImageError::IO` when it passed a path to non-exist file.
    #[test]
    fn test_read_from_error_io() {
        let img = Image::read_from_file("./tests/image/non-exist.png");
        assert!(matches!(img, Err(ImageError::IO(_))));
    }

    /// This test checks whether `Image::read_from_file` return `ImageError::UnsupportedImgType` error when it passed a path to transparent png file.
    #[test]
    fn test_read_from_error_unsupported() {
        let img = Image::read_from_file("./tests/image/transparent.png");
        assert!(matches!(img, Err(ImageError::UnsupportedImgType)));
    }
    /// This test checks whether `Image::read_from_file` return `ImageError::Decode` error when it passed a path to non-png file.
    #[test]
    fn test_read_from_error_decode() {
        let img = Image::read_from_file("./tests/image/not-png.txt");
        assert!(matches!(img, Err(ImageError::Decode(_))));
    }

    #[test]
    fn test_into_text() {
        let img = Image::read_from_file("./tests/image/00.png").unwrap();
        let text: Text<'static> = img.clone().into();
        assert_eq!(img.data, text);
    }

    #[test]
    fn rgb_into_test() {
        let rgb = Rgb(2, 4, 8);
        let tui_rgb: Color = From::from(rgb);
        assert_eq!(tui_rgb, Color::Rgb(2, 4, 8));
    }

    #[test]
    fn test_into_text_with_cursor() {
        let img = Image::read_from_file("./tests/image/00.png").unwrap();
        let (w, h) = (img.width as usize, img.height as usize);

        let cursor_coord = (3, 1);
        assert!(cursor_coord < (w - 1, h - 1));
        let with_cursor = img.into_text_with_cursor(cursor_coord);

        for y in 0..h {
            for x in 0..w {
                let span = &with_cursor.lines[y].0[x];
                let fg = span.style.fg.unwrap();
                let bg = span.style.bg.unwrap();
                if let (Color::Rgb(fr, fg, fb), Color::Rgb(br, bg, bb)) = (fg, bg) {
                    let fg = Rgb(fr, fg, fb);
                    let bg = Rgb(br, bg, bb);
                    if (x, y) == cursor_coord {
                        assert_eq!(fg.opposite(), bg);
                    } else {
                        assert_eq!(fg, bg);
                    }
                } else {
                    unreachable!()
                }
            }
        }
    }
}
