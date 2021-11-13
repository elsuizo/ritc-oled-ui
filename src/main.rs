//----------------------------------------------------------------------------
// @date 2021-11-13
// @author Martin Noblia
// TODOs
// - [X] Periodic task blinky compile and working
// - [ ] include the rtc clock
// - [ ] include the oled display
// - [ ] do the menu with buttons
//----------------------------------------------------------------------------
#![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

use panic_semihosting as _;
use rtic::app;

#[app(device = stm32f1xx_hal::pac, dispatchers = [EXTI2])]
mod app {
    use stm32f1xx_hal::gpio::State;
    use stm32f1xx_hal::{gpio, prelude::*};
    use systick_monotonic::*;

    //-------------------------------------------------------------------------
    //                        type alias
    //-------------------------------------------------------------------------
    type Led = gpio::gpioc::PC13<gpio::Output<gpio::PushPull>>;
    // A monotonic timer to enable scheduling in RTIC
    #[monotonic(binds = SysTick, default = true)]
    type MyMono = Systick<72>; // 30 Hz

    //-------------------------------------------------------------------------
    //                        resources declaration
    //-------------------------------------------------------------------------
    // Resources shared between tasks
    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: Led,
    }

    //-------------------------------------------------------------------------
    //                        initialization fn
    //-------------------------------------------------------------------------
    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        //-------------------------------------------------------------------------
        //                        hardware initialization
        //-------------------------------------------------------------------------
        let mut rcc = cx.device.RCC.constrain();
        let mut gpioc = cx.device.GPIOC.split(&mut rcc.apb2);
        let led = gpioc
            .pc13
            .into_push_pull_output_with_state(&mut gpioc.crh, State::Low);
        //-------------------------------------------------------------------------
        //                        rtic initialization
        //-------------------------------------------------------------------------
        let systick = cx.core.SYST;
        let mono = Systick::new(systick, 8_000_000);

        // Spawn the task `blinky` 1 second after `init` finishes, this is enabled
        // by the `#[monotonic(..)]` above
        blinky::spawn_after(1.secs()).unwrap();

        (Shared {}, Local { led }, init::Monotonics(mono))
    }

    #[task(local = [led])]
    fn blinky(cx: blinky::Context) {
        // Periodic ever 1 seconds
        cx.local.led.toggle().unwrap();
        blinky::spawn_after(1.secs()).unwrap();
    }
}
