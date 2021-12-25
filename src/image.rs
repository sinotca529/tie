use std::{fs::File, path::Path};
use thiserror::Error;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
struct Rgb(u8, u8, u8);

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Image {
    width: u32,
    height: u32,
    data: Vec<Rgb>,
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

        let data: Vec<Rgb> = bytes
            .chunks(3)
            .map(|rgb: &[u8]| Rgb(rgb[0], rgb[1], rgb[2]))
            .collect();

        file.sync_all().map_err(ImageError::IO)?;

        Ok(Image {
            width,
            height,
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This test checks whether `Image::read_from_file` can parse `./tests/image/01.png`.
    #[test]
    fn test_read_from() {
        let img = Image::read_from_file("./tests/image/00.png");
        assert_eq!(
            img.unwrap(),
            Image {
                width: 5,
                height: 2,
                data: vec![
                    Rgb(237, 28, 36),
                    Rgb(63, 72, 204),
                    Rgb(255, 255, 255),
                    Rgb(255, 255, 255),
                    Rgb(255, 127, 39),
                    Rgb(255, 255, 255),
                    Rgb(255, 255, 255),
                    Rgb(255, 255, 255),
                    Rgb(255, 255, 255),
                    Rgb(255, 242, 0),
                ]
            }
        );
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
}
