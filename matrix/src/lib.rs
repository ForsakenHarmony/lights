#![no_std]

pub mod colour;
pub mod hub75;
pub mod iter;
pub mod noise;

use colour::HSV;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitive_style, primitives::Rectangle};
use embedded_hal::blocking::delay::DelayUs;
use hub75::{Hub75, Outputs};
use iter::MatrixIter;
use noise::simplex;

const WIDTH: usize = 8;
const HEIGHT: usize = 8;

pub fn effect<OUT: Outputs>(mut matrix: Hub75<OUT>, mut delay: impl DelayUs<u8>) -> ! {
	// let draw = rects();
	let mut draw = rects();
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

		// matrix.draw(
		// 	Rectangle::new(Point::new(left, 0), Point::new(right, 3))
		// 		.fill(Some(Rgb565::from((0xff, 0x0, 0x00)))),
		// );
		// matrix.draw(
		// 	Rectangle::new(Point::new(63 - right, 4), Point::new(63 - left, 7))
		// 		.fill(Some(Rgb565::from((0x00, 0xff, 0x00)))),
		// );
		// matrix.draw(
		// 	Rectangle::new(Point::new(left, 8), Point::new(right, 11))
		// 		.fill(Some(Rgb565::from((0x00, 0x0, 0xff)))),
		// );
		// matrix.draw(
		// 	Rectangle::new(Point::new(63 - right, 12), Point::new(63 - left, 15))
		// 		.fill(Some(Rgb565::from((0xff, 0xff, 0xff)))),
		// );
		thing += speed;
		if thing > 64.0 {
			speed = -speed;
		}
		if thing < -4.0 {
			speed = -speed;
		}
	};
}
