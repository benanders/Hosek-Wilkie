
//
//  Shader Loading
//

use gl;
use gl::types::*;

use std::ptr;
use std::ffi::CString;

/// The possible shader types.
pub enum ShaderType {
	Vertex = gl::VERTEX_SHADER as isize,
	Fragment = gl::FRAGMENT_SHADER as isize,
}

/// A single OpenGL shader.
pub struct Shader(GLuint);

impl Shader {
	/// Create a new shader, compiling the given source code.
	///
	/// Panics if shader compilation fails.
	// TODO: Not panic when compilation fails, but create a custom error object
	// and return that instead
	pub fn new(kind: ShaderType, source: &str) -> Shader {
		let id = unsafe { gl::CreateShader(kind as GLenum) };

		// Attach the source to the shader
		let c_str = CString::new(source.as_bytes()).unwrap();
		unsafe { gl::ShaderSource(id, 1, &c_str.as_ptr(), ptr::null()); }

		// Compile the shader
		let shader = Shader(id);
		shader.compile();
		shader
	}

	/// Compiles the shader.
	fn compile(&self) {
		// Compile the shader
		unsafe { gl::CompileShader(self.0); }

		// Check for a compilation error
		match self.error_message() {
			Some(message) => panic!("Failed to compile shader :\n{}", message),
			None => {},
		}
	}

	/// Returns true if an error occurred during compilation.
	fn has_error(&self) -> bool {
		// Get the compilation status
		let mut status = gl::FALSE as GLint;
		unsafe {
			gl::GetShaderiv(self.0, gl::COMPILE_STATUS, &mut status);
		}

		// Check the status
		status != gl::TRUE as GLint
	}

	/// Returns the shader's compilation error message, if compilation failed.
	fn error_message(&self) -> Option<String> {
		if self.has_error() {
			// Get the error message length
			let mut length = 0;
			unsafe {
				gl::GetShaderiv(self.0, gl::INFO_LOG_LENGTH, &mut length);
			}

			// Get the contents of the message
			// Set the length of the buffer to skip the NULL terminator at the
			// end of the string
			let mut buffer = Vec::with_capacity(length as usize);
			unsafe {
				buffer.set_len(length as usize - 1);
				let ptr = buffer.as_mut_ptr() as *mut GLchar;
				gl::GetShaderInfoLog(self.0, length, ptr::null_mut(), ptr);
			}

			// Convert message to a string
			Some(String::from_utf8(buffer)
				.expect("Shader compilation error not UTF-8"))
		} else {
			// No error
			None
		}
	}
}

impl Drop for Shader {
	fn drop(&mut self) {
		unsafe { gl::DeleteShader(self.0) };
	}
}


/// A shader program, linking together a number of shaders.
pub struct ShaderProgram(GLuint);

impl ShaderProgram {
	/// Creates a new shader program.
	pub fn new() -> ShaderProgram {
		let id = unsafe { gl::CreateProgram() };
		ShaderProgram(id)
	}

	/// Attaches a shader to the program.
	pub fn attach(&self, shader: Shader) {
		unsafe { gl::AttachShader(self.0, shader.0); }
	}

	/// Links the shader program, panicing if an error occurs.
	// TODO: Don't panic
	pub fn link(&self) {
		// Link the program
		unsafe { gl::LinkProgram(self.0); }

		// Check for error
		match self.error_message() {
			Some(message) => panic!("Failed to link shader: {}", message),
			None => {},
		}
	}

	/// Returns true if a link error occurred.
	fn has_error(&self) -> bool {
		// Get link status
		let mut status = gl::FALSE as GLint;
		unsafe {
			gl::GetProgramiv(self.0, gl::LINK_STATUS, &mut status);
		}

		status != gl::TRUE as GLint
	}

	/// Returns the link error message, if one exists.
	fn error_message(&self) -> Option<String> {
		if self.has_error() {
			// Get the length of the message
			let mut length = 0;
			unsafe {
				gl::GetProgramiv(self.0, gl::INFO_LOG_LENGTH, &mut length);
			}

			// Get the message
			let mut buffer = Vec::with_capacity(length as usize);
			unsafe {
				buffer.set_len(length as usize - 1);
				let ptr = buffer.as_mut_ptr() as *mut GLchar;
				gl::GetProgramInfoLog(self.0, length, ptr::null_mut(), ptr);
			}

			// Convert to string
			Some(String::from_utf8(buffer)
				.expect("Shader compilation error not UTF-8"))
		} else {
			// No error
			None
		}
	}

	/// Binds the shader program.
	pub fn bind(&self) {
		unsafe { gl::UseProgram(self.0); }
	}

	/// Returns the location of an attribute.
	pub fn attr(&self, name: &str) -> GLuint {
		let c_str = CString::new(name).unwrap();
		unsafe { gl::GetAttribLocation(self.0, c_str.as_ptr()) as GLuint }
	}

	/// Returns the location of a uniform.
	pub fn uniform(&self, name: &str) -> GLint {
		let c_str = CString::new(name).unwrap();
		unsafe { gl::GetUniformLocation(self.0, c_str.as_ptr()) }
	}
}

impl Drop for ShaderProgram {
	fn drop(&mut self) {
		unsafe { gl::DeleteProgram(self.0) };
	}
}

/// Set a vertex attribute.
/// Assumes the appropriate VAO, VBO, and shader are bound.
pub fn set_attr(location: GLuint, values: i32, kind: GLenum, start: i32,
		size: i32) {
	unsafe {
		gl::EnableVertexAttribArray(location);
		gl::VertexAttribPointer(
			location,
			values,
			kind,
			gl::FALSE,
			size,
			start as *const GLvoid,
		);
	}
}
