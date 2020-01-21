pub struct Droplet {
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub x_speed: f32,
    pub y_speed: f32,
    pub seed: i32,
    pub collided: bool,
    pub skipping: bool,
    pub slowing: bool,
}
