
//
//  FPS Camera
//

use cgmath::{Rad, PerspectiveFov, Matrix4, Zero, EuclideanSpace, Vector3,
	Point3, InnerSpace};
use std::f32;


/// The player's field of view.
const FOV: f32 = 70.0 * f32::consts::PI / 180.0;

/// The distance between the eye point of the camera and the near plane.
const NEAR: f32 = 0.1;

/// The distance between the eye point of the camera and the far plane.
const FAR: f32 = 1000.0;


/// The speed at which the player can look around.
const LOOK_SPEED: f32 = 0.0015;

/// The speed at which the player can walk.
const MOVE_SPEED: f32 = 0.1;

/// The minimum vertical look angle.
const MIN_ANGLE: f32 = -f32::consts::FRAC_PI_2 + f32::EPSILON;

/// The maximum vertical look angle.
const MAX_ANGLE: f32 = f32::consts::FRAC_PI_2 - f32::EPSILON;


/// A 3D first person camera which keeps track of the player's position and
/// where they're looking.
pub struct Camera {
	/// The player's rotation around the vertical axis.
	horizontal: f32,
	/// The player's rotation around the horizontal axis.
	vertical: f32,
	/// The player's position.
	pub position: Vector3<f32>,

	/// A vector pointing in the forwards direction for the player.
	forward: Vector3<f32>,
	/// A vector pointing to the right relative to where the player's looking.
	right: Vector3<f32>,
	/// A vector pointing straight up.
	up: Vector3<f32>,

	/// The field of view for the projection matrix.
	fov: f32,
	/// The near plane for the projection matrix.
	near: f32,
	/// The far plane for the projection matrix.
	far: f32,
	/// The aspect ratio of the window.
	aspect: f32,

	/// The perspective projection matrix.
	pub projection: Matrix4<f32>,
	/// The orientation matrix (projection and rotation, excluding translation).
	pub orientation: Matrix4<f32>,
	/// The view matrix (projection, rotation, and translation).
	pub view: Matrix4<f32>,
}

impl Camera {
	/// Creates a new camera, in a window with the given dimensions.
	pub fn new(width: u32, height: u32) -> Camera {
		let mut camera = Camera {
			horizontal: f32::consts::FRAC_PI_2,
			vertical: 0.0,
			position: Vector3::new(0.0, 0.0, 0.0),

			forward: Vector3::zero(),
			right: Vector3::zero(),
			up: Vector3::zero(),

			fov: FOV,
			near: NEAR,
			far: FAR,
			aspect: width as f32 / height as f32,

			projection: Matrix4::zero(),
			orientation: Matrix4::zero(),
			view: Matrix4::zero(),
		};

		camera.update_projection();
		camera.update_axes();
		camera.update_orientation();
		camera.update_view();
		camera
	}

	/// Update the camera's projection matrix.
	pub fn update_projection(&mut self) {
		self.projection = Matrix4::from(PerspectiveFov {
			fovy: Rad(self.fov),
			aspect: self.aspect,
			near: self.near,
			far: self.far,
		});
	}

	/// Update the camera's axes relative to the look direction.
	pub fn update_axes(&mut self) {
		// Convert spherical coordinates to cartesian using horizontal and
		// vertical angles, with sphere radius of 1
		self.forward = Vector3::new(
			self.vertical.cos() * self.horizontal.sin(),
			self.vertical.sin(),
			self.vertical.cos() * self.horizontal.cos()
		);

		// Right vector is always horizontal in the xz plane (as camera doesn't
		// rotate around x axis)
		self.right = Vector3::new(
			(self.horizontal - f32::consts::FRAC_PI_2).sin(),
			0.0,
			(self.horizontal - f32::consts::FRAC_PI_2).cos()
		);

		// Up vector is cross product of forward and right vectors
		self.up = self.right.cross(self.forward);
	}

	/// Updates the camera's orientation matrix.
	fn update_orientation(&mut self) {
		self.orientation = Matrix4::look_at(
			Point3::origin(),
			Point3::origin() + self.forward,
			self.up
		);
	}

	/// Updates the camera's view matrix.
	fn update_view(&mut self) {
		self.view = Matrix4::look_at(
			Point3::from_vec(self.position),
			Point3::from_vec(self.position + self.forward),
			self.up
		);
	}

	/// Rotates the camera by a certain amount along each axis.
	pub fn look(&mut self, horizontal: f32, vertical: f32, delta: f32) {
		// Vertical rotation
		self.vertical = (self.vertical + vertical * delta * LOOK_SPEED)
			.max(MIN_ANGLE).min(MAX_ANGLE);

		// Horizontal rotation
		self.horizontal = self.horizontal + horizontal * delta * LOOK_SPEED;
		if self.horizontal < 0.0 {
			self.horizontal += f32::consts::PI * 2.0;
		} else if self.horizontal > f32::consts::PI * 2.0 {
			self.horizontal -= f32::consts::PI * 2.0;
		}

		// Update matrices
		self.update_axes();
		self.update_orientation();
		self.update_view();
	}

	/// Moves the camera around by a certain amount along each axis.
	pub fn walk(&mut self, x: f32, y: f32, z: f32, delta: f32) {
		let scale = delta * MOVE_SPEED;

		// X axis
		if x.abs() > f32::EPSILON {
			self.position += Vector3::new(self.right.x, 0.0, self.right.z)
				.normalize() * x * scale;
		}

		// Y axis
		if y.abs() > f32::EPSILON {
			self.position.y += y * scale;
		}

		// Z axis
		if z.abs() > f32::EPSILON {
			self.position += Vector3::new(self.forward.x, 0.0, self.forward.z)
				.normalize() * z * scale;
		}

		// Update matrices
		self.update_view();
	}
}
