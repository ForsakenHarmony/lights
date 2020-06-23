// #![feature(core_intrinsics)]
// #![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

extern crate cortex_m;
extern crate cortex_m_rt as rt;
extern crate embedded_hal as ehal;
extern crate panic_semihosting;
extern crate stm32l4xx_hal as hal;

use crate::hal::{
	delay::Delay,
	gpio::{
		gpioa::{Parts as PartsA, *},
		gpiob::{Parts as PartsB, *},
		gpioc::{Parts as PartsC, *},
		Output,
		PushPull,
	},
	interrupt,
	prelude::*,
	stm32::Interrupt,
};
use core::{
	borrow::BorrowMut,
	cell::{Cell, RefCell, UnsafeCell},
	ops::DerefMut,
};
use cortex_m::{
	interrupt::{free, Mutex},
	peripheral::NVIC,
};
use cortex_m_semihosting::hprintln;
use hal::timer::{Event, Timer};
use matrix::{hub75::Hub75, Effect as _, RectEffect};
use rt::{entry, exception, ExceptionFrame};

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

static mut MATRIX: Mutex<RefCell<Option<(Hub75<Outputs>, Delay)>>> = Mutex::new(RefCell::new(None));

fn use_matrix<T: Fn(&mut Hub75<Outputs>, &mut Delay)>(func: T) {
	free(|cs| unsafe {
		let mut data_ref = MATRIX.borrow(cs).borrow_mut();
		if let Some(ref mut data) = data_ref.deref_mut() {
			func(&mut data.0, &mut data.1)
		}
	});
}

#[entry]
fn main() -> ! {
	let cp = cortex_m::Peripherals::take().unwrap();
	let dp = hal::stm32::Peripherals::take().unwrap();

	let mut flash = dp.FLASH.constrain();
	let mut rcc = dp.RCC.constrain();
	let mut pwr = dp.PWR.constrain(&mut rcc.apb1r1);

	// TRY the other clock configuration
	// let clocks = rcc.cfgr.freeze(&mut flash.acr);
	let clocks = rcc
		.cfgr
		.sysclk(80.mhz())
		.pclk1(80.mhz())
		.pclk2(80.mhz())
		.freeze(&mut flash.acr, &mut pwr);

	let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);
	let mut gpiob = dp.GPIOB.split(&mut rcc.ahb2);
	let mut gpioc = dp.GPIOC.split(&mut rcc.ahb2);

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

	unsafe {
		NVIC::unmask(Interrupt::TIM7);
	}

	let mut timer = Timer::tim7(dp.TIM7, 1.hz(), clocks, &mut rcc.apb1r1);

	let mut matrix: Hub75<Outputs> =
		Hub75::new((r1, g1, b1, r2, g2, b2, a, b, c, d, clk, lat, oe), 1);
	let mut delay = Delay::new(cp.SYST, clocks);

	free(|cs| unsafe {
		MATRIX.borrow(cs).replace(Some((matrix, delay)));
	});

	timer.clear_interrupt(Event::TimeOut);
	hprintln!("start");

	let mut effect = RectEffect::new();
	effect.step();
	use_matrix(|matrix, _| effect.write(matrix));

	hprintln!("effect");

	timer.listen(Event::TimeOut);
	hprintln!("bonk");
	loop {
		effect.step();
		use_matrix(|matrix, _| effect.write(matrix));
		hprintln!("loop");
	}
}

#[interrupt]
fn TIM7() {
	use_matrix(|matrix, delay| {
		matrix.output(delay);
	});
	let dp = unsafe { hal::stm32::Peripherals::steal() };
	dp.TIM7.sr.write(|w| w.uif().clear_bit());
	// hprintln!("timer");
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
	panic!("{:#?}", ef);
}
