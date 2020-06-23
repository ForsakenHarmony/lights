#![no_std]

pub mod colour;
pub mod hub75;
pub mod iter;
pub mod noise;

use colour::HSV;
use core::ops::DerefMut;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitive_style, primitives::Rectangle};
use embedded_hal::blocking::delay::DelayUs;
use hub75::{Hub75, Outputs};
use iter::MatrixIter;
use noise::simplex;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

pub fn effect_sched<OUT: Outputs>() -> impl FnMut(&mut Hub75<OUT>) {
	// let mut draw = rects();
	let mut draw = cloud();
	return draw;
}

pub fn effect<OUT: Outputs>(mut matrix: Hub75<OUT>, mut delay: impl DelayUs<u8>) -> ! {
	// let mut draw = rects();
	let mut draw = cloud();
	loop {
		draw(&mut matrix);
		matrix.output(&mut delay);
		// sprintln!("Loop");
	}
}

fn _base<T: Outputs>() -> fn(&mut Hub75<T>) {
	return |_matrix| {};
}

fn cloud<T: Outputs>() -> impl FnMut(&mut Hub75<T>) {
	let mut matrix_data = [[HSV::default(); HEIGHT]; WIDTH];

	let mut sin: u8 = 0;
	let mut hue: u8 = 0;

	// const CYCLE_LENGTH: usize = 16;

	let mut x_pos: f32 = 0.0;
	let mut y_pos: f32 = 0.0;

	let draw = move |matrix: &mut Hub75<T>| {
		// sprintln!("1");
		sin = sin + 1;
		hue = hue + 2;

		// sprintln!("2");
		let thing = sin as f32 / 255.0 * 2.0 * core::f32::consts::PI;
		x_pos += libm::sinf(thing);
		y_pos += libm::cosf(thing);

		// sprintln!("3");
		for x in 0..WIDTH {
			// sprintln!("Loomp");
			for y in 0..HEIGHT {
				// let noise_val= (noise((x as f32 + x_pos as f32) / 4.0, (y as f32 + y_pos as f32) / 4.0, 0.0) * 255.0) as u8;
				let noise_val = (simplex(
					(x as f32 + x_pos as f32 / 5.0) / 4.0,
					(y as f32) / 4.0,
					y_pos as f32 / 512.0,
				) * 255.0) as u8;
				let colour = HSV::new(noise_val + hue, noise_val, 255 /* noise_val */);
				matrix_data[x][y] = colour;
				// sprintln!("{:?} {:?}",	colour, hsv2rgb_rainbow(colour));
			}
		}
		// sprintln!("4");
		matrix.draw_iter(MatrixIter::new(&matrix_data)).unwrap();
	};

	return draw;
}

pub struct CloudEffect {
	matrix_data: [[HSV; HEIGHT]; WIDTH],
	sin:         u8,
	hue:         u8,
	x_pos:       f32,
	y_pos:       f32,
}

impl CloudEffect {
	pub fn new() -> Self {
		CloudEffect {
			matrix_data: [[HSV::default(); HEIGHT]; WIDTH],

			sin: 0,
			hue: 0,

			x_pos: 0.0,
			y_pos: 0.0,
		}
	}
}

impl Effect for CloudEffect {
	fn step(&mut self) {
		self.sin = self.sin + 1;
		self.hue = self.hue + 2;

		// sprintln!("2");
		let thing = self.sin as f32 / 255.0 * 2.0 * core::f32::consts::PI;
		self.x_pos += libm::sinf(thing);
		self.y_pos += libm::cosf(thing);

		// sprintln!("3");
		for x in 0..WIDTH {
			// sprintln!("Loomp");
			for y in 0..HEIGHT {
				// let noise_val= (noise((x as f32 + x_pos as f32) / 4.0, (y as f32 + y_pos as f32) / 4.0, 0.0) * 255.0) as u8;
				let noise_val = (simplex(
					(x as f32 + self.x_pos as f32 / 5.0) / 4.0,
					(y as f32) / 4.0,
					self.y_pos as f32 / 512.0,
				) * 255.0) as u8;
				let colour = HSV::new(noise_val + self.hue, noise_val, 255 /* noise_val */);
				self.matrix_data[x][y] = colour;
				// sprintln!("{:?} {:?}",	colour, hsv2rgb_rainbow(colour));
			}
		}
	}

	fn write<PINS: Outputs>(&self, matrix: &mut Hub75<PINS>) {
		matrix
			.draw_iter(MatrixIter::new(&self.matrix_data))
			.unwrap();
	}
}

fn rects<T: Outputs>() -> impl FnMut(&mut Hub75<T>) {
	let mut thing: f32 = 0.0;
	let mut speed: f32 = 0.05;
	return move |matrix| {
		matrix.clear();
		let left = thing as i32;
		let right = thing as i32 + 3;

		Rectangle::new(Point::new(left, 0), Point::new(right, 3))
			.into_styled(primitive_style!(fill_color = Rgb565::RED))
			.draw(matrix)
			.unwrap();
		Rectangle::new(Point::new(63 - right, 4), Point::new(63 - left, 7))
			.into_styled(primitive_style!(fill_color = Rgb565::GREEN))
			.draw(matrix)
			.unwrap();
		Rectangle::new(Point::new(left, 8), Point::new(right, 11))
			.into_styled(primitive_style!(fill_color = Rgb565::BLUE))
			.draw(matrix)
			.unwrap();
		Rectangle::new(Point::new(63 - right, 12), Point::new(63 - left, 15))
			.into_styled(primitive_style!(fill_color = Rgb565::WHITE))
			.draw(matrix)
			.unwrap();
		Rectangle::new(Point::new(left, 16), Point::new(right, 19))
			.into_styled(primitive_style!(fill_color = Rgb565::RED))
			.draw(matrix)
			.unwrap();
		Rectangle::new(Point::new(63 - right, 20), Point::new(63 - left, 23))
			.into_styled(primitive_style!(fill_color = Rgb565::GREEN))
			.draw(matrix)
			.unwrap();
		Rectangle::new(Point::new(left, 24), Point::new(right, 27))
			.into_styled(primitive_style!(fill_color = Rgb565::BLUE))
			.draw(matrix)
			.unwrap();
		Rectangle::new(Point::new(63 - right, 28), Point::new(63 - left, 31))
			.into_styled(primitive_style!(fill_color = Rgb565::WHITE))
			.draw(matrix)
			.unwrap();

		thing += speed;
		if thing > 64.0 {
			speed = -speed;
		}
		if thing < -4.0 {
			speed = -speed;
		}
	};
}

pub fn new_effect() -> impl Effect + Send {
	RectEffect::new()
}

pub trait Effect {
	fn step(&mut self);
	fn write<PINS: Outputs>(&self, matrix: &mut Hub75<PINS>);
}

pub struct RectEffect {
	thing: f32,
	speed: f32,
}

impl RectEffect {
	pub fn new() -> Self {
		RectEffect {
			thing: 0.0,
			speed: 1.0,
		}
	}
}

impl Effect for RectEffect {
	fn step(&mut self) {
		self.thing += self.speed;
		if self.thing > 64.0 {
			self.speed = -self.speed;
		}
		if self.thing < -4.0 {
			self.speed = -self.speed;
		}
	}

	fn write<PINS: Outputs>(&self, mut matrix: &mut Hub75<PINS>) {
		let matrix = matrix.deref_mut();

		matrix.clear();
		let left = self.thing as i32;
		let right = self.thing as i32 + 3;

		Rectangle::new(Point::new(left, 0), Point::new(right, 3))
			.into_styled(primitive_style!(fill_color = Rgb565::RED))
			.draw(matrix)
			.unwrap();
		Rectangle::new(Point::new(63 - right, 4), Point::new(63 - left, 7))
			.into_styled(primitive_style!(fill_color = Rgb565::GREEN))
			.draw(matrix)
			.unwrap();
		Rectangle::new(Point::new(left, 8), Point::new(right, 11))
			.into_styled(primitive_style!(fill_color = Rgb565::BLUE))
			.draw(matrix)
			.unwrap();
		Rectangle::new(Point::new(63 - right, 12), Point::new(63 - left, 15))
			.into_styled(primitive_style!(fill_color = Rgb565::WHITE))
			.draw(matrix)
			.unwrap();
		Rectangle::new(Point::new(left, 16), Point::new(right, 19))
			.into_styled(primitive_style!(fill_color = Rgb565::RED))
			.draw(matrix)
			.unwrap();
		Rectangle::new(Point::new(63 - right, 20), Point::new(63 - left, 23))
			.into_styled(primitive_style!(fill_color = Rgb565::GREEN))
			.draw(matrix)
			.unwrap();
		Rectangle::new(Point::new(left, 24), Point::new(right, 27))
			.into_styled(primitive_style!(fill_color = Rgb565::BLUE))
			.draw(matrix)
			.unwrap();
		Rectangle::new(Point::new(63 - right, 28), Point::new(63 - left, 31))
			.into_styled(primitive_style!(fill_color = Rgb565::WHITE))
			.draw(matrix)
			.unwrap();
	}
}
