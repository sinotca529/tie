use app::App;
use command::keyinput::KeyInput;

use crate::image::Image;

mod app;
mod command;
mod image;

fn main() {
    // let text: Text = Image::read_from_file("./tests/image/00.png").unwrap().into();
    // dbg!(&text);
    let img = Image::read_from_file("tests/image/00.png").unwrap();
    App::new(&img, KeyInput).run().unwrap();
}
