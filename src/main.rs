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

#[app(device = stm32f1xx_hal::pac, dispatchers = [EXTI2, EXTI0])]
mod app {
    use crate::buttons::Button;
    use crate::io::Logger;
    use stm32f1xx_hal::gpio::State;
    use stm32f1xx_hal::{gpio, pac, prelude::*};
    use systick_monotonic::*;

    use embedded_hal::digital::v2::OutputPin;
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
    type Button0Pin = gpio::gpioa::PA5<gpio::Input<gpio::PullUp>>;
    type Button1Pin = gpio::gpioa::PA6<gpio::Input<gpio::PullUp>>;
    type Button2Pin = gpio::gpioa::PA7<gpio::Input<gpio::PullUp>>;
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
        // TODO(elsuizo:2021-11-21): maybe this macro is nice but also i like that the locking
        // resources explicity for gain verbosity
        // #[lock_free]
        led: Led,
    }

    #[local]
    struct Local {
        button0: Button<Button0Pin>,
        button1: Button<Button1Pin>,
        button2: Button<Button2Pin>,
        display: OledDisplay,
        logger: Logger,
        menu_fsm: crate::ui::MenuFSM,
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
        let serial = Serial::usart1(
            cx.device.USART1,
            (tx, rx),
            &mut afio.mapr,
            Config::default().baudrate(9600.bps()),
            clocks,
            &mut rcc.apb2,
        );
        let tx = serial.split().0;
        let logger = Logger::new(tx);
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

        // TODO(elsuizo:2021-11-25): better names for the buttons
        let button0_pin = gpioa.pa5.into_pull_up_input(&mut gpioa.crl);
        let button1_pin = gpioa.pa6.into_pull_up_input(&mut gpioa.crl);
        let button2_pin = gpioa.pa7.into_pull_up_input(&mut gpioa.crl);

        // NOTE(elsuizo:2021-11-24): here we dont need a super fast spawn!!!
        react::spawn_after(1.secs()).unwrap();

        (
            Shared { led },
            Local {
                button0: Button::new(button0_pin),
                button1: Button::new(button1_pin),
                button2: Button::new(button2_pin),
                display,
                logger,
                menu_fsm: crate::ui::MenuFSM::init(crate::ui::MenuState::Row1),
            },
            init::Monotonics(mono),
        )
    }

    //-------------------------------------------------------------------------
    //                        tasks
    //-------------------------------------------------------------------------
    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            continue;
        }
    }
    // NOTE(elsuizo:2021-11-24): the maximum period of this periodic task for a responsive button
    // action is 13 ms
    // NOTE(elsuizo:2021-11-21): remember that the method set_low() needs the trait: `use embedded_hal::digital::v2::OutputPin;`
    // to be used!!!
    #[task(local = [button0, button1, button2])]
    fn react(cx: react::Context) {
        use crate::buttons::PinState::*;
        use crate::ui::Msg::*;

        if let PinUp = cx.local.button0.polling() {
            dispatch_msg::spawn(Button0).ok();
        }
        if let PinUp = cx.local.button1.polling() {
            dispatch_msg::spawn(Button1).ok();
        }
        if let PinUp = cx.local.button2.polling() {
            dispatch_msg::spawn(Button2).ok();
        }
        react::spawn_after(10.millis()).ok();
    }

    #[task(local = [display, logger, menu_fsm], shared = [led])]
    fn dispatch_msg(cx: dispatch_msg::Context, msg: crate::ui::Msg) {
        use crate::ui::Msg::*;
        let dispatch_msg::SharedResources { mut led } = cx.shared;
        cx.local.display.clear();
        cx.local.menu_fsm.next_state(msg);
        match msg {
            Button0 => {
                led.lock(|l| l.toggle().ok());
                cx.local.logger.log("button0 pressed!!!").ok();
                crate::ui::draw_menu(cx.local.display, cx.local.menu_fsm.state).ok();
                cx.local.display.flush().ok();
            }
            Button1 => {
                led.lock(|l| l.toggle().ok());
                cx.local.logger.log("button1 pressed!!!").ok();
                crate::ui::draw_menu(cx.local.display, cx.local.menu_fsm.state).ok();
                cx.local.display.flush().ok();
            }
            Button2 => {
                led.lock(|l| l.toggle().ok());
                cx.local.logger.log("button2 pressed!!!").ok();
                crate::ui::draw_menu(cx.local.display, cx.local.menu_fsm.state).ok();
                cx.local.display.flush().ok();
            }
        };
        // rtic::pend(stm32::Interrupt::EXTI1);
    }
}
