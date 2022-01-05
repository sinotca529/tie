use app::App;
use command::{keyinput::KeyInput, CommandStream};

use crate::image::Image;

mod app;
mod command;
mod image;
mod widget;

#[derive(thiserror::Error, Debug)]
pub enum Error<E: 'static + std::error::Error + std::fmt::Debug> {
    #[error("Error occurred in the app.")]
    App(#[source] crate::app::Error<E>),

    #[error("Error occurred in Image.")]
    Image(#[source] crate::image::Error),

    #[error("Incorrect argument: `{0}`")]
    Arg(String),
}

fn main() -> Result<(), Error<<KeyInput as CommandStream>::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return Err(Error::Arg("Please send a png file's path".into()));
    }

    let img_path = &args[1];

    let img = Image::open(img_path).map_err(Error::Image)?;
    App::new(img, KeyInput::new()).run().map_err(Error::App)?;

    Ok(())
}
