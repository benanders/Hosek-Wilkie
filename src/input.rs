
//
//  Input
//

use glutin;
use glutin::{Event, Window, ElementState};


/// The number of virtual key codes we need to keep track of.
const KEYS_COUNT: usize = 134;

/// The number of mouse buttons we need to keep track of.
const MOUSE_BUTTONS_COUNT: usize = 3;


/// Rename the virtual key code to `Key`.
pub type Key = glutin::VirtualKeyCode;


/// Persists input data across frames.
pub struct Input {
	/// An array indexed by virtual key codes, set to true if a key is held
	/// down.
	keys_down: [bool; KEYS_COUNT],

	/// An array indexed by mouse buttons, set to true if a mouse button is held
	/// down.
	mouse_buttons_down: [bool; MOUSE_BUTTONS_COUNT],

	/// An array indexed by mouse buttons, set to true if a mouse button was
	/// just pressed. These values are only true for a single frame.
	mouse_buttons_pressed: [bool; MOUSE_BUTTONS_COUNT],

	/// The most recent movement of the mouse along the x axis.
	mouse_delta_x: f32,

	/// The most recent movement of the mouse along the y axis.
	mouse_delta_y: f32,

	/// True if the main window is open.
	window_open: bool,

	/// The width of the window.
	width: u32,

	/// The height of the window.
	height: u32,

	/// The scale factor of the window.
	scale_factor: f32,
}

impl Input {
	/// Creates a new input handler, using the window's dimensions.
	pub fn new(window: &Window) -> Input {
		let (point_width, height) = window.get_inner_size_points().unwrap();
		let (pixel_width, _) = window.get_inner_size_pixels().unwrap();
		let scale = pixel_width as f32 / point_width as f32;
		Input {
			keys_down: [false; KEYS_COUNT],
			mouse_buttons_down: [false; MOUSE_BUTTONS_COUNT],
			mouse_buttons_pressed: [false; MOUSE_BUTTONS_COUNT],
			mouse_delta_x: 0.0,
			mouse_delta_y: 0.0,
			window_open: true,
			width: point_width,
			height: height,
			scale_factor: scale,
		}
	}

	/// Returns the most recent mouse delta.
	pub fn mouse_delta(&self) -> (f32, f32) {
		(self.mouse_delta_x, self.mouse_delta_y)
	}

	/// Returns true if a key is held down.
	pub fn is_key_down(&self, key: Key) -> bool {
		let index = key as usize;
		if index < KEYS_COUNT {
			self.keys_down[index]
		} else {
			// We're not keeping track of the requested key
			false
		}
	}

	/// Returns true if a mouse button is held down.
	pub fn is_mouse_down(&self, button: MouseButton) -> bool {
		let index = button as usize;
		if index < MOUSE_BUTTONS_COUNT {
			self.mouse_buttons_down[index]
		} else {
			// Not keeping track of this button
			false
		}
	}

	/// Returns true if a mouse button was just pressed.
	pub fn was_mouse_pressed(&self, button: MouseButton) -> bool {
		let index = button as usize;
		if index < MOUSE_BUTTONS_COUNT {
			self.mouse_buttons_pressed[index]
		} else {
			// Not keeping track of this button
			false
		}
	}

	/// Returns true as long as the main window is open.
	pub fn window_is_open(&self) -> bool {
		self.window_open
	}

	/// Called when the mouse moves.
	fn mouse_move(&mut self, x: i32, y: i32, window: &Window) {
		// Convert the mouse coordinates to points instead of pixels
		let real_x = x as f32 / self.scale_factor;
		let real_y = y as f32 / self.scale_factor;

		// Calculate the new deltas
		let center_x = self.width / 2;
		let center_y = self.height / 2;
		self.mouse_delta_x = center_x as f32 - real_x;
		self.mouse_delta_y = center_y as f32 - real_y;

		// Reset the mouse location in the window
		window.set_cursor_position(center_x as i32, center_y as i32).unwrap();
	}

	/// Called when a key is pressed or released.
	fn key(&mut self, key: Key, is_down: bool) {
		let index = key as usize;

		// Only if we're tracking the key
		if index < KEYS_COUNT {
			self.keys_down[index] = is_down;
		}
	}

	/// Called when the mouse is pressed or released.
	fn mouse(&mut self, glutin_button: glutin::MouseButton, is_down: bool) {
		let potential = MouseButton::from_glutin(glutin_button);

		// Only if we can translate the glutin button to one we care about
		if let Some(button) = potential {
			let index = button as usize;

			// Only if we're tracking this mouse button
			if index < MOUSE_BUTTONS_COUNT {
				self.mouse_buttons_down[index] = is_down;
				self.mouse_buttons_pressed[index] = is_down;
			}
		}
	}

	/// Called when an event occurs to update state.
	pub fn handle_event(&mut self, event: Event, window: &Window) {
		match event {
			Event::Closed => self.window_open = false,
			Event::MouseMoved(x, y) => self.mouse_move(x, y, window),
			Event::KeyboardInput(action, _, Some(key)) =>
				self.key(key, action == ElementState::Pressed),
			Event::MouseInput(action, button) =>
				self.mouse(button, action == ElementState::Pressed),
			_ => {},
		}
	}

	/// Called once a frame to update state.
	pub fn update(&mut self) {
		// Reset the mouse deltas
		self.mouse_delta_x = 0.0;
		self.mouse_delta_y = 0.0;

		// Reset the button pressed states
		for i in 0 .. MOUSE_BUTTONS_COUNT {
			self.mouse_buttons_pressed[i] = false;
		}
	}
}


/// All mouse buttons that we care about.
pub enum MouseButton {
	Left,
	Right,
	Middle,
}

impl MouseButton {
	/// Convert from a glutin mouse button.
	fn from_glutin(button: glutin::MouseButton) -> Option<MouseButton> {
		match button {
			glutin::MouseButton::Left => Some(MouseButton::Left),
			glutin::MouseButton::Right => Some(MouseButton::Right),
			glutin::MouseButton::Middle => Some(MouseButton::Middle),
			_ => None,
		}
	}
}
