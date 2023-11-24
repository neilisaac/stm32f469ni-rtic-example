#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [SPI1])]
mod app {
    use stm32f4xx_hal::prelude::*;
    use systick_monotonic::Systick;

    #[monotonic(binds = SysTick, default = true)]
    type MonotonicTimer = Systick<100>; // 100 Hz / 10 ms granularity

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        foo: i32,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        ctx.device
            .RCC
            .constrain()
            .cfgr
            .use_hse(8.MHz())
            .sysclk(180.MHz())
            .pclk1(36.MHz())
            .freeze();

        let monotonics = init::Monotonics(Systick::new(ctx.core.SYST, 180_000_000));

        startup::spawn().unwrap();
        defmt::info!("init!");

        let shared = Shared {};
        let local = Local { foo: 42 };

        (shared, local, monotonics)
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        defmt::info!("idle!");
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(local = [foo], priority = 1)]
    fn startup(ctx: startup::Context) {
        defmt::info!("starting! {}", ctx.local.foo);
    }
}
