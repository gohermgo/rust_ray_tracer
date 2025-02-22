use std::{
    fs::File,
    io::{Read, Write},
};

use geometry::ops::Norm;
use geometry::{Vert3, Vert4};
use rust_ray_tracer::{
    Body, Environment, Force, Projectile,
    canvas::{Canvas, Color, PPMReader},
};

fn main() {
    let start = Vert4::point(0., 1., 0.);
    let velocity = Vert4::vector(1., 1.8, 0.).norm() * 11.25;
    let mut p = Projectile(Body::new(start, velocity));

    let e = Environment {
        gravity: Force::new(Vert4::vector(0., -0.1, 0.)),
        wind: Force::new(Vert4::vector(-0.01, 0., 0.)),
    };

    let height = 550;
    let calc_y = |y: usize| height - y;
    let mut c = Canvas::new(900, 550);

    while p.0.p_position.y() > 0. {
        c.write_pixel(
            (
                p.0.p_position.x() as usize,
                calc_y(p.0.p_position.y() as usize),
            ),
            Color(Vert3::X),
        );
        // println!("Pos: {:?}, Vel: {:?}", p.0.p_position, p.0.v_velocity);
        e.tick(&mut p);
    }

    let mut file = File::create("Projectile.ppm").expect("fuck");
    file.flush().unwrap();
    let mut buf = vec![];
    PPMReader::new(&c).unwrap().read_to_end(&mut buf).unwrap();
    file.write_all(&buf).unwrap()
}
