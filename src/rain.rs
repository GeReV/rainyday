use crate::background::Background;
use crate::config::Config;
use crate::droplets::Droplets;
use crate::quad::Quad;
use crate::render_gl;
use crate::render_gl::buffer::ArrayBuffer;
use crate::render_gl::{
    ColorBuffer, Error, FrameBuffer, Program, Shader, Texture, TextureLoadOptions, Viewport,
};
use image::GenericImageView;
use nalgebra as na;
use nalgebra::{Matrix4, Orthographic3, Point3, Translation3, Vector2, Vector3, Vector4};
use ncollide2d::na::Isometry2;
use ncollide2d::pipeline::{CollisionGroups, CollisionObjectSlabHandle, GeometricQueryType};
use ncollide2d::query::Proximity;
use ncollide2d::shape::{Ball, ShapeHandle};
use ncollide2d::world::CollisionWorld;
use rand::prelude::*;
use std::rc::Rc;
use std::time::Duration;

const VIEW_DISTANCE: f32 = 10.0;

const DROPLETS_PER_SECOND: usize = 50;

const DROPLET_SIZE_GRAVITY_THRESHOLD: f32 = 5.0;
const PRIVATE_GRAVITY_FORCE_FACTOR_Y: f32 = 0.25;
const PRIVATE_GRAVITY_FORCE_FACTOR_X: f32 = 0.0;

const DROP_VERT: &str = include_str!("../assets/shaders/drop.vert");
const DROP_FRAG: &str = include_str!("../assets/shaders/drop.frag");
const DROP_WIPE_VERT: &str = include_str!("../assets/shaders/drop_wipe.vert");
const DROP_WIPE_FRAG: &str = include_str!("../assets/shaders/drop_wipe.frag");
const COLORED_QUAD_VERT: &str = include_str!("../assets/shaders/colored_quad.vert");
const COLORED_QUAD_FRAG: &str = include_str!("../assets/shaders/colored_quad.frag");
const QUAD_VERT: &str = include_str!("../assets/shaders/quad.vert");
const FINAL_FRAG: &str = include_str!("../assets/shaders/final.frag");

fn load_shader(gl: &gl::Gl, vert_source: &str, frag_source: &str, debug_name: &str) -> Program {
    Program::from_shaders(
        &gl,
        &[
            Shader::from_vert_source_str(&gl, vert_source).unwrap(),
            Shader::from_frag_source_str(&gl, frag_source).unwrap(),
        ],
    )
    .map_err(|msg| Error::LinkError {
        message: msg,
        name: debug_name.to_string(),
    })
    .unwrap()
}

pub struct Rain {
    gl: gl::Gl,

    max_droplet_count: usize,
    droplet_size_range: (f32, f32),

    updates: Vec<(CollisionObjectSlabHandle, CollisionObjectSlabHandle)>,

    world: CollisionWorld<f32, usize>,

    collision_group: CollisionGroups,
    contacts_query: GeometricQueryType<f32>,

    viewport: Viewport,

    view_matrix: Matrix4<f32>,
    projection_matrix: Matrix4<f32>,

    time_accumulator: f64,
    droplets_accumulator: usize,

    droplets: Droplets,

    black_color_buffer: ColorBuffer,

    background_texture: Rc<Texture>,
    background_mask: Texture,
    background_buffer: Texture,

    background: Background,
    drop_quad: Quad,
    fullscreen_quad: Quad,

    drop_program: Program,
    drop_wipe_program: Program,
    colored_quad_program: Program,
    final_program: Program,

    frame_buffer: FrameBuffer,
}

impl Rain {
    pub fn new(
        gl: &gl::Gl,
        max_droplet_count: usize,
        droplet_size_range: (f32, f32),
        window_size: (u32, u32),
        config: &Config,
    ) -> Result<Self, failure::Error> {
        let mut droplets: Droplets = Droplets::with_capacity(max_droplet_count);

        let world = CollisionWorld::new(2.0);

        let collision_group = CollisionGroups::new();
        let contacts_query = GeometricQueryType::Proximity(0.0);

        let mut viewport = Viewport::for_window(window_size.0 as i32, window_size.1 as i32);

        viewport.set_used(&gl);

        let view_matrix: Matrix4<f32> = (Translation3::<f32>::from(Point3::origin().coords)
            * Translation3::<f32>::from(Vector3::z() * VIEW_DISTANCE))
        .inverse()
        .to_homogeneous();

        let mut projection_matrix: Matrix4<f32> = Orthographic3::new(
            0.0,
            window_size.0 as f32,
            0.0,
            window_size.1 as f32,
            0.01,
            1000.0,
        )
        .into_inner();

        let background_texture = {
            let path = match config.background() {
                Some(path) => path,
                None => {
                    let current_exe = std::env::current_exe().unwrap();
                    let mut dir = current_exe.parent().unwrap();

                    dir.join("assets\\textures\\background.jpg")
                        .to_str()
                        .unwrap()
                        .to_string()
                }
            };

            let mut options = TextureLoadOptions::from_res_rgb(&path);
            options.gen_mipmaps = true;

            let mut image = image::open(&path).unwrap();

            let texture_dimensions = image.dimensions();
            let (screen_width, screen_height) = window_size;

            let screen_ratio = screen_width as f32 / screen_height as f32;
            let texture_ratio = texture_dimensions.0 as f32 / texture_dimensions.1 as f32;

            let target_dimensions = if screen_ratio < texture_ratio {
                let ratio = screen_ratio / texture_ratio;
                (
                    texture_dimensions.0 as f32 * ratio,
                    texture_dimensions.1 as f32,
                )
            } else {
                let ratio = texture_ratio / screen_ratio;
                (
                    texture_dimensions.0 as f32,
                    texture_dimensions.1 as f32 * ratio,
                )
            };

            let offsets = (
                (texture_dimensions.0 as f32 - target_dimensions.0) * 0.5,
                (texture_dimensions.1 as f32 - target_dimensions.1) * 0.5,
            );

            let x = offsets.0;
            let y = offsets.1;
            let width = texture_dimensions.0 as f32 - offsets.0;
            let height = texture_dimensions.1 as f32 - offsets.1;

            let image = image.crop(x as u32, y as u32, (width - x) as u32, (height - y) as u32);

            Texture::from_image(options, &gl, &image).unwrap()
        };

        let texture_rc = Rc::<Texture>::new(background_texture);

        let drop_quad = Quad::default(&gl);

        let background =
            Background::new(&gl, texture_rc.clone(), window_size.0, window_size.1, 1.0)?;

        let drop_program = load_shader(&gl, DROP_VERT, DROP_FRAG, "drop");

        let drop_wipe_program = load_shader(&gl, DROP_WIPE_VERT, DROP_WIPE_FRAG, "drop_wipe");

        let colored_quad_program =
            load_shader(&gl, COLORED_QUAD_VERT, COLORED_QUAD_FRAG, "colored_quad");

        let final_program = load_shader(&gl, QUAD_VERT, FINAL_FRAG, "final");

        let background_mask = Texture::new(&gl, window_size.0, window_size.1)?;

        let background_buffer = Texture::new(&gl, window_size.0, window_size.1)?;

        let fullscreen_quad =
            Quad::new_with_size(&gl, 0.0, 0.0, window_size.1 as f32, window_size.0 as f32);

        let black_color_buffer = ColorBuffer::from_rgba(0.0, 0.0, 0.0, 1.0);

        let frame_buffer = FrameBuffer::new(&gl);

        {
            frame_buffer.bind();
            frame_buffer.attach_texture(&background_mask);

            black_color_buffer.set_used(&gl);
            black_color_buffer.clear(&gl);

            frame_buffer.unbind();
        }

        Ok(Rain {
            gl: gl.clone(),

            max_droplet_count,
            droplet_size_range,

            updates: Vec::<(CollisionObjectSlabHandle, CollisionObjectSlabHandle)>::new(),

            viewport,

            time_accumulator: 0.0,
            droplets_accumulator: DROPLETS_PER_SECOND,

            droplets,

            world,
            collision_group,
            contacts_query,

            view_matrix,
            projection_matrix,

            black_color_buffer,

            background_texture: texture_rc,
            background_mask,
            background_buffer,

            background,
            drop_quad,
            fullscreen_quad,

            drop_program,
            drop_wipe_program,
            colored_quad_program,
            final_program,

            frame_buffer,
        })
    }

    pub fn update(&mut self, delta: &Duration) {
        let mut rng = rand::thread_rng();

        self.time_accumulator += delta.as_secs_f64();

        if self.time_accumulator > 1.0 {
            self.time_accumulator -= 1.0;

            self.droplets_accumulator += DROPLETS_PER_SECOND;
        }

        // Updates
        {
            Self::gravity_non_linear(&mut self.droplets, &mut self.world, &mut rng, delta);

            Self::trail(
                &mut self.droplets,
                &mut self.world,
                &mut rng,
                &self.collision_group,
                &self.contacts_query,
                &delta,
            );

            self.updates.clear();

            // We get an "allowance" of DROPLETS_PER_SECOND every second.
            // This part of the loop will attempt to spend them at random times, and is more likely to
            // spend them the more time has past.
            // TODO: Any better way to spend these more evenly?
            // TODO: What happens when budget > fps?
            if self.droplets_accumulator > 0
                && rng.gen_bool(self.time_accumulator.max(0.0).min(1.0))
            {
                if let Some((i, d)) = self.droplets.checkout() {
                    d.pos = Vector2::new(
                        rng.gen_range(0.0, self.viewport.w as f32),
                        rng.gen_range(0.0, self.viewport.h as f32),
                    );
                    d.size = rng.gen_range(self.droplet_size_range.0, self.droplet_size_range.1);

                    let shape_handle = ShapeHandle::new(Ball::new(d.size * 0.5));

                    let handle = self
                        .world
                        .add(
                            Isometry2::new(d.pos.clone_owned(), na::zero()),
                            shape_handle,
                            self.collision_group,
                            self.contacts_query,
                            i,
                        )
                        .0;

                    d.collision_handle = handle;

                    self.droplets_accumulator -= 1;
                }
            }

            for ev in self.world.proximity_events().iter().collect::<Vec<_>>() {
                if ev.new_status == Proximity::Intersecting {
                    if let (Some(obj1), Some(obj2)) = (
                        self.world.collision_object(ev.collider1),
                        self.world.collision_object(ev.collider2),
                    ) {
                        let sphere1 = obj1.shape().local_bounding_sphere();
                        let sphere2 = obj2.shape().local_bounding_sphere();

                        let rad1 = sphere1.radius();
                        let rad2 = sphere2.radius();

                        let pair = if rad1 > rad2 {
                            (ev.collider1, ev.collider2)
                        } else if rad1 < rad2 {
                            (ev.collider2, ev.collider1)
                        } else if sphere1.center().y > sphere2.center().y {
                            (ev.collider1, ev.collider2)
                        } else {
                            (ev.collider2, ev.collider1)
                        };

                        self.updates.push(pair);
                    }
                }
            }

            for (keep_handle, delete_handle) in self.updates.iter() {
                if let (Some(keep), Some(delete)) = self
                    .world
                    .collision_object_pair_mut(*keep_handle, *delete_handle)
                {
                    let keep_droplet_index = *keep.data();
                    let delete_droplet_index = *delete.data();

                    let delete_droplet_size = self.droplets[delete_droplet_index].size;

                    let keep_droplet = &mut self.droplets[keep_droplet_index];

                    // TODO: How much does a droplet grow when is absorbs another?
                    keep_droplet.size = ((keep_droplet.size * 0.5).powf(3.0)
                        + (delete_droplet_size * 0.5).powf(3.0))
                    .cbrt()
                        * 2.0;

                    keep.set_shape(ShapeHandle::new(Ball::new(keep_droplet.size * 0.5)));
                }
            }

            for (_, delete_handle) in self.updates.iter() {
                if let Some(delete) = self.world.collision_object(*delete_handle) {
                    self.droplets.free(*delete.data());
                    self.world.remove(&[*delete_handle]);
                }
            }
        }
    }

    pub fn render(&self, delta: &Duration) {
        let matrix = &self.projection_matrix * &self.view_matrix;

        let resolution = Vector2::new(self.viewport.w as f32, self.viewport.h as f32);

        // Background pass
        {
            self.background.prepass(
                &self.gl,
                &self.view_matrix,
                &self.projection_matrix,
                &resolution,
            );

            self.frame_buffer.bind();
            self.frame_buffer.attach_texture(&self.background_buffer);

            self.background.render(
                &self.gl,
                &self.view_matrix,
                &self.projection_matrix,
                &resolution,
            );

            self.frame_buffer.unbind();
        }

        // Mask pass
        {
            self.frame_buffer.bind();
            self.frame_buffer.attach_texture(&self.background_mask);

            unsafe {
                self.gl.BlendFuncSeparate(
                    gl::SRC_ALPHA,
                    gl::ONE_MINUS_SRC_ALPHA,
                    gl::ZERO,
                    gl::ONE,
                );
            }

            {
                self.colored_quad_program.set_used();

                if let Some(loc) = self.colored_quad_program.get_uniform_location("MVP") {
                    self.colored_quad_program
                        .set_uniform_matrix_4fv(loc, &matrix);
                }

                if let Some(loc) = self.colored_quad_program.get_uniform_location("Color") {
                    self.colored_quad_program.set_uniform_4f(
                        loc,
                        &Vector4::new(0.0, 0.0, 0.0, 0.25 * delta.as_secs_f32()),
                    );
                }

                self.fullscreen_quad.render(&self.gl);
            }

            {
                self.drop_wipe_program.set_used();

                if let Some(loc) = self.drop_wipe_program.get_uniform_location("Resolution") {
                    self.drop_wipe_program.set_uniform_2f(loc, &resolution);
                }

                if let Some(loc) = self.drop_wipe_program.get_uniform_location("MVP") {
                    self.drop_wipe_program.set_uniform_matrix_4fv(loc, &matrix);
                }

                self.render_droplets(&self.gl, &self.drop_quad, &self.droplets);
            }

            unsafe {
                self.gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            }

            self.frame_buffer.unbind();
        }

        // Merge pass
        {
            self.final_program.set_used();

            self.black_color_buffer.set_used(&self.gl);
            self.black_color_buffer.clear(&self.gl);

            if let Some(loc) = self.final_program.get_uniform_location("MVP") {
                self.final_program.set_uniform_matrix_4fv(loc, &matrix);
            }

            if let Some(loc) = self.final_program.get_uniform_location("Texture0") {
                self.background_buffer.bind_at(0);
                self.final_program.set_uniform_1i(loc, 0);
            }
            if let Some(loc) = self.final_program.get_uniform_location("Texture1") {
                self.background_texture.bind_at(1);
                self.final_program.set_uniform_1i(loc, 1);
            }
            if let Some(loc) = self.final_program.get_uniform_location("Mask") {
                self.background_mask.bind_at(2);
                self.final_program.set_uniform_1i(loc, 2);
            }

            self.fullscreen_quad.render(&self.gl);
        }

        {
            self.drop_program.set_used();

            if let Some(loc) = self.drop_program.get_uniform_location("Resolution") {
                self.drop_program.set_uniform_2f(loc, &resolution);
            }

            if let Some(loc) = self.drop_program.get_uniform_location("MVP") {
                self.drop_program.set_uniform_matrix_4fv(loc, &matrix);
            }

            if let Some(loc) = self.drop_program.get_uniform_location("Texture") {
                self.background_texture.bind_at(0);
                self.drop_program.set_uniform_1i(loc, 0);
            }

            self.render_droplets(&self.gl, &self.drop_quad, &self.droplets);
        }
    }

    fn render_droplets(&self, gl: &gl::Gl, quad: &Quad, droplets: &Droplets) {
        quad.vao.bind();

        let instance_vbo: ArrayBuffer = ArrayBuffer::new(&gl);
        instance_vbo.bind();

        let offsets: Vec<na::Vector3<f32>> = droplets
            .into_iter()
            .filter(|d| !d.deleted)
            .map(|d| na::Vector3::new(d.pos.x, d.pos.y, d.size))
            .collect();

        instance_vbo.static_draw_data(&offsets);

        unsafe {
            gl.EnableVertexAttribArray(3);
            gl.VertexAttribPointer(
                3,
                3,         // the number of components per generic vertex attribute
                gl::FLOAT, // data type
                gl::FALSE,
                std::mem::size_of::<na::Vector3<f32>>() as gl::types::GLint,
                0 as *const gl::types::GLvoid,
            );
        }
        instance_vbo.unbind();

        unsafe {
            gl.VertexAttribDivisor(3, 1);
        }

        unsafe {
            gl.DrawElementsInstanced(
                gl::TRIANGLES,
                6,
                gl::UNSIGNED_BYTE,
                ::std::ptr::null(),
                offsets.len() as i32,
            );
        }
        quad.vao.unbind();
    }

    fn gravity_non_linear(
        droplets: &mut Droplets,
        world: &mut CollisionWorld<f32, usize>,
        rng: &mut ThreadRng,
        dt: &Duration,
    ) {
        let fps = 1.0 / dt.as_secs_f32();
        let gravity_y = PRIVATE_GRAVITY_FORCE_FACTOR_Y * dt.as_secs_f32();

        for i in 0..droplets.len() {
            let mut delete_index: Option<usize> = None;

            {
                let droplet = &mut droplets[i];

                if droplet.deleted || droplet.size < DROPLET_SIZE_GRAVITY_THRESHOLD {
                    continue;
                }

                if droplet.size < DROPLET_SIZE_GRAVITY_THRESHOLD && droplet.seed > 0 {
                    droplet.slowing = true;
                }

                let movement_probability = 0.01 * dt.as_secs_f64();

                if droplet.seed <= 0 {
                    droplet.seed =
                        (droplet.size * 0.5 * rng.gen_range(0.0, 1.0) * fps).floor() as i32;
                    droplet.skipping = droplet.skipping == false;
                    droplet.slowing = true;
                }

                droplet.seed -= 1;

                assert!(droplet.size >= 1.0);

                if droplet.speed.y > 0.0 {
                    if droplet.slowing {
                        droplet.speed *= 0.9;
                        if droplet.speed.y < gravity_y {
                            droplet.slowing = false;
                        }
                    } else if droplet.skipping {
                        droplet.speed.y = gravity_y;
                        droplet.speed.x = PRIVATE_GRAVITY_FORCE_FACTOR_X;
                    } else {
                        droplet.speed.y += gravity_y * droplet.size;
                        droplet.speed.x += PRIVATE_GRAVITY_FORCE_FACTOR_X * droplet.size;
                    }
                } else if rng.gen_bool((1.0 - 1.0 / droplet.size as f64) * movement_probability) {
                    droplet.speed.y = gravity_y;
                    droplet.speed.x = PRIVATE_GRAVITY_FORCE_FACTOR_X;
                }

                //        if this.options.gravityAngleVariance != 0 {
                //            droplet.x_speed +=
                //                (rnd.gen() * 2 - 1) * droplet.y_speed * this.options.gravityAngleVariance
                //        }

                droplet.pos.y -= droplet.speed.y;
                droplet.pos.x += droplet.speed.x;

                if droplet.pos.y + droplet.size * 0.5 < 0.0 {
                    delete_index = Some(i);

                    world.remove(&[droplet.collision_handle]);
                } else if droplet.speed.x != 0.0 || droplet.speed.y != 0.0 {
                    let handle = droplet.collision_handle;

                    let object = world.get_mut(handle).unwrap();

                    object.set_position(Isometry2::new(droplet.pos.clone_owned(), na::zero()));
                }
            }

            if let Some(delete_index) = delete_index {
                droplets.free(delete_index);
            }
        }

        world.update();
    }

    fn trail(
        droplets: &mut Droplets,
        world: &mut CollisionWorld<f32, usize>,
        rng: &mut ThreadRng,
        collision_group: &CollisionGroups,
        contacts_query: &GeometricQueryType<f32>,
        dt: &Duration,
    ) {
        let gravity_y = PRIVATE_GRAVITY_FORCE_FACTOR_Y * dt.as_secs_f32();

        for i in 0..droplets.len() {
            let pos;
            let size;

            {
                let droplet = &mut droplets[i];

                if droplet.speed.y <= gravity_y {
                    continue;
                }

                if droplet.size >= 6.0
                    && (droplet.last_trail_y.is_none()
                        || (droplet.last_trail_y.unwrap_or(0.0) - droplet.pos.y)
                            >= rng.gen_range(0.1, 1.0) * 200.0)
                {
                    droplet.last_trail_y = Some(droplet.pos.y);

                    size = rng.gen_range(0.9, 1.1) * droplet.size * 0.25;
                    pos = Vector2::new(
                        droplet.pos.x + rng.gen_range(-1.0, 1.0),
                        droplet.pos.y + droplet.size * 0.5 + droplet.speed.y + size * 0.5,
                    );

                    droplet.size =
                        ((droplet.size * 0.5).powf(3.0) - (size * 0.5).powf(3.0)).cbrt() * 2.0;

                    if let Some(droplet_collision) = world.get_mut(droplet.collision_handle) {
                        droplet_collision.set_shape(ShapeHandle::new(Ball::new(droplet.size * 0.5)))
                    }
                } else {
                    continue;
                }
            }

            if let Some((i, d)) = droplets.checkout() {
                d.pos = pos;
                d.size = size;

                let shape_handle = ShapeHandle::new(Ball::new(d.size * 0.5));

                let handle = world
                    .add(
                        Isometry2::new(d.pos.clone_owned(), na::zero()),
                        shape_handle,
                        *collision_group,
                        *contacts_query,
                        i,
                    )
                    .0;

                d.collision_handle = handle;
            }
        }
    }
}
