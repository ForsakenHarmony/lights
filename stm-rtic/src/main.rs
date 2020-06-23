#![no_main]
#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

use alloc::boxed::Box;
use core::{
	alloc::{GlobalAlloc, Layout},
	borrow::BorrowMut,
	convert::TryInto,
};
use cortex_m::{
	asm::delay,
	interrupt::{free, Mutex},
	peripheral::DWT,
};
use cortex_m_rt::{exception, ExceptionFrame};
use cortex_m_semihosting::{debug, hprintln};
use embedded_hal::blocking::delay::DelayUs;
use heapless::pool;
use matrix::{effect_sched, hub75::Hub75, CloudEffect, Effect, RectEffect};
use panic_semihosting as _;
use rtic::{
	app,
	cyccnt::{Instant, U32Ext},
};
use stm32l4xx_hal::{
	gpio::{
		gpioa::{Parts as PartsA, *},
		gpiob::{Parts as PartsB, *},
		gpioc::{Parts as PartsC, *},
		Output,
		PushPull,
	},
	prelude::*,
	rcc::Rcc,
	stm32l4::stm32l4x6::Interrupt,
	time::Hertz,
};

type Outputs = (
	PA9<Output<PushPull>>, // R1
	PB6<Output<PushPull>>, // R2
	PA6<Output<PushPull>>, // G1
	PC7<Output<PushPull>>, // G2
	PA5<Output<PushPull>>, // B1
	PA7<Output<PushPull>>, // B2
	//
	PB11<Output<PushPull>>, // A
	PB12<Output<PushPull>>, // B
	PA11<Output<PushPull>>, // C
	PA12<Output<PushPull>>, // D
	//
	PC9<Output<PushPull>>, // CLK
	PB8<Output<PushPull>>, // OE
	PB9<Output<PushPull>>, // LAT
);

const REFRESH_PERIOD: u32 = 320_000;
const STEP_PERIOD: u32 = 1_333_333;

pub struct Delay {
	freq: Hertz,
}

impl Delay {
	pub fn new(freq: Hertz) -> Self {
		Delay { freq }
	}
}

impl DelayUs<u8> for Delay {
	fn delay_us(&mut self, us: u8) {
		self.delay_us(us as u32);
	}
}

impl DelayUs<u32> for Delay {
	fn delay_us(&mut self, us: u32) {
		let cycles = us * (self.freq.0 / 1_000_000);
		delay(1);
	}
}

pool!(P: [u8; 4096]);

#[global_allocator]
static ALLOC: P = P {};

unsafe impl GlobalAlloc for P {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		P::alloc(self, layout)
	}

	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		P::dealloc(self, ptr, layout);
	}
}

#[alloc_error_handler]
fn foo(ly: core::alloc::Layout) -> ! {
	panic!("failed to allocate {:?}", ly);
}

#[app(device = stm32l4xx_hal::stm32, peripherals = true, monotonic = rtic::cyccnt::CYCCNT)]
const APP: () = {
	struct Resources {
		matrix: Hub75<Outputs>,
		delay:  Delay,
		step:   CloudEffect,
	}

	#[init(schedule = [refresh_matrix, refresh_effect, write_effect])]
	fn init(mut cx: init::Context) -> init::LateResources {
		// Initialize (enable) the monotonic timer (CYCCNT)
		cx.core.DCB.enable_trace();
		// required on Cortex-M7 devices that software lock the DWT (e.g. STM32F7)
		DWT::unlock();
		cx.core.DWT.enable_cycle_counter();

		// Cortex-M peripherals
		let cp: rtic::Peripherals = cx.core;
		// Device specific peripherals
		let dp: stm32l4xx_hal::stm32::Peripherals = cx.device;

		let mut flash = dp.FLASH.constrain();
		let mut rcc = dp.RCC.constrain();

		let clocks = rcc
			.cfgr
			.sysclk(80.mhz())
			.pclk1(80.mhz())
			.pclk2(80.mhz())
			.freeze(&mut flash.acr);

		let gpioa = dp.GPIOA.split(&mut rcc.ahb2);
		let gpiob = dp.GPIOB.split(&mut rcc.ahb2);
		let gpioc = dp.GPIOC.split(&mut rcc.ahb2);

		let pins = init_pins(gpioa, gpiob, gpioc);

		let mut matrix = Hub75::new(pins, 3);
		let delay = Delay::new(clocks.sysclk());
		let mut step = CloudEffect::new();
		step.step();
		step.write(&mut matrix);
		// let mut step = effect_sched();

		// semantically, the monotonic timer is frozen at time "zero" during `init`
		// NOTE do *not* call `Instant::now` in this context; it will return a nonsense value
		let now = cx.start; // the start time of the system

		hprintln!("init @ {:?}", now).unwrap();

		cx.schedule
			.refresh_matrix(now + REFRESH_PERIOD.cycles())
			.unwrap();
		// cx.schedule
		// 	.refresh_effect(now + STEP_PERIOD.cycles())
		// 	.unwrap();
		cx.schedule
			.write_effect(now + STEP_PERIOD.cycles())
			.unwrap();
		// rtic::pend(Interrupt::UART4);

		// let step = Mutex::new(Box::new(step));
		// let matrix = matrix;

		init::LateResources {
			matrix,
			delay,
			step,
		}
	}

	// #[task(schedule = [refresh_matrix])]
	// fn refresh_matrix(cx: refresh_matrix::Context) {
	// 	hprintln!("refresh_matrix @ {:?}", Instant::now()).unwrap();
	// 	cx.schedule
	// 		.refresh_matrix(cx.scheduled + PERIOD.cycles())
	// 		.unwrap();
	// }

	#[task(priority = 255, resources = [matrix, delay], schedule = [refresh_matrix])]
	fn refresh_matrix(cx: refresh_matrix::Context) {
		// hprintln!("refresh_matrix @ {:?}", Instant::now()).unwrap();
		// let matrix_mutex: &mut Mutex<Hub75<_>> = cx.resources.matrix;
		// free(|cs| {
		// 	let mut matrix = matrix_mutex.borrow(cs);
		// 	matrix.output(&mut (*cx.resources.delay));
		// });
		cx.resources.matrix.output(&mut (*cx.resources.delay));
		cx.schedule
			.refresh_matrix(cx.scheduled + REFRESH_PERIOD.cycles())
			.unwrap();
	}

	#[task(resources = [step], schedule = [write_effect])]
	fn refresh_effect(mut cx: refresh_effect::Context) {
		// hprintln!("refresh_effect @ {:?}", Instant::now()).unwrap();
		let step = &mut *cx.resources.step;
		let before = Instant::now();
		step.step();
		let after = Instant::now();
		let duration: u32 = (after - before).try_into().unwrap();
		// hprintln!(
		// 	"refresh_effect @ {:?}, took: {:?}",
		// 	Instant::now(),
		// 	duration
		// )
		// .unwrap();
		cx.schedule
			.write_effect(cx.scheduled + STEP_PERIOD.cycles())
			.unwrap();
	}

	#[task(resources = [matrix, step], schedule = [refresh_effect, write_effect])]
	fn write_effect(mut cx: write_effect::Context) {
		// hprintln!("refresh_effect @ {:?}", Instant::now()).unwrap();
		let step = &mut *cx.resources.step;
		let before = Instant::now();
		cx.resources.matrix.lock(|mut matrix| {
			step.write(&mut matrix);
		});
		let after = Instant::now();
		let duration: u32 = (after - before).try_into().unwrap();
		// hprintln!("write_effect @ {:?}, took: {:?}", Instant::now(), duration).unwrap();
		cx.schedule
			.write_effect(cx.scheduled + STEP_PERIOD.cycles())
			.unwrap();
	}

	#[idle]
	fn idle(cx: idle::Context) -> ! {
		rtic::pend(Interrupt::UART4);
		rtic::pend(Interrupt::UART5);
		// extern "C" {
		// 	fn UART4();
		// }
		//
		// unsafe { UART4() }

		// cx.schedule
		// 	.refresh_matrix(Instant::now() + PERIOD.cycles())
		// 	.unwrap();

		loop {}
	}

	extern "C" {
		fn UART4();
		fn UART5();
	}
};

fn init_pins(mut gpioa: PartsA, mut gpiob: PartsB, mut gpioc: PartsC) -> Outputs {
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
		.pc9
		.into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);
	let oe = gpiob
		.pb9
		.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
	let lat = gpiob
		.pb8
		.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

	(r1, g1, b1, r2, g2, b2, a, b, c, d, clk, lat, oe)
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
	panic!("{:#?}", ef);
}
