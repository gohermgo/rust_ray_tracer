use geometry::Vert4;

pub mod canvas;
pub struct Body {
    pub p_position: Vert4,
    pub v_velocity: Vert4,
}
impl Body {
    #[inline]
    pub const fn new(position: Vert4, velocity: Vert4) -> Body {
        debug_assert!(velocity.w() == 0.);
        debug_assert!(position.w() == 1.);
        Body {
            p_position: position,
            v_velocity: velocity,
        }
    }
}
#[repr(transparent)]
pub struct Projectile(pub Body);
#[repr(transparent)]
pub struct Force(Vert4);
impl Force {
    #[inline]
    pub const fn new(f: Vert4) -> Force {
        debug_assert!(f.w() == 0.);
        Force(f)
    }
    pub fn apply_to_body(&self, body: &mut Body) {
        body.v_velocity += &self.0
    }
}
pub struct Environment {
    pub gravity: Force,
    pub wind: Force,
}
impl Environment {
    pub fn tick(&self, Projectile(projectile_body): &mut Projectile) {
        projectile_body.p_position += &projectile_body.v_velocity;
        Force(&self.wind.0 + &self.gravity.0).apply_to_body(projectile_body);
    }
}
