#![no_main]
#![no_std]

use defmt_rtt as _;
use panic_probe as _;

mod state;

#[rtic::app(device = stm32f4xx_hal::pac, peripherals = true, dispatchers = [SPI1])]
mod app {
    use crate::state::{Event, State};
    use embedded_hal::digital::v2::OutputPin;
    use inverted_pin::InvertedPin;
    use stm32f4xx_hal::{self as hal, prelude::*};
    use systick_monotonic::Systick;

    #[monotonic(binds = SysTick, default = true)]
    type MonotonicTimer = Systick<100>; // 100 Hz / 10 ms granularity

    #[shared]
    struct Shared {
        tim2: hal::timer::Counter<hal::pac::TIM2, 1000000>,
    }

    #[local]
    struct Local {
        leds: [InvertedPin<hal::gpio::EPin<hal::gpio::Output>>; 4],
        button: hal::gpio::Pin<'A', 0>,
        state: State,
    }

    #[init]
    fn init(mut ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("init!");

        let clocks = ctx
            .device
            .RCC
            .constrain()
            .cfgr
            .use_hse(8.MHz())
            .sysclk(180.MHz())
            .pclk1(36.MHz())
            .freeze();

        let monotonics = init::Monotonics(Systick::new(ctx.core.SYST, 180_000_000));

        let mut syscfg = ctx.device.SYSCFG.constrain();

        let gpioa = ctx.device.GPIOA.split();
        let gpiod = ctx.device.GPIOD.split();
        let gpiog = ctx.device.GPIOG.split();
        let gpiok = ctx.device.GPIOK.split();

        let mut button = gpioa.pa0;
        button.make_interrupt_source(&mut syscfg);
        button.trigger_on_edge(&mut ctx.device.EXTI, hal::gpio::Edge::Rising);
        button.enable_interrupt(&mut ctx.device.EXTI);

        let mut tim2 = ctx.device.TIM2.counter_us(&clocks);
        tim2.listen(hal::timer::Event::Update);

        let shared = Shared { tim2 };

        let local = Local {
            button,
            state: State::Idle,
            leds: [
                InvertedPin::new(gpiok.pk3.into_push_pull_output().erase()),
                InvertedPin::new(gpiod.pd5.into_push_pull_output().erase()),
                InvertedPin::new(gpiod.pd4.into_push_pull_output().erase()),
                InvertedPin::new(gpiog.pg6.into_push_pull_output().erase()),
            ],
        };

        _ = update_led::spawn(Event::Reset);

        (shared, local, monotonics)
    }

    #[task(binds = EXTI0, local = [button], priority = 2)]
    fn press(ctx: press::Context) {
        defmt::info!("button pressed");
        ctx.local.button.clear_interrupt_pending_bit();
        _ = update_led::spawn(Event::ButtonPress);
    }

    #[task(binds = TIM2, shared = [tim2], priority = 2)]
    fn handle_timer(mut ctx: handle_timer::Context) {
        ctx.shared.tim2.lock(|t| t.clear_all_flags());
        _ = update_led::spawn(Event::TimerElapsed);
    }

    #[task(local=[state, leds], shared = [tim2], priority = 1)]
    fn update_led(mut ctx: update_led::Context, event: Event) {
        ctx.local.state.update(event);
        let n = ctx.local.state.value();

        for (i, led) in ctx.local.leds.iter_mut().enumerate() {
            let on = (n & (1 << i)) != 0;
            led.set_state(on.into()).expect("set pin");
        }

        if ctx.local.state.active() {
            ctx.shared
                .tim2
                .lock(|t| t.start(200.millis()))
                .expect("start timer");
        }
    }
}
