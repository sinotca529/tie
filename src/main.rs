use app::{App, AppError};
use command::{keyinput::KeyInput, CommandStream};
use image::ImageError;
use thiserror::Error;

use crate::image::Image;

mod app;
mod command;
mod image;
mod widget;

#[derive(Error, Debug)]
pub enum MainError<E: 'static + std::error::Error + std::fmt::Debug> {
    #[error("Error occurred in the app.")]
    AppError(#[source] AppError<E>),

    #[error("Error occurred in Image.")]
    ImageError(#[source] ImageError),

    #[error("Incorrect argument: `{0}`")]
    ArgError(String),
}

fn main() -> Result<(), MainError<<KeyInput as CommandStream>::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return Err(MainError::ArgError("Please send a png file's path".into()));
    }

    let img_path = &args[1];

    let img = Image::open(img_path).map_err(MainError::ImageError)?;
    App::new(img, KeyInput::new())
        .run()
        .map_err(MainError::AppError)?;

    Ok(())
}
