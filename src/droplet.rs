use nalgebra::Vector2;
use ncollide2d::pipeline::CollisionObjectSlabHandle;

pub struct Droplet {
    pub pos: Vector2<f32>,
    pub size: f32,
    pub speed: Vector2<f32>,
    pub seed: i32,
    pub skipping: bool,
    pub deleted: bool,
    pub slowing: bool,
    pub collision_handle: CollisionObjectSlabHandle,
    pub last_trail_y: Option<f32>,
}

impl Droplet {
    pub fn new() -> Droplet {
        Droplet {
            pos: Vector2::default(),
            size: 1.0,
            speed: Vector2::default(),
            seed: 0,
            skipping: false,
            deleted: false,
            slowing: false,
            collision_handle: CollisionObjectSlabHandle(0),
            last_trail_y: None,
        }
    }
}
