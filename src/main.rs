#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use stm32f1xx_hal as hal;
use hal::{pac, pac::DWT, pac::DCB, delay::Delay, prelude::*};

use rtt_target::{rprintln, rtt_init_print};

#[entry]
fn main() -> ! {
    use chess2::board::board_representation;
    use chess2::algorithm;

    // Init buffers for debug printing
    rtt_init_print!();

    // Get access to device and core peripherals
    let dp = pac::Peripherals::take().unwrap();
    let mut cp = cortex_m::Peripherals::take().unwrap();

    // Get access to RCC, FLASH, AFIO, and GPIO
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let mut afio = dp.AFIO.constrain(&mut rcc.apb2);
    let mut gpioa = dp.GPIOA.split(&mut rcc.apb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.apb2);

    // Disable jtag so pa15, pb3, and pb4 can be used
    // These pins have to be set like so: pb7.into_push_pull_output(&mut gpiob.crl);
    let (pa15, pb3, pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);

    // Initialize led pin
    // When initializing pins use crl register for pins 0-7 and crh for pins 8-15
    let mut led = gpiob.pb7.into_push_pull_output(&mut gpiob.crl).downgrade();

    // Configure and apply clock configuration
    let clock_mhz = 48;
    let clocks = rcc.cfgr
        // External oscillator
        .use_hse(16.mhz())

        // Bus and core clocks
        .hclk(clock_mhz.mhz())
        .sysclk(clock_mhz.mhz())

        // Peripheral clocks
        .pclk1(12.mhz())
        .pclk2(12.mhz())
    .freeze(&mut flash.acr);

    // Set up systick delay
    let mut delay = Delay::new(cp.SYST, clocks);

    // Enable cycle counter
    cp.DCB.enable_trace();
    cp.DWT.enable_cycle_counter();

    // Initiliaze board to starting board
    let board = board_representation::Board {
        board: [71776119061217280, 9295429630892703744, 4755801206503243776, 2594073385365405696, 576460752303423488, 1152921504606846976, 65280, 129, 66, 36, 8, 16, 7926616819148718190],
        whites_move:true,
        points: board_representation::Points { white_points: 0, black_points: 0 },
        points_delta: 0,
        half_moves: 0,
        half_move_clock: 0,
        en_passant_target: None
    };

    let pieces_info = chess2::piece::constants::gen();

    let best_move = algorithm::gen_best_move(
        true,
        &DWT::cycle_count(),
        &chess2::embedded::ms_to_clocks(1000, clock_mhz),
        6,
        0,
        0,
        algorithm::AlphaBeta::new(),
        &[[0i16; 64]; 12],
        board,
        &pieces_info,
    );

    rprintln!("{:?}", best_move);

    // Blink led
    loop {
        led.set_high().ok();
        rprintln!("Led on");
        delay.delay_ms(1000u16);

        led.set_low().ok();
        rprintln!("Led off");
        delay.delay_ms(1000u16);
    }
}
