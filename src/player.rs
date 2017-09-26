
//
//  Player
//

use camera::Camera;
use input::{Input, Key};

use glutin::VirtualKeyCode;


/// A mapping between a key code and axis of movement.
pub struct KeyMap {
	key: Key,
	x: i32,
	y: i32,
	z: i32,
}

/// A map of keys to which axis of movement they control.
const KEY_MAPPINGS: [KeyMap; 7] = [
	KeyMap { key: VirtualKeyCode::W,      x:  0, y:  0, z:  1 }, // Forward
	KeyMap { key: VirtualKeyCode::S,      x:  0, y:  0, z: -1 }, // Back
	KeyMap { key: VirtualKeyCode::A,      x: -1, y:  0, z:  0 }, // Left
	KeyMap { key: VirtualKeyCode::D,      x:  1, y:  0, z:  0 }, // Right
	KeyMap { key: VirtualKeyCode::Space,  x:  0, y:  1, z:  0 }, // Up
	KeyMap { key: VirtualKeyCode::LShift, x:  0, y: -1, z:  0 }, // Down
	KeyMap { key: VirtualKeyCode::RShift, x:  0, y: -1, z:  0 }, // Down
];


/// The player, controlling the camera by handling user input.
pub struct Player {
	/// The underlying camera the player controls.
	pub camera: Camera,
}

impl Player {
	/// Create a new player object.
	pub fn new(camera: Camera) -> Player {
		Player {
			camera: camera,
		}
	}

	/// Called every frame to update the player's motion.
	pub fn update(&mut self, input: &Input, delta: f32) {
		// Movement
		let (x, y, z) = self.motion_vector(input);
		if x != 0 || y != 0 || z != 0 {
			self.camera.walk(x as f32, y as f32, z as f32, delta);
		}

		// Look
		let (dx, dy) = input.mouse_delta();
		if dx != 0.0 || dy != 0.0 {
			self.camera.look(dx, dy, delta);
		}
	}

	/// Calculates the player's movement direction from which keys are held
	/// down.
	fn motion_vector(&self, input: &Input) -> (i32, i32, i32) {
		let mut x = 0;
		let mut y = 0;
		let mut z = 0;

		// Check each key
		for i in 0 .. KEY_MAPPINGS.len() {
			let key_map = &KEY_MAPPINGS[i];
			if input.is_key_down(key_map.key) {
				// Clamp each to the range (-1, 1)
				x = clamp(x + key_map.x, -1, 1);
				y = clamp(y + key_map.y, -1, 1);
				z = clamp(z + key_map.z, -1, 1);
			}
		}

		(x, y, z)
	}
}

/// Restrict a value to a range.
fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
	if value < min {
		min
	} else if value > max {
		max
	} else {
		value
	}
}
