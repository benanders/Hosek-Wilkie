
//
//  Sky Prototype
//

extern crate gl;
extern crate glutin;
extern crate cgmath;

use input::Input;
use player::Player;
use camera::Camera;
use shader::{Shader, ShaderType, ShaderProgram};
use hosek::{DATASETS_RGB, DATASETS_RGB_RAD};

use gl::types::*;
use glutin::{WindowBuilder, CursorState, VirtualKeyCode};
use cgmath::{Matrix, Vector2, Vector3, ElementWise, InnerSpace, Quaternion, Rotation3, Rad, Rotation};
use std::{mem, ptr, f32};

mod player;
mod camera;
mod input;
mod shader;
mod hosek;

const TURBIDITY: f32 = 4.0;
const ALBEDO: [f32; 3] = [0.1, 0.1, 0.1];
const NORMALIZED_SUN_Y: f32 = 1.0;

static VERT_SOURCE: &'static str = include_str!("shaders/vert.glsl");
static FRAG_SOURCE: &'static str = include_str!("shaders/frag.glsl");

static VERTEX_DATA: [GLfloat; 24] = [
	-1.0, -1.0,  1.0, // 0: Left,  bottom, front
	 1.0, -1.0,  1.0, // 1: Right, bottom, front
	 1.0,  1.0,  1.0, // 2: Right, top,    front
	-1.0,  1.0,  1.0, // 3: Left,  top,    front
	-1.0, -1.0, -1.0, // 4: Left,  bottom, back
	 1.0, -1.0, -1.0, // 5: Right, bottom, back
	 1.0,  1.0, -1.0, // 6: Right, top,    back
	-1.0,  1.0, -1.0, // 7: Left,  top,    back
];

static INDEX_DATA: [GLushort; 36] = [
	2, 1, 0, 0, 3, 2, // Front
	5, 6, 7, 7, 4, 5, // Back
	3, 0, 4, 4, 7, 3, // Left
	6, 5, 1, 1, 2, 6, // Top
	6, 2, 3, 3, 7, 6, // Right
	0, 1, 5, 5, 4, 0, // Bottom
];

fn main() {
	// Create a window
	let width = 900;
	let height = 620;
	let window = WindowBuilder::new()
		.with_dimensions(width, height)
		.with_title("Skybox Prototype")
		.with_vsync()
		.build().unwrap();

	// Hide the cursor to fake capturing it
	window.set_cursor_position(width as i32 / 2, height as i32 / 2).unwrap();
	window.set_cursor_state(CursorState::Hide).unwrap();

	// Create input system
	let mut input = Input::new(&window);

	// Load OpenGL
	unsafe {
		window.make_current().unwrap();
		gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
		gl::ClearColor(0.0, 0.0, 0.0, 1.0);
		gl::Enable(gl::DEPTH_TEST);
		gl::Enable(gl::CULL_FACE);
	}

	// Player
	let camera = Camera::new(width, height);
	let mut player = Player::new(camera);

	// Load shaders
	let vert = Shader::new(ShaderType::Vertex, VERT_SOURCE);
	let frag = Shader::new(ShaderType::Fragment, FRAG_SOURCE);
	let program = ShaderProgram::new();
	program.attach(vert);
	program.attach(frag);
	program.link();
	program.bind();

	// Buffers
	let mut vao = 0;
	let mut pos_buffer = 0;
	let mut index_buffer = 0;
	unsafe {
		// VAO
		gl::GenVertexArrays(1, &mut vao);
		gl::BindVertexArray(vao);

		// Position
		gl::GenBuffers(1, &mut pos_buffer);
		gl::BindBuffer(gl::ARRAY_BUFFER, pos_buffer);
		let vert_size = (VERTEX_DATA.len() * mem::size_of::<GLfloat>()) as GLsizeiptr;
		let vert_ptr = mem::transmute(&VERTEX_DATA[0]);
		gl::BufferData(gl::ARRAY_BUFFER, vert_size, vert_ptr, gl::STATIC_DRAW);

		// Indices
		gl::GenBuffers(1, &mut index_buffer);
		gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, index_buffer);
		let idx_size = (INDEX_DATA.len() * mem::size_of::<GLushort>()) as GLsizeiptr;
		let idx_ptr = mem::transmute(&INDEX_DATA[0]);
		gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, idx_size, idx_ptr, gl::STATIC_DRAW);
	}

	// Shader attributes
	unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, pos_buffer) };
	let pos_loc = program.attr("position");
	shader::set_attr(pos_loc, 3, gl::FLOAT, 0, 0); // Position

	// Shader uniforms
	let projection_uniform = program.uniform("projection");
	let orientation_uniform = program.uniform("orientation");

	// Sky data uniforms
	let params_uniform = program.uniform("params");
	let sun_uniform = program.uniform("sun_direction");

	// Compute sky values
	let mut sun = Vector2::new(0.0, 0.0);
	let (sun_dir, params) = recalc_sun(sun);

	// Main event loop
	while input.window_is_open() {
		// Handle events
		for event in window.poll_events() {
			input.handle_event(event, &window);
		}

		// Update
		player.update(&input, 1.0);
		if input.is_key_down(VirtualKeyCode::Up) {
			sun.x -= 0.01;
		} else if input.is_key_down(VirtualKeyCode::Down) {
			sun.x += 0.01;
		}
		let (sun_dir, params) = recalc_sun(sun);

		input.update();

		unsafe {
			// Clear the screen to the clear colour
			gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

			// Matrix uniforms
			gl::UniformMatrix4fv(projection_uniform, 1, gl::FALSE, player.camera.projection.as_ptr());
			gl::UniformMatrix4fv(orientation_uniform, 1, gl::FALSE, player.camera.orientation.as_ptr());

			// Sky uniforms
			gl::Uniform3fv(params_uniform, 10, mem::transmute(&params[0]));
			gl::Uniform3f(sun_uniform, sun_dir.x, sun_dir.y, sun_dir.z);

			// Render
			gl::DrawElements(gl::TRIANGLES, INDEX_DATA.len() as GLint, gl::UNSIGNED_SHORT, ptr::null());
		}

		// Show the triangle on screen
		window.swap_buffers().unwrap();
	}
}


fn recalc_sun(sun_pos: Vector2<f32>) -> (Vector3<f32>, [Vector3<f32>; 10]) {
	let sun_dir = Quaternion::from_axis_angle(Vector3::new(0.0, 1.0, 0.0), Rad(sun_pos.y)).rotate_vector(Quaternion::from_axis_angle(Vector3::new(-1.0, 0.0, 0.0), Rad(sun_pos.x)).rotate_vector(Vector3::new(0.0, 0.0, 1.0)));
	// println!("{:?}", sun_dir);

	// let hor = (sun_dir.x * sun_dir.x + sun_dir.z * sun_dir.z).sqrt();
	let sun_theta = clamp(sun_dir.y, 0.0, 1.0).acos();
	// println!("sun theta {}, cos sun theta {}", sun_theta, sun_theta.cos());
	let mut params = [Vector3::new(0.0, 0.0, 0.0); 10];
	for i in 0 .. 3 {
		params[0][i] = evaluate(&DATASETS_RGB[i][0 ..], 9, TURBIDITY, ALBEDO[i], sun_theta);
		params[1][i] = evaluate(&DATASETS_RGB[i][1 ..], 9, TURBIDITY, ALBEDO[i], sun_theta);
		params[2][i] = evaluate(&DATASETS_RGB[i][2 ..], 9, TURBIDITY, ALBEDO[i], sun_theta);
		params[3][i] = evaluate(&DATASETS_RGB[i][3 ..], 9, TURBIDITY, ALBEDO[i], sun_theta);
		params[4][i] = evaluate(&DATASETS_RGB[i][4 ..], 9, TURBIDITY, ALBEDO[i], sun_theta);
		params[5][i] = evaluate(&DATASETS_RGB[i][5 ..], 9, TURBIDITY, ALBEDO[i], sun_theta);
		params[6][i] = evaluate(&DATASETS_RGB[i][6 ..], 9, TURBIDITY, ALBEDO[i], sun_theta);

		params[7][i] = evaluate(&DATASETS_RGB[i][8 ..], 9, TURBIDITY, ALBEDO[i], sun_theta);
		params[8][i] = evaluate(&DATASETS_RGB[i][7 ..], 9, TURBIDITY, ALBEDO[i], sun_theta);

		// Z value thing
		params[9][i] = evaluate(DATASETS_RGB_RAD[i], 1, TURBIDITY, ALBEDO[i], sun_theta);
	}

	let S = hosek_wilkie(sun_theta.cos(), 0.0, 1.0, &params[0 .. 9]).mul_element_wise(params[9]);
	// println!("S {:?}", S);
	params[9] /= S.dot(Vector3::new(0.2126, 0.7152, 0.0722));

	let mut sun_amount = (sun_dir.y / f32::consts::FRAC_PI_2) % 4.0;
	if sun_amount > 2.0 {
		sun_amount = 0.0;//-(sun_amount - 2.0);
	}
	if sun_amount > 1.0 {
		sun_amount = 2.0 - sun_amount;
	} else if sun_amount < -1.0 {
		sun_amount = -2.0 - sun_amount;
	}

	let normalized_sun_y = 0.6 + 0.45 * sun_amount;
	params[9] *= normalized_sun_y;

	// for i in 0 .. 10 {
	// 	println!("params {}: {:?}", i, params[i]);
	// }

	(sun_dir, params)
}


fn evaluate_spline(dataset: &[f32], start: usize, stride: usize, value: f32) -> f32 {
	1.0 *  (1.0 - value).powi(5) *                 dataset[start + 0 * stride] +
	5.0 *  (1.0 - value).powi(4) * value.powi(1) * dataset[start + 1 * stride] +
	10.0 * (1.0 - value).powi(3) * value.powi(2) * dataset[start + 2 * stride] +
	10.0 * (1.0 - value).powi(2) * value.powi(3) * dataset[start + 3 * stride] +
	5.0 *  (1.0 - value).powi(1) * value.powi(4) * dataset[start + 4 * stride] +
	1.0 *                          value.powi(5) * dataset[start + 5 * stride]
}

fn evaluate(dataset: &[f32], stride: usize, turbidity: f32, albedo: f32, sun_theta: f32) -> f32 {
	// splines are functions of elevation^1/3
	let elevationK = (1.0 - sun_theta / f32::consts::FRAC_PI_2).max(0.0).powf(1.0 / 3.0);

	// table has values for turbidity 1..10
	let turbidity0 = clamp(turbidity as usize, 1, 10);
	let turbidity1 = min(turbidity0 + 1, 10);
	let turbidityK = clamp(turbidity - turbidity0 as f32, 0.0, 1.0);

	let datasetA0 = 0;
	let datasetA1 = stride * 6 * 10;

	let a0t0 = evaluate_spline(dataset, datasetA0 + stride * 6 * (turbidity0 - 1), stride, elevationK);
	let a1t0 = evaluate_spline(dataset, datasetA1 + stride * 6 * (turbidity0 - 1), stride, elevationK);
	let a0t1 = evaluate_spline(dataset, datasetA0 + stride * 6 * (turbidity1 - 1), stride, elevationK);
	let a1t1 = evaluate_spline(dataset, datasetA1 + stride * 6 * (turbidity1 - 1), stride, elevationK);

	a0t0 * (1.0 - albedo) * (1.0 - turbidityK) + a1t0 * albedo * (1.0 - turbidityK) + a0t1 * (1.0 - albedo) * turbidityK + a1t1 * albedo * turbidityK
}

fn hosek_wilkie(cos_theta: f32, gamma: f32, cos_gamma: f32, params: &[Vector3<f32>]) -> Vector3<f32> {
	let A = params[0];
	let B = params[1];
	let C = params[2];
	let D = params[3];
	let E = params[4];
	let F = params[5];
	let G = params[6];
	let H = params[7];
	let I = params[8];

	// println!("INPUT");
	// println!("cos theta {}", cos_theta);
	// println!("gamma {}", gamma);
	// println!("cos gamma {}", cos_gamma);
	// println!("params {:?}",  params);
	// println!("END INPUT");

	// println!("cos gamma {}", cos_gamma);
    // float3 chi = (1.f + cos_gamma * cos_gamma) / pow(1.f + H * H - 2.f * cos_gamma * H, float3(1.5f));

	let chi = (1.0 + cos_gamma * cos_gamma) / powv(H.mul_element_wise(H).add_element_wise(1.0) - 2.0 * cos_gamma * H, Vector3::new(1.5, 1.5, 1.5));
	// println!("denom {:?}",  powv(H.mul_element_wise(H).add_element_wise(1.0) - 2.0 * cos_gamma * H, Vector3::new(1.5, 1.5, 1.5)));
	// println!("chi {:?}", chi);
	(A.mul_element_wise(exp(B / (cos_theta + 0.01))).add_element_wise(1.0)).mul_element_wise((C + D.mul_element_wise(exp(E * gamma)) + F * (cos_gamma * cos_gamma) + G.mul_element_wise(chi) + I * cos_theta.max(0.0).sqrt()))
}

fn powv(a: Vector3<f32>, b: Vector3<f32>) -> Vector3<f32> {
	Vector3::new(a.x.powf(b.x), a.y.powf(b.y), a.z.powf(b.z))
}

fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
	if value < min {
		min
	} else if value > max {
		max
	} else {
		value
	}
}

fn min<T: PartialOrd>(value: T, min: T) -> T {
	if value < min {
		value
	} else {
		min
	}
}

fn exp(vec: Vector3<f32>) -> Vector3<f32> {
	Vector3::new(vec.x.exp(), vec.y.exp(), vec.z.exp())
}
