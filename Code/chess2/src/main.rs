#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::digital::v2::InputPin;
use stm32f1xx_hal as hal;
use hal::{pac, pac::DWT, pac::DCB, delay::Delay, prelude::*};

use rtt_target::{rprintln, rtt_init_print};


use chess2::board::board_representation;
use chess2::algorithm;
use chess2::embedded;

const led_error_strobe_us: u32 = 30000; // How long to pulse each led during a board setup or piece move error

#[entry]
fn main() -> ! {
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

    // Configure and apply clock configuration
    let clock_mhz = 72;
    let clocks = rcc.cfgr
        // External oscillator
        .use_hse(8.mhz())

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

    let mut cycle_counter = embedded::cycle_counter::Counter::new();

    // Initialise hall and led grid shift register
    let mut grid_sr = embedded::ShiftRegister {
        clock: gpioa.pa3.into_push_pull_output(&mut gpioa.crl).downgrade(),
        data: gpioa.pa5.into_push_pull_output(&mut gpioa.crl).downgrade(),
        latch: gpioa.pa4.into_push_pull_output(&mut gpioa.crl).downgrade(),
        bits: 16,
    };
    grid_sr.init(&mut delay);
    embedded::write_grid(&mut grid_sr, &mut delay, 0, false); // Initialise grid with leds off

    // Initialise character lcd
    let mut lcd = embedded::character_lcd::Lcd {
        shift_register: embedded::ShiftRegister {
            clock: gpiob.pb1.into_push_pull_output(&mut gpiob.crl).downgrade(),
            data: gpioa.pa7.into_push_pull_output(&mut gpioa.crl).downgrade(),
            latch: gpiob.pb0.into_push_pull_output(&mut gpiob.crl).downgrade(),
            bits: 8,
        },
        register_select: gpiob.pb2.into_push_pull_output(&mut gpiob.crl).downgrade(),
    };
    lcd.init(&mut delay);

    let hall_sensor = gpiob.pb12.into_floating_input(&mut gpiob.crh).downgrade(); // Pin to read value of the selected hall sensor

    let mut button = embedded::button::Button {
        pin: gpiob.pb13.into_pull_down_input(&mut gpiob.crh).downgrade(),
        last_press_cycle: 0,
        debounce_cycles: embedded::ms_to_cycles(80, clock_mhz as u64), // 80ms debounce
        consecutive_cycles: embedded::ms_to_cycles(150, clock_mhz as u64), // When button presses are registered less than 200ms apart then the presses are sequential
        c_presses: 0,
        consecutive_presses: 0, 
    };

    // Turn on led and select hall sensor at bitboard bit 0
    //chess2::embedded::write_grid(&mut grid_sr, &mut delay, 0, true);

    // Initiliaze board to starting board
    let starting_board = board_representation::Board {
        board: [71776119061217280, 9295429630892703744, 4755801206503243776, 2594073385365405696, 576460752303423488, 1152921504606846976, 65280, 129, 66, 36, 8, 16, 7926616819148718190],
        whites_move: true,
        points: board_representation::Points { white_points: 0, black_points: 0 },
        points_delta: 0,
        half_moves: 0,
        half_move_clock: 0,
        en_passant_target: None
    };

    let pieces_info = chess2::piece::constants::gen();

    /*
    let best_move = algorithm::gen_best_move(
        true,
        &DWT::cycle_count(),
        &chess2::embedded::ms_to_cycles(1000, clock_mhz),
        6,
        0,
        0,
        algorithm::AlphaBeta::new(),
        &[[0i16; 64]; 12],
        board,
        &pieces_info,
    );
    */

    

    loop {
        delay.delay_ms(1u16);

        // Get player team
        let mut player_white = true;
        {
            let mut game_started = false;
            

            let mut team_message_start_cycle = 0; // The clock cycle the current team select message started getting displayed at
            let team_message_cycles = embedded::ms_to_cycles(1000, clock_mhz as u64); // How many clock cycles the game start message should be displayed for before switching to the oposite team

            while !game_started {

                // Display start game message for white and black
                lcd.set_cursor(&mut delay, [0, 0]);
                if player_white {
                    lcd.print(&mut delay, "Start as white?");
                } else {
                    lcd.print(&mut delay, "Start as black?");                
                }
                lcd.set_cursor(&mut delay, [0, 1]);
                lcd.print(&mut delay, "(Press button)");


                if button.press(&mut cycle_counter) {
                    game_started = true;
                }

                if cycle_counter.cycles > team_message_start_cycle + team_message_cycles {
                    team_message_start_cycle = cycle_counter.cycles;
                    player_white = !player_white;
                }
            }
        }

        lcd.clear(&mut delay);

        // Ensure the physical board is set up properly
        let mut physical_bitboard = embedded::read_board_halls(&mut grid_sr, &hall_sensor, &mut delay); // Get bitboard of pieces on the physical board
        {
            let expected_board: u64 = 0b1111111111111111000000000000000000000000000000001111111111111111;
            
            while physical_bitboard != expected_board {
                lcd.set_cursor(&mut delay, [0, 0]);
                lcd.print(&mut delay, "Please setup");
                lcd.set_cursor(&mut delay, [0, 1]);
                lcd.print(&mut delay, "the board");

                physical_bitboard = embedded::read_board_halls(&mut grid_sr, &hall_sensor, &mut delay); // Update physical bitboard

                // If the button is pressed highlight the positions where pieces have to placed
                if button.press(&mut cycle_counter) {
                    embedded::leds_from_bitboard(&mut grid_sr, &mut delay, expected_board ^ physical_bitboard, led_error_strobe_us);
                }

                delay.delay_ms(1u16);
            }
        }

        lcd.clear(&mut delay);

        // Initialise board
        let mut board = starting_board;

        // Game loop
        // Each loop represents one turn
        // The loop will break once the game has finished
        loop {
            let players_turn = player_white == board.whites_move; // Determine wether the current turn is for the player or computer to make

            let mut piece_move = chess2::PieceMove::new();

            // Get move from player / computer
            if players_turn {

                // Loop until the player has made a proper move
                loop {
                    lcd.set_cursor(&mut delay, [0, 0]);
                    lcd.print(&mut delay, "Players turn");
                    lcd.set_cursor(&mut delay, [0, 1]);
                    lcd_print_team(&mut lcd, &mut delay, player_white);

                    if button.press(&mut cycle_counter) {
                        let new_physical_bitboard = embedded::read_board_halls(&mut grid_sr, &hall_sensor, &mut delay); // Get bitboard of pieces on the physical board

                        let player_move = chess2::find_bitboard_move(physical_bitboard, new_physical_bitboard, &board);

                        match player_move {

                            // If the move was ok break the loop
                            Ok(player_move) => {
                                piece_move = player_move;
                                break;
                            },

                            // If there was an error with the move make the player revert the move so they can try again
                            Err(_) => {
                                lcd.clear(&mut delay);
                                lcd.set_cursor(&mut delay, [0, 0]);
                                lcd.print(&mut delay, "Invalid move");
                                lcd.set_cursor(&mut delay, [0, 1]);
                                lcd.print(&mut delay, "Please revert");

                                move_error(physical_bitboard, new_physical_bitboard, &mut grid_sr, &hall_sensor, &mut button, &mut cycle_counter, &mut delay);
                                lcd.clear(&mut delay);
                            },
                        }                        
                    }
                }
            } else {
                lcd.set_cursor(&mut delay, [0, 0]);
                lcd.print(&mut delay, "Computers turn");
                lcd.set_cursor(&mut delay, [0, 1]);
                lcd_print_team(&mut lcd, &mut delay, !player_white);
            }

            

            lcd.clear(&mut delay);
        }
    }
}

// Prints team (white / black) to lcd
fn lcd_print_team(lcd: &mut chess2::embedded::character_lcd::Lcd, delay: &mut Delay, team_white: bool) {
    if team_white {
        lcd.print(delay, "(White)");
    } else {
        lcd.print(delay, "(Black)");
    }
}

// Returns once pieces have been reverted back to their proper position after a player has made a turn error
fn move_error<T: InputPin>(desired_bitboard: u64, mut current_bitboard: u64, grid_sr: &mut embedded::ShiftRegister, hall_sensor: &T, button: &mut embedded::button::Button, cycle_counter: &mut embedded::cycle_counter::Counter, delay: &mut Delay) {
    while current_bitboard != desired_bitboard {
        current_bitboard = embedded::read_board_halls(grid_sr, hall_sensor, delay); // Get bitboard of pieces on the physical board

        // When the button is pressed toggle leds where pieces should / shouldn't be to help player fix the board
        if button.press(cycle_counter) {
            embedded::leds_from_bitboard(grid_sr, delay, desired_bitboard ^ current_bitboard, led_error_strobe_us);
        }
    }
}