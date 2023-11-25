#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [SPI1])]
mod app {
    use stm32f4xx_hal::{
        self as hal,
        gpio::{Output, PushPull},
        prelude::*,
    };
    use systick_monotonic::{fugit::Duration, Systick};

    #[monotonic(binds = SysTick, default = true)]
    type MonotonicTimer = Systick<100>; // 100 Hz / 10 ms granularity

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: hal::gpio::PG6<Output<PushPull>>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let _clocks = ctx
            .device
            .RCC
            .constrain()
            .cfgr
            .use_hse(8.MHz())
            .sysclk(180.MHz())
            .pclk1(36.MHz())
            .freeze();

        let monotonics = init::Monotonics(Systick::new(ctx.core.SYST, 180_000_000));

        // let timer = ctx.device.TIM2.

        blink::spawn().unwrap();

        defmt::info!("init!");

        let gpiog = ctx.device.GPIOG.split();

        let shared = Shared {};
        let local = Local {
            led: gpiog.pg6.into_push_pull_output(),
        };

        (shared, local, monotonics)
    }

    #[task(local = [led])]
    fn blink(ctx: blink::Context) {
        ctx.local.led.set_state(!ctx.local.led.get_state());

        blink::spawn_after(Duration::<u64, 1, 100>::millis(1000)).unwrap();
    }
}
