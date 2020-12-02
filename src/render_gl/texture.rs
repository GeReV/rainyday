use std::os::raw;

pub struct TextureLoadOptions {
    format: gl::types::GLenum,
    pub gen_mipmaps: bool,
}

impl TextureLoadOptions {
    pub fn rgb() -> TextureLoadOptions {
        TextureLoadOptions {
            format: gl::RGB,
            gen_mipmaps: false,
        }
    }

    pub fn rgba() -> TextureLoadOptions {
        TextureLoadOptions {
            format: gl::RGBA,
            gen_mipmaps: false,
        }
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
        unsafe { self.gl.DeleteTextures(1, &self.obj) };
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
                gl::RGBA16 as gl::types::GLint,
                width as i32,
                height as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                std::ptr::null(),
            );
        }

        unsafe {
            gl.BindTexture(gl::TEXTURE_2D, 0);
        }

        Ok(texture)
    }

    pub fn from_image(
        options: TextureLoadOptions,
        gl: &gl::Gl,
        image: &image::DynamicImage,
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

        unsafe {
            gl.BindTexture(gl::TEXTURE_2D, texture.obj);
        }

        // https://www.khronos.org/opengl/wiki/Common_Mistakes

        match options.format {
            gl::RGB => {
                let img = image.flipv().to_rgb();

                let dims = img.dimensions();

                texture.width = dims.0;
                texture.height = dims.1;

                if options.gen_mipmaps {
                    unsafe {
                        gl.TexImage2D(
                            gl::TEXTURE_2D,
                            0,
                            gl::RGB8 as gl::types::GLint,
                            texture.width as i32,
                            texture.height as i32,
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
                            texture.width as i32,
                            texture.height as i32,
                            0,
                            gl::RGB,
                            gl::UNSIGNED_BYTE,
                            img.as_ptr() as *const raw::c_void,
                        );
                    }
                }
            }
            gl::RGBA => {
                let img = image.flipv().to_rgba();

                let dims = img.dimensions();

                texture.width = dims.0;
                texture.height = dims.1;

                if options.gen_mipmaps {
                    unsafe {
                        gl.TexImage2D(
                            gl::TEXTURE_2D,
                            0,
                            gl::RGBA8 as gl::types::GLint,
                            texture.width as i32,
                            texture.height as i32,
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
                            texture.width as i32,
                            texture.height as i32,
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

        Ok(texture)
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
