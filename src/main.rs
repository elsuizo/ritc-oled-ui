//----------------------------------------------------------------------------
// @date 2021-11-13
// @author Martin Noblia
// TODOs
// - [X] Periodic task blinky compile and working
// - [ ] include the oled display
// - [ ] do the menu with buttons
//  - [ ] read the buttons
//  - [ ] generate a state machine with the menu states
// - [ ] enable UART debug
//----------------------------------------------------------------------------
#![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

mod buttons;
mod io;
mod ui;
use panic_semihosting as _;
use rtic::app;

#[app(device = stm32f1xx_hal::pac, dispatchers = [EXTI2])]
mod app {
    use crate::buttons::Button;
    use crate::io::Logger;
    use stm32f1xx_hal::gpio::State;
    use stm32f1xx_hal::{gpio, pac, prelude::*};
    use systick_monotonic::*;

    use pac::I2C1;
    use sh1106::{prelude::*, Builder};
    use stm32f1xx_hal::{
        i2c::{BlockingI2c, DutyCycle, Mode},
        prelude::*,
        serial::{self, Config, Serial},
        stm32,
    };
    //-------------------------------------------------------------------------
    //                        type alias
    //-------------------------------------------------------------------------
    type Led = gpio::gpioc::PC13<gpio::Output<gpio::PushPull>>;
    type Sda = gpio::gpiob::PB9<gpio::Alternate<gpio::OpenDrain>>;
    type Scl = gpio::gpiob::PB8<gpio::Alternate<gpio::OpenDrain>>;
    type Button0Pin = gpio::gpioa::PA6<gpio::Input<gpio::PullUp>>;
    type OledDisplay = GraphicsMode<I2cInterface<BlockingI2c<I2C1, (Scl, Sda)>>>;
    // A monotonic timer to enable scheduling in RTIC
    #[monotonic(binds = SysTick, default = true)]
    type MyMono = Systick<72>;

    //-------------------------------------------------------------------------
    //                        resources declaration
    //-------------------------------------------------------------------------
    // Resources shared between tasks
    #[shared]
    struct Shared {
        counter: usize,
    }

    #[local]
    struct Local {
        button0: Button<Button0Pin>,
        led: Led,
        display: OledDisplay,
        logger: Logger,
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
        let mut flash = cx.device.FLASH.constrain();
        let clocks = rcc.cfgr.freeze(&mut flash.acr);
        let mut afio = cx.device.AFIO.constrain(&mut rcc.apb2);

        let mut gpioa = cx.device.GPIOA.split(&mut rcc.apb2);
        let mut gpiob = cx.device.GPIOB.split(&mut rcc.apb2);
        let mut gpioc = cx.device.GPIOC.split(&mut rcc.apb2);
        let led = gpioc
            .pc13
            .into_push_pull_output_with_state(&mut gpioc.crh, State::Low);

        // USART1
        let tx = gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl);
        let rx = gpiob.pb7;
        let mut serial = Serial::usart1(
            cx.device.USART1,
            (tx, rx),
            &mut afio.mapr,
            Config::default().baudrate(9600.bps()),
            clocks,
            &mut rcc.apb2,
        );
        let tx = serial.split().0;
        let mut logger = Logger::new(tx);
        // oled display pins
        let scl = gpiob.pb8.into_alternate_open_drain(&mut gpiob.crh);
        let sda = gpiob.pb9.into_alternate_open_drain(&mut gpiob.crh);
        let i2c = BlockingI2c::i2c1(
            cx.device.I2C1,
            (scl, sda),
            &mut afio.mapr,
            Mode::Fast {
                frequency: 100.khz().into(),
                duty_cycle: DutyCycle::Ratio2to1,
            },
            clocks,
            &mut rcc.apb1,
            1000,
            10,
            1000,
            1000,
        );
        //-------------------------------------------------------------------------
        //                        rtic initialization
        //-------------------------------------------------------------------------

        let mut display: GraphicsMode<_> = Builder::new().connect_i2c(i2c).into();
        display.init().unwrap();
        display.flush().unwrap();
        let systick = cx.core.SYST;
        let mono = Systick::new(systick, 8_000_000);

        let button0_pin = gpioa.pa6.into_pull_up_input(&mut gpioa.crl);
        // Spawn the task `blinky` 1 second after `init` finishes, this is enabled
        // by the `#[monotonic(..)]` above
        blinky::spawn_after(1.secs()).unwrap();

        (
            Shared { counter: 0 },
            Local {
                button0: Button::new(button0_pin),
                led,
                display,
                logger,
            },
            init::Monotonics(mono),
        )
    }
    //-------------------------------------------------------------------------
    //                        tasks
    //-------------------------------------------------------------------------

    #[idle(shared = [counter])]
    fn idle(mut ctx: idle::Context) -> ! {
        let idle::SharedResources { mut counter } = ctx.shared;
        loop {
            counter.lock(|c| {
                *c += 1;
            });
            continue;
        }
    }

    #[task(local = [display, button0])]
    fn react(cx: react::Context) {
        use crate::ui::draw_text;
        if let crate::buttons::Event::Pressed = cx.local.button0.poll() {
            cx.local.display.flush().unwrap();
            draw_text(cx.local.display, "button pressed!!!", 0, 24).unwrap();
        }
    }

    #[task(local = [led, logger], shared = [counter])]
    fn blinky(cx: blinky::Context) {
        use core::fmt::Write;
        use heapless::String;
        // Periodic ever 1 seconds
        cx.local.led.toggle().unwrap();

        blinky::spawn_after(1.secs()).unwrap();
    }
}
