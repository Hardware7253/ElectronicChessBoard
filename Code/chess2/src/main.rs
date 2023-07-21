#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::digital::v2::InputPin;
use stm32f1xx_hal as hal;
use hal::{pac, pac::DWT, pac::DCB, delay::Delay, prelude::*};

use arrform::{arrform, ArrForm};

use rtt_target::{rprintln, rtt_init_print};


use chess2::board::board_representation;
use chess2::algorithm;
use chess2::embedded;

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
        press_raw: false,
        press_start_cycle: None,
        long_press_cycles: embedded::ms_to_cycles(650, clock_mhz as u64), // Button needs to be held for atleast 650ms for a long press
        long_press: false,
        last_press_cycle: 0,
        debounce_cycles: embedded::ms_to_cycles(10, clock_mhz as u64), // 10ms debounce
        consecutive_cycles: embedded::ms_to_cycles(150, clock_mhz as u64), // When button presses are registered less than 160ms apart then the presses are sequential
        c_presses: 0,
        consecutive_presses: 0, 
    };

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

    let pieces_info = chess2::piece::constants::gen(); // Generate piece info

    let max_search_times: [u64; 8] = [1000, 3000, 5000, 10000, 20000, 30000, 50000, 100000]; // Options for maximum search times (ms) for the minimax algorithm
    let mut search_time_index: usize = 2; // Index for the currently selected minimax search time
    let max_search_depth = 6; // Maximum minimax search depth

    let mut opening_heatmap = [[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 10, 1, 18, 10, 9, 9, 1, 0, 1, 33, 61, 475, 338, 22, 6, 5, 51, 142, 1144, 2288, 2246, 392, 88, 80, 88, 74, 361, 111, 276, 124, 322, 62, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 1, 0, 4, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 35, 32, 94, 499, 3, 0], [1, 0, 0, 0, 0, 0, 0, 2, 0, 0, 2, 0, 0, 19, 0, 2, 0, 0, 15, 1, 2, 7, 0, 0, 1, 31, 0, 19, 145, 2, 79, 0, 9, 0, 11, 268, 58, 0, 1, 7, 16, 17, 1470, 1, 3, 2054, 9, 15, 0, 0, 2, 115, 62, 1, 0, 0, 0, 1, 0, 0, 5, 2, 2, 0], [1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 3, 20, 22, 1, 0, 0, 1, 35, 0, 0, 17, 0, 2, 0, 314, 1, 13, 2, 0, 292, 0, 139, 2, 509, 2, 0, 47, 0, 35, 6, 108, 1, 162, 124, 1, 2, 3, 0, 51, 19, 57, 148, 1, 205, 0, 1, 0, 2, 0, 0, 3, 0, 0], [0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 3, 2, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 4, 1, 2, 0, 24, 22, 0, 13, 32, 3, 2, 24, 3, 0, 48, 7, 17, 6, 42, 0, 0, 0, 0, 66, 49, 67, 3, 0, 0, 0, 1, 0, 3, 3, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 9, 4, 1, 0, 0, 0, 23, 4, 0, 26, 498, 6], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 348, 125, 418, 716, 867, 40, 525, 86, 17, 238, 834, 1360, 1326, 216, 134, 18, 0, 13, 174, 512, 190, 170, 68, 4, 1, 0, 34, 3, 4, 37, 4, 0, 0, 6, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0], [0, 8, 3, 3, 17, 458, 5, 0, 1, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0], [0, 13, 0, 2, 1, 1, 8, 0, 0, 4, 3, 219, 58, 2, 1, 0, 21, 32, 1057, 15, 1, 1874, 4, 29, 56, 0, 8, 130, 31, 3, 1, 10, 0, 9, 4, 40, 190, 2, 21, 0, 0, 0, 31, 0, 2, 1, 3, 0, 0, 0, 1, 2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0], [0, 0, 0, 0, 1, 3, 0, 1, 1, 74, 0, 44, 307, 0, 387, 2, 20, 31, 2, 44, 56, 5, 9, 5, 27, 0, 241, 0, 2, 79, 3, 1, 0, 297, 3, 5, 2, 0, 98, 4, 0, 0, 60, 3, 1, 8, 0, 3, 0, 0, 0, 5, 1, 3, 1, 1, 0, 1, 0, 1, 0, 3, 0, 0], [1, 1, 2, 4, 5, 0, 0, 0, 0, 0, 36, 10, 62, 0, 0, 0, 0, 28, 0, 10, 2, 36, 6, 0, 79, 0, 0, 53, 5, 4, 12, 2, 0, 1, 1, 9, 3, 2, 0, 51, 1, 0, 1, 0, 0, 0, 0, 2, 0, 2, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0], [0, 0, 2, 7, 0, 4, 458, 0, 0, 0, 0, 0, 5, 17, 2, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]];

    // Testing how many clock cycles it takes for the computer the generate a move from a starting board position at a search depth of 4
    /*
    cycle_counter.update();
    let start_cycles = cycle_counter.cycles;

    algorithm::gen_best_move(
        true,
        &mut cycle_counter,
        &start_cycles,
        &chess2::embedded::ms_to_cycles(100000, clock_mhz as u64),
        4,
        0,
        0,
        algorithm::AlphaBeta::new(),
        &opening_heatmap,
        starting_board,
        &pieces_info,
    );

    cycle_counter.update();
    let end_cycles = cycle_counter.cycles;
    let elapsed_cycles = end_cycles - start_cycles;

    rprintln!("Computer move took {} clock cycles", elapsed_cycles);
    rprintln!("(Rougly {} seconds)", elapsed_cycles / (clock_mhz as u64 * 1000000));
    */

    loop {
        delay.delay_ms(1u16);
        lcd.clear(&mut delay);

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
                    embedded::leds_from_bitboard(&mut grid_sr, &mut delay, expected_board ^ physical_bitboard, 30000);
                }

                delay.delay_ms(1u16);
            }
        }

        // Initialise board
        let mut board = starting_board;

        // Game loop
        // Each loop represents one turn
        // The loop will break once the game has finished
        'game: loop {
            lcd.clear(&mut delay);

            let players_turn = player_white == board.whites_move; // Determine wether the current turn is for the player or computer to make

            // Piece move for the chess engine and the physical board must be stored seperately
            // Because the physical board has a dynamic orientation for the teams, while the internal engine board representation has a static orientation for the white and black team perspective
            let mut piece_internal_move = algorithm::Move::new();
            let mut piece_physical_move = algorithm::Move::new();

            physical_bitboard = embedded::read_board_halls(&mut grid_sr, &hall_sensor, &mut delay); // Get bitboard of pieces on the physical board before a piece has been moved
            let mut physical_bitboard_pm: u64 = 0; // Bitboard of pieces on the physical board after a piece has been moved

            // Get move from player / computer
            if players_turn {

                let mut init_physical_bitboard = physical_bitboard;

                // Loop until the player has made a proper move
                let mut piece_removed = false;
                loop {
                    lcd.set_cursor(&mut delay, [0, 0]);
                    lcd.print(&mut delay, "Players turn");
                    lcd.set_cursor(&mut delay, [0, 1]);
                    lcd_print_team(&mut lcd, &mut delay, player_white);

                    let new_physical_bitboard = embedded::read_board_halls(&mut grid_sr, &hall_sensor, &mut delay); // Get bitboard of pieces on the physical board

                    let piece_change = chess2::find_piece_change(physical_bitboard, new_physical_bitboard);

                    // Keep track of pieces being removed from the board so pieces can be captured without throwing an error
                    if piece_change == -1 {
                        if !piece_removed {
                            // Set the initial bitboard to the current bitboard if one piece is removed from the board
                            // This allows the capture move to be detected
                            // Only the first piece that is removed from the board can be captured
                            init_physical_bitboard = new_physical_bitboard;
                            piece_removed = true;
                        }
                    } else if piece_change > -1 {
                        init_physical_bitboard = physical_bitboard;
                    }

                    let button_pressed = button.press(&mut cycle_counter);

                    // When the button registers a long press open a menu to change the maximum time that the computer takes to search
                    if button.long_press {
                        lcd.clear(&mut delay);

                        let mut increment_queued = false;
                        let mut press_start_cycle: Option<u64> = None;
                        loop {
                            lcd.set_cursor(&mut delay, [0, 0]);
                            lcd.print(&mut delay, "Engine search ms");
                            lcd.set_cursor(&mut delay, [0, 1]);
                    
                            let af = arrform!(64, "{}", max_search_times[search_time_index]);
                            lcd.print(&mut delay, af.as_str());

                            let button_pressed = button.press(&mut cycle_counter);

                            // Long press the button again to close the menu
                            if button.long_press {
                                break;
                            }

                            // If the button is pressed increment the search_time_index
                            if button_pressed {
                                increment_queued = true;
                                press_start_cycle = button.press_start_cycle;
                            }

                            // Queue increment of the search time until after the button has been released
                            // This avoids the value updating while the user is trying to long press to exit the menu
                            if increment_queued && press_start_cycle != button.press_start_cycle {
                                lcd.clear(&mut delay);
                                search_time_index += 1;
                                if search_time_index > (max_search_times.len() - 1) {
                                    search_time_index = 0;
                                }
                                increment_queued = false;
                            }
                        }
                        button.consecutive_presses = 0; // Reset consecutive presses that may have been made while the user cycles through the search times
                        lcd.clear(&mut delay);
                    }

                    // When the button is pressed greater than 9 times consecutevily resign
                    if button.consecutive_presses > 9 {
                        break 'game;
                    }

                    if button_pressed {

                        // Do nothing if the board has not changed
                        if physical_bitboard == new_physical_bitboard {
                            continue;
                        }
                        
                        let player_move = chess2::find_bitboard_move(init_physical_bitboard, new_physical_bitboard, &board);

                        button.press(&mut cycle_counter);

                        match player_move {

                            // If the move was ok break the loop
                            Ok(player_move) => {
                                piece_physical_move = player_move;
                                physical_bitboard_pm = new_physical_bitboard;
                                break;
                            },

                            // If there was an error with the move make the player revert the move so they can try again
                            Err(_) => {
                                lcd.clear(&mut delay);
                                lcd.set_cursor(&mut delay, [0, 0]);
                                lcd.print(&mut delay, "Invalid move");
                                lcd.set_cursor(&mut delay, [0, 1]);
                                lcd.print(&mut delay, "Please revert");

                                show_bitboard_move(physical_bitboard, &mut grid_sr, &hall_sensor, &mut button, &mut cycle_counter, &mut delay);
                                lcd.clear(&mut delay);
                                piece_removed = false;
                                button.press(&mut cycle_counter);
                            },
                        }                        
                    }
                }
            } else {
                lcd.set_cursor(&mut delay, [0, 0]);
                lcd.print(&mut delay, "Computers turn");
                lcd.set_cursor(&mut delay, [0, 1]);
                lcd_print_team(&mut lcd, &mut delay, !player_white);

                cycle_counter.update();
                let start_cycles = cycle_counter.cycles;

                // Generate a move which takes no longer than max_search_times[search_time_index] and has a maximum search depth of max_search_depth
                piece_internal_move = algorithm::gen_best_move(
                    true,
                    &mut cycle_counter,
                    &start_cycles,
                    &chess2::embedded::ms_to_cycles(max_search_times[search_time_index], clock_mhz as u64),
                    max_search_depth,
                    0,
                    0,
                    algorithm::AlphaBeta::new(),
                    &opening_heatmap,
                    board,
                    &pieces_info,
                ).piece_move.unwrap();
            }

            // Set piece_internal / piece_physical move (whichever hasn't been updated yet)
            if player_white {

                // When the player is white the physical and internal boards are the same orientation so nothing needs to be flipped
                if players_turn {
                    piece_internal_move = piece_physical_move;
                } else {
                    piece_physical_move = piece_internal_move;
                }
            } else {

                // When the player is black the physical and internal boards are opposite orientations so the moves have to flipped
                if players_turn {
                    piece_internal_move = piece_physical_move.flip();
                    piece_internal_move.initial_piece_coordinates.board_index = chess2::find_board_index(&board, piece_internal_move.initial_piece_coordinates.bit).unwrap(); // Update board index
                } else {
                    piece_physical_move = piece_internal_move.flip();
                } 
            }

            // Get friendly and enemy kings
            let friendly_king_index;
            let enemy_king_index;
            if board.whites_move {
                friendly_king_index = 5;
                enemy_king_index = 11;
            } else {
                friendly_king_index = 11;
                enemy_king_index = 5;
            }

            let friendly_king = board_representation::BoardCoordinates {
                board_index: friendly_king_index,
                bit: chess2::find_bit_on(board.board[friendly_king_index], 0),
            };

            let enemy_king = board_representation::BoardCoordinates {
                board_index: enemy_king_index,
                bit: chess2::find_bit_on(board.board[enemy_king_index], 0),
            };

            use chess2::board::move_generator;
            use move_generator::TurnError;

            // Get new board after turn has been made
            let team_bitboards = chess2::TeamBitboards::new(friendly_king.board_index, &board);
            let enemy_attacks = move_generator::gen_enemy_attacks(&friendly_king, team_bitboards, &board, &pieces_info);
            let new_turn_board = move_generator::new_turn(&piece_internal_move.initial_piece_coordinates, piece_internal_move.final_piece_bit, friendly_king, &enemy_king, &enemy_attacks, team_bitboards, board, &pieces_info);

            match new_turn_board {
                Ok(new_board) => {

                    // Get what the phsysical bitboard should be after the turn is made
                    let mut new_physical_bitboard = new_board.to_bitboard();

                    if !player_white {
                        new_physical_bitboard = chess2::flip_bitboard(new_physical_bitboard); // Flip the bitboard to physical board perspective
                    }

                    // Show computer move
                    if !players_turn {
                        show_move(new_physical_bitboard, &piece_physical_move, &mut grid_sr, &hall_sensor, &mut button, &mut cycle_counter, &mut delay)
                    }

                    board = new_board;
                },
                Err(error) => {
                    lcd.clear(&mut delay);
                    lcd.home(&mut delay);
                    
                    match error {

                        // When there is a win error break the game loop so a new game can be started
                        TurnError::Win => {

                            // Get what the phsysical bitboard should be after the turn is made
                            let mut new_physical_bitboard = board.to_bitboard();

                            new_physical_bitboard ^= 1 << piece_internal_move.initial_piece_coordinates.bit; // Toggle initial piece bit

                            if !chess2::bit_on(new_physical_bitboard, piece_internal_move.final_piece_bit) {
                                new_physical_bitboard ^= 1 << piece_internal_move.final_piece_bit; // Toggle the final piece bit if there wasn't a capture
                            }

                            if !player_white {
                                new_physical_bitboard = chess2::flip_bitboard(new_physical_bitboard); // Flip the bitboard to physical board perspective
                            }

                            // Show computer move
                            if !players_turn {
                                show_move(new_physical_bitboard, &piece_physical_move, &mut grid_sr, &hall_sensor, &mut button, &mut cycle_counter, &mut delay)
                            }

                            // Print the winning team to the lcd
                            lcd.print(&mut delay, "Game over");
                            lcd.set_cursor(&mut delay, [0, 1]);
                            lcd_print_team(&mut lcd, &mut delay, board.whites_move);
                            lcd.print(&mut delay, " team wins");
                            break 'game;
                        },
                        TurnError::Draw => {
                            lcd.print(&mut delay, "Game over (draw)");
                            break 'game;
                        },

                        // When there is an invalid move error make the player revert the turn and try again
                        TurnError::InvalidMove => {
                            lcd.print(&mut delay, "Invalid move");
                            lcd.set_cursor(&mut delay, [0, 1]);
                            lcd.print(&mut delay, "Please revert");

                            show_bitboard_move(physical_bitboard, &mut grid_sr, &hall_sensor, &mut button, &mut cycle_counter, &mut delay);
                            button.press(&mut cycle_counter);
                            continue;
                        },
                        TurnError::InvalidMoveCheck => {
                            lcd.print(&mut delay, "King in check");
                            lcd.set_cursor(&mut delay, [0, 1]);
                            lcd.print(&mut delay, "Please revert");

                            show_bitboard_move(physical_bitboard, &mut grid_sr, &hall_sensor, &mut button, &mut cycle_counter, &mut delay);
                            button.press(&mut cycle_counter);
                            continue;
                        },
                    }
                },
            }

            // Draw game based on half move clock after the move has taken place
            // This is so checkmates made this move take priority over the half move draw
            if board.half_move_clock >= 100 {
                lcd.clear(&mut delay);
                lcd.set_cursor(&mut delay, [0, 0]);
                lcd.print(&mut delay, "Game over (draw)");
                lcd.set_cursor(&mut delay, [0, 1]);
                lcd.print(&mut delay, "Fifty move rule");
                break 'game;
            }

            // Once the early and mid phases of the game are done reset the opening heatmap
            // After this point no heatmap will affect the computer moves
            if board.half_moves > 20 {
                opening_heatmap = [[0i16; 64]; 12];
            }

            button.press(&mut cycle_counter);
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

// Only exits once the physical bitboard equals the desired bitboard
// Lights leds to show the user what pieces they need to move to do this
fn show_bitboard_move<T: InputPin>(desired_bitboard: u64, grid_sr: &mut embedded::ShiftRegister, hall_sensor: &T, button: &mut embedded::button::Button, cycle_counter: &mut embedded::cycle_counter::Counter, delay: &mut Delay) {
    
    let mut current_bitboard = embedded::read_board_halls(grid_sr, hall_sensor, delay); // Get bitboard of pieces on the physical board
    while current_bitboard != desired_bitboard {
        current_bitboard = embedded::read_board_halls(grid_sr, hall_sensor, delay);
        embedded::leds_from_bitboard(grid_sr, delay, desired_bitboard ^ current_bitboard, 3000);
    }
}

// Only exits once the piece_physical move has been made on the board
fn show_move<T: InputPin>(desired_bitboard: u64, piece_physical_move: &chess2::algorithm::Move, grid_sr: &mut embedded::ShiftRegister, hall_sensor: &T, button: &mut embedded::button::Button, cycle_counter: &mut embedded::cycle_counter::Counter, delay: &mut Delay) {
    let current_bitboard = embedded::read_board_halls(grid_sr, hall_sensor, delay); // Get bitboard of pieces on the physical board

    // If the bit where the piece has to move is allready on then it is performing a capture
    // When this happens make the player remove the capture piece first
    if chess2::bit_on(current_bitboard, piece_physical_move.final_piece_bit) {
        show_bitboard_move(desired_bitboard ^ 1 << piece_physical_move.final_piece_bit, grid_sr, hall_sensor, button, cycle_counter, delay);
    }

    show_bitboard_move(desired_bitboard, grid_sr, hall_sensor, button, cycle_counter, delay);
}
