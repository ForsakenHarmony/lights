use embedded_graphics::pixelcolor::Rgb565;

#[derive(Copy, Clone, Debug)]
pub struct HSV {
	hue:        u8,
	saturation: u8,
	value:      u8,
}

impl HSV {
	pub fn new(hue: u8, saturation: u8, value: u8) -> Self {
		HSV {
			hue,
			saturation,
			value,
		}
	}
}

impl Into<Rgb565> for HSV {
	fn into(self) -> Rgb565 {
		hsv2rgb_rainbow(self)
	}
}

impl Default for HSV {
	fn default() -> Self {
		HSV::new(0, 0, 0)
	}
}

// from fastled
fn scale8(i: u8, scale: u8) -> u8 {
	(((i as u16) * (1 + scale as u16)) >> 8) as u8
}

// from fastled
fn scale8_video(i: u8, scale: u8) -> u8 {
	(((i as usize * scale as usize) >> 8) + if i > 0 && scale > 0 { 1 } else { 0 }) as u8
}

// from fastled
fn hsv2rgb_rainbow(hsv: HSV) -> Rgb565 {
	const K255: u8 = 255;
	const K171: u8 = 171;
	const K170: u8 = 170;
	const K85: u8 = 85;

	// Yellow has a higher inherent brightness than
	// any other color; 'pure' yellow is perceived to
	// be 93% as bright as white.  In order to make
	// yellow appear the correct relative brightness,
	// it has to be rendered brighter than all other
	// colors.
	// Level Y1 is a moderate boost, the default.
	// Level Y2 is a strong boost.
	const Y1: bool = true;
	const Y2: bool = false;

	// G2: Whether to divide all greens by two.
	// Depends GREATLY on your particular LEDs
	const G2: bool = false;

	// GSCALE: what to scale green down by.
	// Depends GREATLY on your particular LEDs
	const GSCALE: u8 = 0;

	let hue: u8 = hsv.hue;
	let sat: u8 = hsv.saturation;
	let mut val: u8 = hsv.value;

	let offset: u8 = hue & 0x1F; // 0..31

	// offset8 = offset * 8
	let mut offset8: u8 = offset;
	{
		offset8 <<= 3;
	}

	let third: u8 = scale8(offset8, (256u16 / 3) as u8); // max = 85

	let mut r = 0;
	let mut g = 0;
	let mut b = 0;

	if hue & 0x80 == 0 {
		// 0XX
		if hue & 0x40 == 0 {
			// 00X
			//section 0-1
			if hue & 0x20 == 0 {
				// 000
				//case 0: // R -> O
				r = K255 - third;
				g = third;
				b = 0;
			} else {
				// 001
				//case 1: // O -> Y
				if Y1 {
					r = K171;
					g = K85 + third;
					b = 0;
				}
				if Y2 {
					r = K170 + third;
					//uint8_t twothirds = (third << 1);
					let twothirds = scale8(offset8, ((256 * 2) / 3) as u8); // max=170
					g = K85 + twothirds;
					b = 0;
				}
			}
		} else {
			//01X
			// section 2-3
			if hue & 0x20 == 0 {
				// 010
				//case 2: // Y -> G
				if Y1 {
					//uint8_t twothirds = (third << 1);
					let twothirds = scale8(offset8, ((256 * 2) / 3) as u8); // max=170
					r = K171 - twothirds;
					g = K170 + third;
					b = 0;
				}
				if Y2 {
					r = K255 - offset8;
					g = K255;
					b = 0;
				}
			} else {
				// 011
				// case 3: // G -> A
				r = 0;
				g = K255 - third;
				b = third;
			}
		}
	} else {
		// section 4-7
		// 1XX
		if hue & 0x40 == 0 {
			// 10X
			if hue & 0x20 == 0 {
				// 100
				//case 4: // A -> B
				r = 0;
				//uint8_t twothirds = (third << 1);
				let twothirds = scale8(offset8, ((256 * 2) / 3) as u8); // max=170
				g = K171 - twothirds; //K170?
				b = K85 + twothirds;
			} else {
				// 101
				//case 5: // B -> P
				r = third;
				g = 0;

				b = K255 - third;
			}
		} else {
			if hue & 0x20 == 0 {
				// 110
				//case 6: // P -- K
				r = K85 + third;
				g = 0;

				b = K171 - third;
			} else {
				// 111
				//case 7: // K -> R
				r = K170 + third;
				g = 0;

				b = K85 - third;
			}
		}
	}

	// This is one of the good places to scale the green down,
	// although the client can scale green down as well.
	if G2 {
		g = g >> 1;
	}
	if GSCALE > 0 {
		g = scale8_video(g, GSCALE);
	}

	// Scale down colors if we're desaturated at all
	// and add the brightness_floor to r, g, and b.
	if sat != 255 {
		if sat == 0 {
			r = 255;
			b = 255;
			g = 255;
		} else {
			//nscale8x3_video( r, g, b, sat);
			if r > 0 {
				r = scale8(r, sat)
			}
			if g > 0 {
				g = scale8(g, sat)
			}
			if b > 0 {
				b = scale8(b, sat)
			}

			let mut desat = 255 - sat;
			desat = scale8(desat, desat);

			let brightness_floor = desat;
			r += brightness_floor;
			g += brightness_floor;
			b += brightness_floor;
		}
	}

	// Now scale everything down if we're at value < 255.
	if val != 255 {
		val = scale8_video(val, val);
		if val == 0 {
			r = 0;
			g = 0;
			b = 0;
		} else {
			if r > 0 {
				r = scale8(r, val)
			}
			if g > 0 {
				g = scale8(g, val)
			}
			if b > 0 {
				b = scale8(b, val)
			}
		}
	}

	Rgb565::new(r, g, b)
}
