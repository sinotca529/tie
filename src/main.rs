use app::App;

use crate::image::Image;

mod app;
mod image;

fn main() {
    // let text: Text = Image::read_from_file("./tests/image/00.png").unwrap().into();
    // dbg!(&text);
    let img = Image::read_from_file("tests/image/00.png").unwrap();
    App::new(&img).run().unwrap();
}
