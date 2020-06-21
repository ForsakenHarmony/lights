use crate::{colour::HSV, HEIGHT, WIDTH};
use embedded_graphics::{drawable::*, pixelcolor::Rgb565, prelude::Point};

pub struct MatrixIter<'a> {
	x:      usize,
	y:      usize,
	matrix: &'a [[HSV; HEIGHT]; WIDTH],
}

impl<'a> MatrixIter<'a> {
	pub fn new(matrix: &'a [[HSV; HEIGHT]; WIDTH]) -> MatrixIter {
		MatrixIter { x: 0, y: 0, matrix }
	}
}

impl Iterator for MatrixIter<'_> {
	type Item = Pixel<Rgb565>;

	fn next(&mut self) -> Option<Self::Item> {
		// sprintln!("Next");
		if self.x >= WIDTH {
			self.y += 1;
			self.x = 0;
		}
		if self.y >= HEIGHT {
			return None;
		}

		// sprintln!("Inner {}:{}", self.x, self.y);
		let pixel = Pixel(
			Point::new(self.x as i32, self.y as i32),
			self.matrix[self.x][self.y].into(),
		);

		self.x += 1;

		Some(pixel)
	}
}
