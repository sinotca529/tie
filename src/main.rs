use app::App;
use command::keyinput::KeyInput;

use crate::image::Image;

mod app;
mod command;
mod image;
mod widget;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Please send a png file's path");
        return;
    }

    let img_path = &args[1];

    let img = Image::open(img_path).unwrap();
    App::new(img, KeyInput::new()).run().unwrap();
}
