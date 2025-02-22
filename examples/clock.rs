use std::{
    fs::File,
    io::{Read, Write},
};

use geometry::{Vert3, Vert4};
use rust_ray_tracer::canvas::{Canvas, Color, PPMReader};

fn main() {
    let width = 100;
    let height = 100;

    let calc_y = |y: f32| (height as f32 / 2.) + ((width as f32 * 0.33) * y);

    let calc_x = |x: f32| (width as f32 / 2.) + ((width as f32 * 0.33) * x);

    let mut c = Canvas::new(width, height);
    for vert in geometry::clock(Vert4::point(0., 0., 1.)) {
        let pixel_coords = (calc_x(vert.x()) as usize, calc_y(vert.z()) as usize);

        println!("{vert:?}");
        c.write_pixel(pixel_coords, Color(Vert3::new(1., 1., 1.)));
    }

    let mut file = File::create("Clock.ppm").expect("fuck");
    file.flush().unwrap();
    let mut buf = vec![];
    PPMReader::new(&c).unwrap().read_to_end(&mut buf).unwrap();
    file.write_all(&buf).unwrap()
}
