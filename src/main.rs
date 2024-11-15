
use std::f32::consts::PI;
use macroquad::prelude::*;
use macroquad::rand::{rand, gen_range};
use rayon::prelude::*;

// types

#[derive(Clone)]
struct Voxel {
	color: Vec4,
	transparent: bool
}

#[derive(Clone)]
struct Neighbors {
	up_x  : usize,
	down_x: usize,
	up_y  : usize,
	down_y: usize,
	up_z  : usize,
	down_z: usize,
}

#[derive(Clone)]
struct VoxelPair {
	voxel: Voxel,
	neighbors: Neighbors
}

struct Position {
	voxel_id: usize,
	local_position: Vec3
}

type World = Vec<VoxelPair>;

struct Camera {
	i: usize,
	position: Vec3,
	angle: Vec2,
	movement_speed: f32,
	rotation_speed: (f32, f32),
	fov: (f32, f32),
	screen: (usize, usize),
}

// Settings
const RESOLUTION: (f32, f32) = (800 as f32, 600 as f32);
const SCREEN: (usize, usize) = (200, 150);
const WORLDSIZE: [usize; 3] = [64, 64, 64];
const MOVEMENT_SPEED: f32 = 0.1;
const ROTATION_SPEED: (f32, f32) = (0.75, 0.75);
const FOV: (f32, f32) = (PI/2.0, PI/2.0*(SCREEN.1 as f32)/(SCREEN.0 as f32));
const VIEW_DISTANCE: usize = 128;
const TOUCH_DISTANCE: usize = 16;
const AMBIENT: Vec4 = vec4(0.0, 0.0, 0.0, 1.0);
const RECTSIZE_X: f32 = RESOLUTION.0 / (SCREEN.0 as f32);
const RECTSIZE_Y: f32 = RESOLUTION.1 / (SCREEN.1 as f32);


// Raycasting algorithm
fn lattice_intersect(pos: Vec3, v: Vec3) -> (Vec3, [i32; 3], f32) {
	let t = ((v.signum() + 1.0) / 2.0 - pos) / v;
	let t_min: f32 = t.min_element();
	let mut i_min: usize = 0;
	for i in 0..3 {
		if t[i] == t_min {
			i_min = i
		}
	}
	let mut key: [i32; 3] = [0, 0, 0];
	key[i_min] = v[i_min].signum() as i32;
	let key2 = vec3(key[0] as f32, key[1] as f32, key[2] as f32);
	let x_new = pos + t_min*v - key2;
	return (x_new, key, (t_min*v).length());
}

fn raycast(world: &World, vox_id: usize, basepoint: Vec3, ray: Vec3, max_steps: usize) -> (usize, Vec3, f32) {
	let (mut i, mut x) = (vox_id, basepoint);
	let mut k: [i32; 3];
	let mut dist = 0.0;
	let mut dt = 0.0;
	for step in 0..max_steps {
		(x, k, dt)  = lattice_intersect(x, ray);
		dist = dist + dt;
		if k[0] == 1 {
			i = world[i].neighbors.up_x;
		} else if k[0] == -1 {
			i = world[i].neighbors.down_x;
		} else if k[1] == 1 {
			i = world[i].neighbors.up_y;
		} else if k[1] == -1 {
			i = world[i].neighbors.down_y;
		} else if k[2] == 1 {
			i = world[i].neighbors.up_z;
		} else if k[2] == -1 {
			i = world[i].neighbors.down_z;
		}
		if ! world[i].voxel.transparent {
			return (i, x, dist);
		}
	}
	return (i, x, max_steps as f32);
}

// World generation

fn furl(i: usize, j: usize, k: usize, ny: usize, nz: usize) -> usize {
	return i*ny*nz + j*nz + k
}
fn randf() -> f32 {
	(rand() as f32) / (u32::MAX as f32)
}
fn randr(a: f32, b: f32) -> f32 {
	a + (b -a) * randf()
}

fn build_world(nx: usize, ny: usize, nz: usize) -> World {

	// initialize world of given size with trivial linking
	let v = Voxel {
		color: vec4(0.0, 0.0, 0.0, 1.0),
		transparent: true
	};
	let voxelpair = VoxelPair {
		voxel: v,
		neighbors: Neighbors {
			up_x  : 0,
			down_x: 0,
			up_y  : 0,
			down_y: 0,
			up_z  : 0,
			down_z: 0
		}
	};

	// link all the voxels to their neighbors (this defines the topology)
	let mut world: World = vec![voxelpair; nx*ny*nz];
	for i in 0..nx {
		for j in 0..ny {
			for k in 0..nz {
				let n = furl(i, j, k, ny, nz);
				world[n].neighbors = Neighbors {
					up_x  : furl((i as i32 + 1).rem_euclid(nx as i32) as usize, j, k, ny, nz),
					down_x: furl((i as i32 - 1).rem_euclid(nx as i32) as usize, j, k, ny, nz),
					up_y  : furl(i, (j as i32 + 1).rem_euclid(ny as i32) as usize, k, ny, nz),
					down_y: furl(i, (j as i32 - 1).rem_euclid(ny as i32) as usize, k, ny, nz),
					up_z  : furl(i, j, (k as i32 + 1).rem_euclid(nz as i32) as usize, ny, nz),
					down_z: furl(i, j, (k as i32 - 1).rem_euclid(nz as i32) as usize, ny, nz)
				};
			}
		}
	}

	// randomize terrain
	for x in 0..nx {
		for y in 0..(ny/2) {
            for z in 0..nz {
				let n = furl(x, y, z, ny, nz);
				world[n].voxel.color = vec4(
					randr(0.5, 0.55),
					randr(0.5, 0.55),
					randr(0.5, 0.55),
					1.0
				);
				world[n].voxel.transparent = false
			}
		}
	}

	return world;
}

#[macroquad::main("voxeltorus")]
async fn main() {
	request_new_screen_size(RESOLUTION.0, RESOLUTION.1);
	next_frame().await;
	
	// Build world
	let mut world = build_world(WORLDSIZE[0], WORLDSIZE[1], WORLDSIZE[2]);
	
	// Place camera
	let mut camera = Camera {
		i: 0,
		position: vec3(0.5, 0.5, 0.5),
		angle: vec2(0.0, 0.0),
		movement_speed: MOVEMENT_SPEED,
		rotation_speed: ROTATION_SPEED,
		fov: FOV,
		screen: SCREEN,
	};
	let mut screen: Vec<Vec<(Vec4, f32)>> = vec![vec![(vec4(0.0, 0.0, 0.0, 0.0), 0.0); camera.screen.1]; camera.screen.0];
	let mut grabbed = true;
	
	// Unstuck camera
	while ! world[camera.i].voxel.transparent {
		camera.i = world[camera.i].neighbors.up_y;
	}
	let mut selected = Voxel {
		color: vec4(0.5, 0.4, 0.3, 1.0),
		transparent: false
	};
	
	//upward velocity (for gravity)
	let mut upward_velocity = 0.0;

	loop {
		// Take player input
		if is_mouse_button_released(MouseButton::Left) {
			grabbed = true;
		}
		if is_key_down(KeyCode::Escape) {
			grabbed = false;
		}
		set_cursor_grab(grabbed);
		show_mouse(!grabbed);

		// Rotate player camera
		let mut mouse_delta = vec2(0.0, 0.0);
		if grabbed {
			mouse_delta = mouse_delta_position();
		}
		camera.angle = camera.angle - vec2(camera.rotation_speed.0 * mouse_delta.x, -camera.rotation_speed.1 * mouse_delta.y);
		camera.angle[1] = clamp(camera.angle[1], -PI/2.0, PI/2.0);

		// Move player
		let look  = vec3( camera.angle[0].cos()*camera.angle[1].cos(), camera.angle[1].sin(),  camera.angle[0].sin()*camera.angle[1].cos());
		let up	= vec3(-camera.angle[0].cos()*camera.angle[1].sin(), camera.angle[1].cos(), -camera.angle[0].sin()*camera.angle[1].sin());
		let right = vec3(-camera.angle[0].sin(),					   0.0,					camera.angle[0].cos());
		let mut dx = vec3(0.0, 0.0, 0.0);


		let on_ground = !(world[world[camera.i].neighbors.down_y].voxel.transparent) & (camera.position.y <= 0.5);
		
		if is_key_down(KeyCode::LeftShift) {
			dx = dx - vec3(0.0, 1.0, 0.0)
		}
		if is_key_down(KeyCode::W) {
			dx = dx + look;
		}
		if is_key_down(KeyCode::S) {
			dx = dx - look;
		}
		if is_key_down(KeyCode::A) {
			dx = dx - right;
		}
		if is_key_down(KeyCode::D) {
			dx = dx + right;
		}

		match dx.try_normalize() {
			Some(dx) => {
				camera.position = camera.position + camera.movement_speed * dx;
			},
			None => {},
		}

		if on_ground {
			upward_velocity = 0.0;
		} else {
			upward_velocity = upward_velocity - 0.01;
		}

		if is_key_down(KeyCode::Space) & on_ground {
			// dx = dx + vec3(0.0, 1.0, 0.0)
			upward_velocity = 0.2;
		}

		camera.position = camera.position + upward_velocity * vec3(0.0, 1.0, 0.0);

		let mut camera_delta = vec3(0.0, 0.0, 0.0);
		if camera.position[0] < 0.0 {
			camera.i = world[camera.i].neighbors.down_x;
			camera_delta[0] = 1.0;
		} else if camera.position[0] > 1.0 {
			camera.i = world[camera.i].neighbors.up_x;
			camera_delta[0] = -1.0;
		}
		if camera.position[1] < 0.0 {
			camera.i = world[camera.i].neighbors.down_y;
			camera_delta[1] = 1.0;
		} else if camera.position[1] > 1.0 {
			camera.i = world[camera.i].neighbors.up_y;
			camera_delta[1] = -1.0;
		}
		if camera.position[2] < 0.0 {
			camera.i = world[camera.i].neighbors.down_z;
			camera_delta[2] = 1.0;
		} else if camera.position[2] > 1.0 {
			camera.i = world[camera.i].neighbors.up_z;
			camera_delta[2] = -1.0;
		}
		camera.position = camera.position + camera_delta;

		if on_ground & (camera.position.y < 0.5) {
			camera.position.y = 0.5;
		}

		let (target_i, target_x, _) = raycast(&world, camera.i, camera.position, look, TOUCH_DISTANCE);
		if is_mouse_button_pressed(MouseButton::Left) {
			world[target_i].voxel.transparent = true;
		}
		if is_mouse_button_pressed(MouseButton::Right) {
			if ! world[target_i].voxel.transparent {
				let (i, _, _) = raycast(&world, target_i, target_x, -look, 1);
				world[i].voxel.color = selected.color;
				world[i].voxel.transparent = false;
			}
		}

		// Draw pixels

		screen.par_iter_mut().enumerate().for_each(|(i, screen_i)| {
			screen_i.par_iter_mut().enumerate().for_each(|(j, screen_i_j)| {
				let right_coeff = (((i as f32) / (camera.screen.0 as f32) - 0.5) * camera.fov.0).atan();
				let up_coeff = (((j as f32) / (camera.screen.1 as f32) - 0.5) * camera.fov.1).atan();
				let ray = look + right_coeff*right - up_coeff*up;
				let (rayhit_i, rayhit_x, distance) = raycast(&world, camera.i, camera.position, ray, VIEW_DISTANCE);
				let mut fade = 1.7321 * distance / (VIEW_DISTANCE as f32);
				if rayhit_i == target_i {
					fade = 0.5*(fade + 1.0);
				}
				(*screen_i_j).0 = fade*AMBIENT + (1.0 - fade)*world[rayhit_i].voxel.color;
				(*screen_i_j).1 = distance;
			})
		});
		
		screen.iter().enumerate().for_each(|(i, screen_i)| {
			screen_i.iter().enumerate().for_each(|(j, _)| {
				draw_rectangle(
					RECTSIZE_X*(i as f32),
					RECTSIZE_Y*(j as f32),
					RECTSIZE_X,
					RECTSIZE_Y,
					Color::from_vec(screen[i][j].0)
				);
			})
		});

		// Screen text

		draw_text(&format!("{}", (1.0 / get_frame_time()) as usize), 2.0, 16.0, 24.0, WHITE);

		next_frame().await;
	}
}
