//----------------------------------------------------------------------------
// @date 2021-11-13
// @author Martin Noblia
// TODOs
// - [X] Periodic task blinky compile and working
// - [X] include the oled display
// - [X] do the menu with buttons
// - [X] enable UART debug
//  - [X] read the buttons
//  - [X] generate a state machine with the menu states
//----------------------------------------------------------------------------
#![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

mod buttons;
mod io;
mod ui;
use crate::buttons::Button;
use crate::io::Logger;
// use crate::Systick;
// use panic_rtt_target as _;
use panic_semihosting as _;
use rtic::app;
use stm32f1xx_hal::gpio::PinState;
use stm32f1xx_hal::{gpio, pac, prelude::*};

use pac::I2C1;
use sh1106::{prelude::*, Builder};
use stm32f1xx_hal::{
    i2c::{BlockingI2c, DutyCycle, Mode},
    serial::{Config, Serial},
};
use systick_monotonic::{fugit::Duration, Systick};

#[app(device = stm32f1xx_hal::pac, peripherals = true, dispatchers = [SPI2])]
mod app {
    use super::*;
    //-------------------------------------------------------------------------
    //                        type alias
    //-------------------------------------------------------------------------
    type Led = gpio::gpioc::PC13<gpio::Output<gpio::PushPull>>;
    type Sda = gpio::gpiob::PB9<gpio::Alternate<gpio::OpenDrain>>;
    type Scl = gpio::gpiob::PB8<gpio::Alternate<gpio::OpenDrain>>;
    type ButtonUpPin = gpio::gpioa::PA5<gpio::Input<gpio::PullUp>>;
    type ButtonDownPin = gpio::gpioa::PA6<gpio::Input<gpio::PullUp>>;
    type ButtonEnterPin = gpio::gpioa::PA7<gpio::Input<gpio::PullUp>>;
    type OledDisplay = GraphicsMode<I2cInterface<BlockingI2c<I2C1, (Scl, Sda)>>>;
    // NOTE(elsuizo: 2023-03-31): old monotonic timer
    // A monotonic timer to enable scheduling in RTIC
    // #[monotonic(binds = SysTick, default = true)]
    // type MyMono = Systick<72>;

    #[monotonic(binds = SysTick, default = true)]
    type MonoTimer = Systick<1000>;
    //-------------------------------------------------------------------------
    //                        resources declaration
    //-------------------------------------------------------------------------
    // Resources shared between tasks
    #[shared]
    struct Shared {
        led: Led,
    }

    #[local]
    struct Local {
        button_up: Button<ButtonUpPin>,
        button_down: Button<ButtonDownPin>,
        button_enter: Button<ButtonEnterPin>,
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
        let rcc = cx.device.RCC.constrain();
        let mut flash = cx.device.FLASH.constrain();
        let clocks = rcc.cfgr.freeze(&mut flash.acr);
        let mut afio = cx.device.AFIO.constrain();

        let mut gpioa = cx.device.GPIOA.split();
        let mut gpiob = cx.device.GPIOB.split();
        let mut gpioc = cx.device.GPIOC.split();
        let led = gpioc
            .pc13
            .into_push_pull_output_with_state(&mut gpioc.crh, PinState::Low);

        // USART1
        let tx = gpiob.pb6.into_alternate_push_pull(&mut gpiob.crl);
        let rx = gpiob.pb7;
        let serial = Serial::new(
            cx.device.USART1,
            (tx, rx),
            &mut afio.mapr,
            Config::default().baudrate(9600.bps()),
            &clocks,
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
                frequency: 100.kHz(),
                duty_cycle: DutyCycle::Ratio2to1,
            },
            clocks,
            1000,
            10,
            1000,
            1000,
        );

        //-------------------------------------------------------------------------
        //                        rtic initialization
        //-------------------------------------------------------------------------
        let mut display: GraphicsMode<_> = Builder::new().connect_i2c(i2c).into();
        display.init().ok();
        display.flush().ok();
        let systick = cx.core.SYST;
        let mono = Systick::new(systick, 8_000_000);

        let button_up_pin = gpioa.pa5.into_pull_up_input(&mut gpioa.crl);
        let button_down_pin = gpioa.pa6.into_pull_up_input(&mut gpioa.crl);
        let button_enter_pin = gpioa.pa7.into_pull_up_input(&mut gpioa.crl);

        // NOTE(elsuizo:2021-11-24): here we dont need a super fast spawn!!!
        // react::spawn_after(1.secs()).unwrap();

        react::spawn_after(Duration::<u64, 1, 1000>::from_ticks(1000)).unwrap();

        (
            Shared { led },
            Local {
                button_up: Button::new(button_up_pin),
                button_down: Button::new(button_down_pin),
                button_enter: Button::new(button_enter_pin),
                display,
                logger,
                menu_fsm: crate::ui::MenuFSM::init(crate::ui::MenuState::Row1(false)),
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
    #[task(local = [button_up, button_down, button_enter])]
    fn react(cx: react::Context) {
        use crate::buttons::PinState::*;
        use crate::ui::Msg::*;

        if let PinUp = cx.local.button_up.poll() {
            dispatch_msg::spawn(Up).ok();
        }
        if let PinUp = cx.local.button_down.poll() {
            dispatch_msg::spawn(Down).ok();
        }
        if let PinUp = cx.local.button_enter.poll() {
            dispatch_msg::spawn(Enter).ok();
        }
        react::spawn_after(Duration::<u64, 1, 1000>::from_ticks(1000)).unwrap();
    }

    #[task(local = [display, logger, menu_fsm], shared = [led])]
    fn dispatch_msg(cx: dispatch_msg::Context, msg: crate::ui::Msg) {
        use crate::ui::Msg::*;
        let dispatch_msg::SharedResources { mut led } = cx.shared;
        cx.local.display.clear();
        cx.local.menu_fsm.next_state(msg);
        match msg {
            Up => {
                led.lock(|l| l.toggle());
                cx.local.logger.log("button Up pressed!!!").ok();
                crate::ui::draw_menu(cx.local.display, cx.local.menu_fsm.state).ok();
                cx.local.display.flush().unwrap();
            }
            Down => {
                led.lock(|l| l.toggle());
                cx.local.logger.log("button Down pressed!!!").ok();
                crate::ui::draw_menu(cx.local.display, cx.local.menu_fsm.state).ok();
                cx.local.display.flush().unwrap();
            }
            Enter => {
                led.lock(|l| l.toggle());
                cx.local.logger.log("button Enter pressed!!!").ok();
                crate::ui::draw_menu(cx.local.display, cx.local.menu_fsm.state).ok();
                cx.local.display.flush().unwrap();
            }
        };
    }
}
