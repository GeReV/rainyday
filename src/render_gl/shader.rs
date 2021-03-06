﻿use nalgebra as na;
use std::ffi::{CStr, CString};
use std::iter::Iterator;

#[derive(Debug, Fail)] // derive Fail, in addition to Debug
pub enum Error {
    #[fail(display = "Failed to compile shader {}: {}", name, message)]
    CompileError { name: String, message: String },
    #[fail(display = "Failed to link program {}: {}", name, message)]
    LinkError { name: String, message: String },
}

pub struct Program {
    id: gl::types::GLuint,
    gl: gl::Gl,
}

impl Program {
    pub fn from_shaders(gl: &gl::Gl, shaders: &[Shader]) -> Result<Program, String> {
        let program_id = unsafe { gl.CreateProgram() };

        for shader in shaders {
            unsafe {
                gl.AttachShader(program_id, shader.id());
            }
        }

        unsafe {
            gl.LinkProgram(program_id);
        }

        let mut success: gl::types::GLint = 1;
        unsafe {
            gl.GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
        }

        if success == 0 {
            let mut len: gl::types::GLint = 0;
            unsafe {
                gl.GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
            }

            let error = create_whitespace_cstring_with_len(len as usize);

            unsafe {
                gl.GetProgramInfoLog(
                    program_id,
                    len,
                    std::ptr::null_mut(),
                    error.as_ptr() as *mut gl::types::GLchar,
                );
            }

            return Err(error.to_string_lossy().into_owned());
        }

        for shader in shaders {
            unsafe {
                gl.DetachShader(program_id, shader.id());
            }
        }

        Ok(Program {
            gl: gl.clone(),
            id: program_id,
        })
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }

    pub fn set_used(&self) {
        unsafe {
            self.gl.UseProgram(self.id);
        }
    }

    pub fn get_uniform_location(&self, name: &str) -> Option<i32> {
        let cname = CString::new(name).expect("expected uniform name to have no nul bytes");

        let location = unsafe {
            self.gl
                .GetUniformLocation(self.id, cname.as_bytes_with_nul().as_ptr() as *const i8)
        };

        if location == -1 {
            return None;
        }

        Some(location)
    }

    pub fn set_uniform_matrix_4fv(&self, location: i32, value: &na::Matrix4<f32>) {
        unsafe {
            self.gl.UniformMatrix4fv(
                location,
                1,
                gl::FALSE,
                value.as_slice().as_ptr() as *const f32,
            );
        }
    }

    pub fn set_uniform_4fv(&self, location: i32, value: &[[f32; 4]]) {
        unsafe {
            self.gl
                .Uniform4fv(location, value.len() as i32, value.as_ptr() as *const f32);
        }
    }

    pub fn set_uniform_4f(&self, location: i32, value: &na::Vector4<f32>) {
        unsafe {
            self.gl
                .Uniform4f(location, value.x, value.y, value.z, value.w);
        }
    }

    pub fn set_uniform_3f(&self, location: i32, value: &na::Vector3<f32>) {
        unsafe {
            self.gl.Uniform3f(location, value.x, value.y, value.z);
        }
    }

    pub fn set_uniform_2f(&self, location: i32, value: &na::Vector2<f32>) {
        unsafe {
            self.gl.Uniform2f(location, value.x, value.y);
        }
    }

    pub fn set_uniform_1f(&self, location: i32, value: f32) {
        unsafe {
            self.gl.Uniform1f(location, value);
        }
    }

    pub fn set_uniform_1i(&self, location: i32, index: i32) {
        unsafe {
            self.gl.Uniform1i(location, index);
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteProgram(self.id);
        }
    }
}

pub struct Shader {
    id: gl::types::GLuint,
    gl: gl::Gl,
}

impl Shader {
    pub fn from_source(
        gl: &gl::Gl,
        source: &CStr,
        kind: gl::types::GLenum,
    ) -> Result<Shader, String> {
        let id = shader_from_source(gl, source, kind)?;

        Ok(Shader { gl: gl.clone(), id })
    }

    pub fn from_vert_source_str(gl: &gl::Gl, source: &str) -> Result<Shader, Error> {
        Shader::from_vert_source(gl, &str_to_cstr(source)).map_err(|msg| Error::CompileError {
            message: msg,
            name: "".into(),
        })
    }

    pub fn from_frag_source_str(gl: &gl::Gl, source: &str) -> Result<Shader, Error> {
        Shader::from_frag_source(gl, &str_to_cstr(source)).map_err(|msg| Error::CompileError {
            message: msg,
            name: "".into(),
        })
    }

    pub fn from_vert_source(gl: &gl::Gl, source: &CStr) -> Result<Shader, String> {
        Shader::from_source(gl, source, gl::VERTEX_SHADER)
    }

    pub fn from_frag_source(gl: &gl::Gl, source: &CStr) -> Result<Shader, String> {
        Shader::from_source(gl, source, gl::FRAGMENT_SHADER)
    }

    pub fn id(&self) -> gl::types::GLuint {
        self.id
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.gl.DeleteShader(self.id);
        }
    }
}

fn shader_from_source(
    gl: &gl::Gl,
    source: &CStr,
    kind: gl::types::GLenum,
) -> Result<gl::types::GLuint, String> {
    let id = unsafe { gl.CreateShader(kind) };

    unsafe {
        gl.ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
        gl.CompileShader(id);
    }

    let mut success: gl::types::GLint = 1;

    unsafe {
        gl.GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
    }

    if success == 0 {
        let mut len: gl::types::GLint = 0;

        unsafe {
            gl.GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
        }

        let error = create_whitespace_cstring_with_len(len as usize);

        unsafe {
            gl.GetShaderInfoLog(
                id,
                len,
                std::ptr::null_mut(),
                error.as_ptr() as *mut gl::types::GLchar,
            );
        }

        return Err(error.to_string_lossy().into_owned());
    }

    Ok(id)
}

fn create_whitespace_cstring_with_len(len: usize) -> CString {
    // allocate buffer of correct size
    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);
    // fill it with len spaces
    buffer.extend([b' '].iter().cycle().take(len));
    // convert buffer to CString
    unsafe { CString::from_vec_unchecked(buffer) }
}

fn str_to_cstr(str: &str) -> std::ffi::CString {
    unsafe { std::ffi::CString::from_vec_unchecked(str.as_bytes().to_vec()) }
}
