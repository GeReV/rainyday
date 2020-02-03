use crate::resources::Resources;
use failure;
use gl;
use std::os::raw;

pub struct TextureLoadOptions<'a> {
    resource_name: &'a str,
    format: gl::types::GLenum,
    pub gen_mipmaps: bool,
}

impl<'a> TextureLoadOptions<'a> {
    pub fn from_res_rgb(resource_name: &str) -> TextureLoadOptions {
        TextureLoadOptions {
            resource_name,
            format: gl::RGB,
            gen_mipmaps: false,
        }
    }

    pub fn from_res_rgba(resource_name: &str) -> TextureLoadOptions {
        TextureLoadOptions {
            resource_name,
            format: gl::RGBA,
            gen_mipmaps: false,
        }
    }
}

pub struct TextureLoadBuilder<'a> {
    options: TextureLoadOptions<'a>,
}

impl<'a> TextureLoadBuilder<'a> {
    pub fn load(self, gl: &gl::Gl, res: &Resources) -> Result<Texture, failure::Error> {
        Texture::from_res(self.options, gl, res)
    }

    pub fn with_gen_mipmaps(mut self) -> Self {
        self.options.gen_mipmaps = true;
        self
    }
}

pub struct Texture {
    gl: gl::Gl,
    obj: gl::types::GLuint,
    width: u32,
    height: u32,
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { self.gl.DeleteTextures(1, &mut self.obj) };
    }
}

impl Texture {
    pub fn new(gl: &gl::Gl, width: u32, height: u32) -> Result<Texture, failure::Error> {
        let mut obj: gl::types::GLuint = 0;
        unsafe {
            gl.GenTextures(1, &mut obj);
        }

        let texture = Texture {
            gl: gl.clone(),
            obj,
            width,
            height,
        };

        unsafe {
            gl.BindTexture(gl::TEXTURE_2D, obj);
        }

        unsafe {
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_BASE_LEVEL, 0);
            gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 0);
            //            gl.TexParameteri(
            //                gl::TEXTURE_2D,
            //                gl::TEXTURE_MIN_FILTER,
            //                gl::LINEAR.try_into().unwrap(),
            //            );
            //            gl.TexParameteri(
            //                gl::TEXTURE_2D,
            //                gl::TEXTURE_MAG_FILTER,
            //                gl::LINEAR.try_into().unwrap(),
            //            );
            gl.TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as gl::types::GLint,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                0 as *const raw::c_void,
            );
        }

        unsafe {
            gl.BindTexture(gl::TEXTURE_2D, 0);
        }

        Ok(texture)
    }

    pub fn from_res_rgb(resource_name: &str) -> TextureLoadBuilder {
        TextureLoadBuilder {
            options: TextureLoadOptions::from_res_rgb(resource_name),
        }
    }

    pub fn from_res_rgba(resource_name: &str) -> TextureLoadBuilder {
        TextureLoadBuilder {
            options: TextureLoadOptions::from_res_rgba(resource_name),
        }
    }

    pub fn from_res<'a>(
        options: TextureLoadOptions<'a>,
        gl: &gl::Gl,
        res: &Resources,
    ) -> Result<Texture, failure::Error> {
        let mut obj: gl::types::GLuint = 0;
        unsafe {
            gl.GenTextures(1, &mut obj);
        }

        let mut texture = Texture {
            gl: gl.clone(),
            obj,
            width: 0,
            height: 0,
        };

        texture.update(options, res)?;

        Ok(texture)
    }

    pub fn update<'a>(
        &mut self,
        options: TextureLoadOptions<'a>,
        res: &Resources,
    ) -> Result<(), failure::Error> {
        let gl = &self.gl;

        unsafe {
            gl.BindTexture(gl::TEXTURE_2D, self.obj);
        }

        // https://www.khronos.org/opengl/wiki/Common_Mistakes

        match options.format {
            gl::RGB => {
                let img = res.load_rgb_image(options.resource_name)?;

                let dims = img.dimensions();

                self.width = dims.0;
                self.height = dims.1;

                if options.gen_mipmaps {
                    unsafe {
                        gl.TexImage2D(
                            gl::TEXTURE_2D,
                            0,
                            gl::RGB8 as gl::types::GLint,
                            img.width() as i32,
                            img.height() as i32,
                            0,
                            gl::RGB,
                            gl::UNSIGNED_BYTE,
                            img.as_ptr() as *const raw::c_void,
                        );
                        gl.GenerateMipmap(gl::TEXTURE_2D);
                    }
                } else {
                    unsafe {
                        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_BASE_LEVEL, 0);
                        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 0);
                        gl.TexImage2D(
                            gl::TEXTURE_2D,
                            0,
                            gl::RGB8 as gl::types::GLint,
                            img.width() as i32,
                            img.height() as i32,
                            0,
                            gl::RGB,
                            gl::UNSIGNED_BYTE,
                            img.as_ptr() as *const raw::c_void,
                        );
                    }
                }
            }
            gl::RGBA => {
                let img = res.load_rgba_image(options.resource_name)?;

                let dims = img.dimensions();

                self.width = dims.0;
                self.height = dims.1;

                if options.gen_mipmaps {
                    unsafe {
                        gl.TexImage2D(
                            gl::TEXTURE_2D,
                            0,
                            gl::RGBA8 as gl::types::GLint,
                            img.width() as i32,
                            img.height() as i32,
                            0,
                            gl::RGBA,
                            gl::UNSIGNED_BYTE,
                            img.as_ptr() as *const raw::c_void,
                        );
                        gl.GenerateMipmap(gl::TEXTURE_2D);
                    }
                } else {
                    unsafe {
                        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_BASE_LEVEL, 0);
                        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 0);
                        gl.TexImage2D(
                            gl::TEXTURE_2D,
                            0,
                            gl::RGBA8 as gl::types::GLint,
                            img.width() as i32,
                            img.height() as i32,
                            0,
                            gl::RGBA,
                            gl::UNSIGNED_BYTE,
                            img.as_ptr() as *const raw::c_void,
                        );
                    }
                }
            }
            _ => unreachable!("Only RGB or RGBA images can be constructed"),
        }

        unsafe {
            gl.BindTexture(gl::TEXTURE_2D, 0);
        }

        Ok(())
    }

    pub fn bind(&self) {
        unsafe {
            self.gl.BindTexture(gl::TEXTURE_2D, self.obj);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            self.gl.BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    pub fn bind_at(&self, index: u32) {
        unsafe {
            self.gl.ActiveTexture(gl::TEXTURE0 + index);
        }
        self.bind();
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.obj
    }
}
