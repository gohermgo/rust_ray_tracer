use geometry::Vert4;
use geometry::ops::Norm;
use rust_ray_tracer::{Body, Environment, Force, Projectile};

fn main() {
    let mut p = Projectile(Body::new(
        Vert4::point(0., 1., 0.),
        Vert4::vector(1., 1., 0.).norm(),
    ));
    let e = Environment {
        gravity: Force::new(Vert4::vector(0., -0.1, 0.)),
        wind: Force::new(Vert4::vector(0.01, 0., 0.)),
    };
    while p.0.p_position.y() > 0. {
        println!("Pos: {:?}, Vel: {:?}", p.0.p_position, p.0.v_velocity);
        e.tick(&mut p);
    }
}
