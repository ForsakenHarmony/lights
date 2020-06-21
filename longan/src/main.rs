#![no_std]
#![no_main]

use core::{
	panic::PanicInfo,
	sync::{atomic, atomic::Ordering},
};
use gd32vf103xx_hal::{delay::McycleDelay, pac, prelude::*, timer::Timer};
use longan_nano::sprintln;
use matrix::{effect, hub75::Hub75};
use riscv_rt::entry;

#[entry]
fn main() -> ! {
	let dp = pac::Peripherals::take().unwrap();

	// Configure clocks
	let mut rcu = dp
		.RCU
		.configure()
		.ext_hf_clock(8.mhz())
		.sysclk(108.mhz())
		.freeze();
	let mut afio = dp.AFIO.constrain(&mut rcu);

	let gpioa = dp.GPIOA.split(&mut rcu);
	let gpiob = dp.GPIOB.split(&mut rcu);
	// let gpioc = dp.GPIOC.split(&mut rcu);
	// let (_, _, _, _, pb4) =
	// afio.disable_jtag(gpioa.pa13, gpioa.pa14, gpioa.pa15, gpiob.pb3, gpiob.pb4);

	// let (mut red, mut green, mut blue) = rgb(gpioc.pc13, gpioa.pa1, gpioa.pa2);
	// let leds = unsafe {
	// 	LEDS.store(&mut Some(&mut red), Ordering::SeqCst);
	// 	LEDS.get_mut().as_ref().unwrap().unwrap()
	// };

	longan_nano::stdout::configure(
		dp.USART0,
		gpioa.pa9,
		gpioa.pa10,
		115_200.bps(),
		&mut afio,
		&mut rcu,
	);

	// let sck = gpioa.pa5.into_alternate_push_pull();
	// let miso = gpioa.pa6.into_floating_input();
	// let mosi = gpioa.pa7.into_alternate_push_pull();
	// let spi0 = Spi::spi0(dp.SPI0, (sck, miso, mosi), MODE_0, 50.mhz(), &clocks);

	// let dc = gpiob.pb0.into_push_pull_output();
	// let rst = gpiob.pb1.into_push_pull_output();
	// let mut cs = gpiob.pb2.into_push_pull_output();
	// cs.set_low().unwrap();

	let r1 = gpiob.pb0.into_push_pull_output();
	let r2 = gpiob.pb5.into_push_pull_output();
	let g1 = gpiob.pb6.into_push_pull_output();
	let g2 = gpiob.pb7.into_push_pull_output();
	let b1 = gpiob.pb8.into_push_pull_output();
	let b2 = gpiob.pb9.into_push_pull_output();

	let a = gpiob.pb12.into_push_pull_output();
	let b = gpiob.pb13.into_push_pull_output();
	let c = gpiob.pb14.into_push_pull_output();
	let d = gpiob.pb15.into_push_pull_output();

	let clk = gpiob.pb1.into_push_pull_output();
	let oe = gpiob.pb10.into_push_pull_output();
	let lat = gpiob.pb11.into_push_pull_output();

	let matrix = Hub75::new((r1, g1, b1, r2, g2, b2, a, b, c, d, clk, lat, oe), 1);

	let delay = McycleDelay::new(&rcu.clocks);

	sprintln!("start");

	effect(matrix, delay)
}

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	sprintln!("{}", info);
	// unsafe {
	// 	let leds = LEDS.get_mut().as_ref();
	// 	leds.and_then(|o| o.map(|leds| leds.on()));
	// }
	loop {
		atomic::compiler_fence(Ordering::SeqCst);
	}
}
