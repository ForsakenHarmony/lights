// #![feature(core_intrinsics)]
// #![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

extern crate cortex_m;
#[macro_use(entry)]
extern crate cortex_m_rt as rt;
extern crate embedded_hal as ehal;
extern crate panic_semihosting;
extern crate stm32l4xx_hal as hal;

use crate::hal::{delay::Delay, prelude::*};
use cortex_m_semihosting::hprintln;
use matrix::{effect, hub75::Hub75};

#[entry]
fn main() -> ! {
	let cp = cortex_m::Peripherals::take().unwrap();
	let p = hal::stm32::Peripherals::take().unwrap();

	let mut flash = p.FLASH.constrain();
	let mut rcc = p.RCC.constrain();

	// TRY the other clock configuration
	// let clocks = rcc.cfgr.freeze(&mut flash.acr);
	let clocks = rcc
		.cfgr
		.sysclk(80.mhz())
		.pclk1(80.mhz())
		.pclk2(80.mhz())
		.freeze(&mut flash.acr);

	let mut gpioa = p.GPIOA.split(&mut rcc.ahb2);
	let mut gpiob = p.GPIOB.split(&mut rcc.ahb2);
	let mut gpioc = p.GPIOC.split(&mut rcc.ahb2);

	let r1 = gpioa
		.pa9
		.into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
	let r2 = gpioc
		.pc7
		.into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);
	let g1 = gpiob
		.pb6
		.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
	let g2 = gpioa
		.pa5
		.into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
	let b1 = gpioa
		.pa6
		.into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
	let b2 = gpioa
		.pa7
		.into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);

	let a = gpiob
		.pb11
		.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
	let b = gpiob
		.pb12
		.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
	let c = gpioa
		.pa11
		.into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);
	let d = gpioa
		.pa12
		.into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);

	let clk = gpioc
		.pc0
		.into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);
	let oe = gpiob
		.pb9
		.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
	let lat = gpiob
		.pb8
		.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

	let matrix = Hub75::new((r1, g1, b1, r2, g2, b2, a, b, c, d, clk, lat, oe), 1);

	let delay = Delay::new(cp.SYST, clocks);

	hprintln!("start");

	effect(matrix, delay)
}
