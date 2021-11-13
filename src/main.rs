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

// NOTE(elsuizo:2021-11-13): this dispatchers thing is the old extern "C" in the v0.5 and its the
// interrupt source that we attatch to the software task
#[app(device = stm32f1xx_hal::pac, dispatchers = [EXTI2])]
mod app {
    // NOTE(elsuizo:2021-11-13): now the includes go here???
    use stm32f1xx_hal::gpio::{Output, State};
    use stm32f1xx_hal::{
        gpio,
        i2c::{BlockingI2c, DutyCycle, I2c, Mode},
        pac,
        prelude::*,
        rtc::Rtc,
        serial::{self, Config, Serial},
        stm32,
        time::Hertz,
        timer,
    };
    use systick_monotonic::*; // Implements the `Monotonic` trait

    //-------------------------------------------------------------------------
    //                        type alias
    //-------------------------------------------------------------------------
    type Led = gpio::gpioc::PC13<gpio::Output<gpio::PushPull>>;
    // A monotonic timer to enable scheduling in RTIC
    #[monotonic(binds = SysTick, default = true)]
    type MyMono = Systick<30>; // 30 Hz

    // Resources shared between tasks
    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led: Led,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        //-------------------------------------------------------------------------
        //                        hardware initialization
        //-------------------------------------------------------------------------
        // cx.core.DCB.enable_trace();
        let mut flash = cx.device.FLASH.constrain();
        let mut rcc = cx.device.RCC.constrain();
        let mut afio = cx.device.AFIO.constrain(&mut rcc.apb2);
        let mut pwr = cx.device.PWR;
        let mut backup_domain = rcc.bkp.constrain(cx.device.BKP, &mut rcc.apb1, &mut pwr);
        let clocks = rcc
            .cfgr
            .use_hse(8.mhz())
            .sysclk(72.mhz())
            .pclk1(36.mhz())
            .freeze(&mut flash.acr);
        let mut gpioa = cx.device.GPIOA.split(&mut rcc.apb2);
        let mut gpiob = cx.device.GPIOB.split(&mut rcc.apb2);
        let mut gpioc = cx.device.GPIOC.split(&mut rcc.apb2);
        let mut led = gpioc
            .pc13
            .into_push_pull_output_with_state(&mut gpioc.crh, State::Low);
        //-------------------------------------------------------------------------
        //                        rtic initialization
        //-------------------------------------------------------------------------
        let systick = cx.core.SYST;
        let mono = Systick::new(systick, 16_000_000);

        // Spawn the task `bar` 1 second after `init` finishes, this is enabled
        // by the `#[monotonic(..)]` above
        blinky::spawn_after(1.secs()).unwrap();

        // debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator

        (Shared {}, Local { led }, init::Monotonics(mono))
    }

    #[task(local = [led])]
    fn blinky(cx: blinky::Context) {
        // Periodic ever 1 seconds
        cx.local.led.toggle().unwrap();
        blinky::spawn_after(1.secs()).unwrap();
    }
}
