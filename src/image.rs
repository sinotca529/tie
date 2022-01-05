use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
};
use tui::{
    style::{Color, Style},
    text::{Span, Spans, Text},
};

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    /// Opposite color of self.
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
    /// Path of the image file.
    path: PathBuf,
    width: u32,
    height: u32,
    /// Data of image described as text to render the image in terminal.
    data: Text<'static>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error.")]
    IO(#[source] std::io::Error),

    #[error("This image type is not supported.")]
    UnsupportedImgType,

    #[error("Failed to decode.")]
    Decode(#[source] png::DecodingError),

    #[error("Failed to encode.")]
    Encode(#[source] png::EncodingError),
}

impl Image {
    const CURSOR_STR: &'static str = "[]";

    /// Read image from file.
    /// This function can open PNG whose color type is RGB and color depth is 8-bit.
    pub fn open(path: impl AsRef<Path>) -> Result<Image, Error> {
        dbg!(path.as_ref());

        let file = File::open(&path).map_err(Error::IO)?;
        let decoder = png::Decoder::new(&file);
        let mut reader = decoder.read_info().map_err(Error::Decode)?;
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).unwrap();

        if (info.color_type != png::ColorType::Rgb) || (info.bit_depth != png::BitDepth::Eight) {
            dbg!(info.color_type, info.bit_depth);
            return Err(Error::UnsupportedImgType);
        }

        let (width, height) = (info.width, info.height);
        dbg!(width, height);

        let bytes = &buf[..info.buffer_size()];

        assert_eq!((width * height * 3) as usize, bytes.len());

        // Each pixel is shown by two characters.
        // Normally, the foreground color and background color are the same.
        // The cursor will be shown by setting the foreground color of the corresponding pixel to another color.
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

        file.sync_all().map_err(Error::IO)?;

        Ok(Image {
            path: path.as_ref().to_path_buf(),
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

    pub fn into_text_with_cursor(mut self, cursor_coord: &(usize, usize)) -> Text<'static> {
        if let Color::Rgb(r, g, b) = self.bg_color(cursor_coord) {
            let opposite_color: Color = Rgb(*r, *g, *b).opposite().into();
            *(self.fg_color_mut(cursor_coord)) = opposite_color;
            self.data
        } else {
            unreachable!()
        }
    }

    /// Change color of the pixel at `coord` with `color`.
    pub fn paint(&mut self, color: Rgb, coord: &(usize, usize)) {
        self.assert_coord(coord);
        *self.fg_color_mut(coord) = color.into();
        *self.bg_color_mut(coord) = color.into();
    }

    /// Save the image as a file specified by the path.
    pub fn save_as(&mut self, path: impl AsRef<Path>) -> Result<(), Error> {
        let file = File::create(&path).map_err(Error::IO)?;
        let w = &mut BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, self.width(), self.height());
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header().map_err(Error::Encode)?;
        writer
            .write_image_data(&self.rgb_vec())
            .map_err(Error::Encode)?;

        self.path = path.as_ref().to_path_buf();

        Ok(())
    }

    /// Save the image.
    pub fn save(&self) -> Result<(), Error> {
        let file = File::create(&self.path).map_err(Error::IO)?;
        let w = &mut BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, self.width(), self.height());
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header().map_err(Error::Encode)?;
        writer
            .write_image_data(&self.rgb_vec())
            .map_err(Error::Encode)?;

        Ok(())
    }
}

impl Image {
    /// Check the coordinate is in the image.
    /// If it is not, this function will panic.
    fn assert_coord(&self, coord: &(usize, usize)) {
        assert!(coord.0 < self.width() as usize);
        assert!(coord.1 < self.height() as usize);
    }

    /// The background color of specified coordinate.
    fn bg_color(&self, coord: &(usize, usize)) -> &Color {
        self.assert_coord(coord);
        match self.data.lines[coord.1].0[coord.0].style.bg {
            Some(ref color) => color,
            None => unreachable!(),
        }
    }

    /// The mutable reference to the background color of specified coordinate.
    fn bg_color_mut(&mut self, coord: &(usize, usize)) -> &mut Color {
        self.assert_coord(coord);
        match self.data.lines[coord.1].0[coord.0].style.bg {
            Some(ref mut color) => color,
            None => unreachable!(),
        }
    }

    ///  The mutable reference to the foreground color of specified coordinate.
    fn fg_color_mut(&mut self, coord: &(usize, usize)) -> &mut Color {
        self.assert_coord(coord);
        match self.data.lines[coord.1].0[coord.0].style.fg {
            Some(ref mut color) => color,
            None => unreachable!(),
        }
    }

    /// An array containing a RGB sequence.
    fn rgb_vec(&self) -> Vec<u8> {
        let mut rgb_vec = Vec::with_capacity((self.height() * self.width() * 3) as usize);

        for y in 0..self.height() as usize {
            for x in 0..self.width() as usize {
                let color = self.bg_color(&(x, y));
                if let Color::Rgb(r, g, b) = color {
                    rgb_vec.push(*r);
                    rgb_vec.push(*g);
                    rgb_vec.push(*b);
                } else {
                    unreachable!()
                }
            }
        }
        rgb_vec
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
        let img = Image::open("./tests/image/00.png");
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

    /// This test checks : bg_color, bg_color_mut, fg_color_mut
    #[test]
    fn test_fg_bg() {
        let img = Image::open("./tests/image/00.png");
        assert!(img.is_ok());
        let mut img = img.unwrap();

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
                let coord = (x, y);
                let expected_color: Color = expected_colors[y as usize][x as usize].into();
                assert_eq!(*img.bg_color(&coord), expected_color);
                assert_eq!(*img.bg_color_mut(&coord), expected_color);
                assert_eq!(*img.fg_color_mut(&coord), expected_color);
            }
        }
    }

    /// This test checks whether `Image::read_from_file` return `ImageError::IO` when it passed a path to non-exist file.
    #[test]
    fn test_read_from_error_io() {
        let img = Image::open("./tests/image/non-exist.png");
        assert!(matches!(img, Err(Error::IO(_))));
    }

    /// This test checks whether `Image::read_from_file` return `ImageError::UnsupportedImgType` error when it passed a path to transparent png file.
    #[test]
    fn test_read_from_error_unsupported() {
        let img = Image::open("./tests/image/transparent.png");
        assert!(matches!(img, Err(Error::UnsupportedImgType)));
    }
    /// This test checks whether `Image::read_from_file` return `ImageError::Decode` error when it passed a path to non-png file.
    #[test]
    fn test_read_from_error_decode() {
        let img = Image::open("./tests/image/not-png.txt");
        assert!(matches!(img, Err(Error::Decode(_))));
    }

    #[test]
    fn test_into_text() {
        let img = Image::open("./tests/image/00.png").unwrap();
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
        let img = Image::open("./tests/image/00.png").unwrap();
        let (w, h) = (img.width as usize, img.height as usize);

        let cursor_coord = (3, 1);
        assert!(cursor_coord < (w - 1, h - 1));
        let with_cursor = img.into_text_with_cursor(&cursor_coord);

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

    #[test]
    fn boudary_test_edit() {
        let mut img = Image::open("./tests/image/00.png").unwrap();
        let (w, h) = (img.width as usize, img.height as usize);

        let coord = (w - 1, h - 1);
        img.paint(Rgb(0, 0, 0), &coord);
    }

    #[test]
    fn test_edit() {
        let mut img = Image::open("./tests/image/00.png").unwrap();
        let coord = (img.width as usize - 1, img.height as usize - 1);
        let color = Rgb(12, 23, 34);
        img.paint(color, &coord);
        assert_eq!(*(img.fg_color_mut(&coord)), color.into());
        assert_eq!(*(img.bg_color(&coord)), color.into());
    }

    #[test]
    fn test_save_as() {
        let tmp_path = "./tests/image/image_test_save_as.png";

        // edit and save img.
        let mut img = Image::open("./tests/image/00.png").unwrap();
        let coord = (img.width as usize - 1, img.height as usize - 1);
        let color = Rgb(128, 128, 128);
        img.paint(color, &coord);
        img.save_as(tmp_path).unwrap();
        assert_eq!(img.path, PathBuf::from(tmp_path));

        // check edited img.
        let correct = Image::open("./tests/image/01.png").unwrap();
        let edited = Image::open(tmp_path).unwrap();

        assert_eq!(correct.width, edited.width);
        assert_eq!(correct.height, edited.height);
        assert_eq!(correct.data, edited.data);

        // remove new img.
        std::fs::remove_file(tmp_path).unwrap();
    }

    #[test]
    fn test_save() {
        let tmp_path = "./tests/image/cp_image_test_save.png";

        let original = Image::open("./tests/image/00.png").unwrap();
        std::fs::copy("./tests/image/00.png", tmp_path).unwrap();

        // saving without edit will not change file.
        let copy = Image::open(tmp_path).unwrap();
        copy.save().unwrap();

        let mut copy = Image::open(tmp_path).unwrap();
        assert_eq!(original.width, copy.width);
        assert_eq!(original.height, copy.height);
        assert_eq!(original.data, copy.data);

        // save after edit test.
        let coord = (copy.width as usize - 1, copy.height as usize - 1);
        let color = Rgb(128, 128, 128);
        copy.paint(color, &coord);
        copy.save().unwrap();

        let correct = Image::open("./tests/image/01.png").unwrap();
        let copy = Image::open(tmp_path).unwrap();

        assert_eq!(correct.width, copy.width);
        assert_eq!(correct.height, copy.height);
        assert_eq!(correct.data, copy.data);

        // remove new img.
        std::fs::remove_file(tmp_path).unwrap();
    }
}
