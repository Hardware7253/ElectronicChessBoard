#![no_std]
#![no_main]

use core::convert::TryFrom;
use core::convert::TryInto;

use core::result::Result::Ok;
use core::result::Result::Err;

pub mod board;
pub mod piece;
pub mod algorithm;

// Convert a char of a number to an integer
// E.g. '1' -> 1
// Offset offsets the ascii value
pub fn char_to_num(c: char, offset: i8) -> Result<i8, ()> {
    let num = c as i8 - {48 + offset};
    if num < 0 || num > 9 {
        return Err(())
    }
    Ok(num)
}

pub fn num_to_char(num: usize) -> Result<char, ()> {
    if num > 9 {
        return Err(());
    }
    let num = u8::try_from(num).unwrap();
    let char_num = (num + 48) as char;
    Ok(char_num)
}

pub fn bit_to_ccn(bit: usize) -> &'static str {
    let ccn_array: [&str; 64] = [
        "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
        "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
        "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
        "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
        "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
        "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
        "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
        "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1",
    ];
    ccn_array[bit]
}

// Converts a bit number (e.g. bit 7 in a u64) to a cartesian coordinates on the board
fn bit_to_cartesian(bit: i8) -> [i8; 2] {
    [bit % 8, bit / 8]
}

// Checks if moving piece from inital by delta bit causes it to go outside of the board space in cartesian coordinates
pub fn bit_move_valid(initial_bit: usize, delta_bit: i8) -> bool {
    let initial_bit: i8 = initial_bit.try_into().unwrap();
    let initial_coordinates = bit_to_cartesian(initial_bit);

    let control_coordinates = bit_to_cartesian(27); // Cordinates that every piece can perform a move from, without going out of bounds
    let control_move_coordinates = bit_to_cartesian(27 + delta_bit);

    // Get coordinates of piece at initial bit after moving to delta bit
    let control_move_delta = [control_move_coordinates[0] - control_coordinates[0], control_move_coordinates[1] - control_coordinates[1]];
    let new_coordinates = [initial_coordinates[0] + control_move_delta[0], initial_coordinates[1] + control_move_delta[1]];

    // Return false if the new_coordinates are out of bounds
    if new_coordinates[0] < 0 || new_coordinates[0] > 7 || new_coordinates[1] < 0 || new_coordinates[1] > 7 {
        return false;
    }

    true
}

// Ors a group of bitboards in a board array
pub fn or_bitboards(from: usize, to: usize, board: &[u64; 13]) -> u64 {
    let mut bitboard = 0;
    for i in from..to + 1 {
        bitboard ^= board[i];
    }
    bitboard
}

// Return true if a board index corresponds to the white team
pub fn board_index_white(index: usize) -> bool {
    if index > 5 {
        return false;
    }
    true
}

// Returns true if a bit is on in a u64 number
pub fn bit_on(num: u64, bit: usize) -> bool {
    let num_from_bit = 1 << bit;
    if num ^ num_from_bit < num {
        return true;
    }
    false
}

// Returns the number of bits on in the given number
pub fn bits_on(num: u64) -> usize {
    let mut bits_on = 0;
    for i in 0..64 {
        if bit_on(num, i) {
            bits_on += 1;
        }
    }
    bits_on
}

// Function returns when it finds what bit is on in a u64 number
// E.g. 8 would return 3
// If no bits are on in the number then default will be returned
pub fn find_bit_on(num: u64, default: usize) -> usize {
    for i in 0..64 {
        if bit_on(num, i) {
            return i;
        }
    }
    default
}

// Finds the bitboard index for a piece at a given bit
pub fn find_board_index(board: &board::board_representation::Board, bit: usize) -> Result<usize, ()> {
    for i in 0..board.board.len() {
        if bit_on(board.board[i], bit) {
            return Ok(i);
        }
    }
    
    Err(())
}

// Return true if 2 numbers have a bit in common
pub fn common_bit(num1: u64, num2: u64) -> bool {
    let xor_nums = num1 ^ num2;
    if xor_nums < num1 || xor_nums < num2 {
        return true;
    }
    false
}

// Flips bitboard bit to enemy team perspective
pub fn flip_bitboard_bit(bit: usize) -> usize {
    let flipped = bit as i8 - 63;
    flipped.abs().try_into().unwrap()
}


// Flip entire bitboard to enemy team persepctive
pub fn flip_bitboard(bitboard: u64) -> u64 {

    let mut flipped_bitboard = 0;
    for bit in 0..64 {
        let flipped_bit = flip_bitboard_bit(bit);

        if bit_on(bitboard, bit) {
            flipped_bitboard |= 1 << flipped_bit;
        }
    }

    flipped_bitboard
}

// Returns the number of pieces which have been added/removed from a bitboard
pub fn find_piece_change(init_bitboard: u64, final_bitboard: u64) -> i8 {
    bits_on(final_bitboard) as i8 - bits_on(init_bitboard) as i8
}

// Finds a piece that has moved given a final and initial bitboard
// Returns an error if more than one piece was moved
pub fn find_bitboard_move(init_bitboard: u64, final_bitboard: u64, init_board: &board::board_representation::Board) -> Result<algorithm::Move, ()> {
    
    // Return an error if pieces were removed from the board
    if bits_on(init_bitboard) != bits_on(final_bitboard) {
        return Err(());
    }

    let change_bitboard = init_bitboard ^ final_bitboard; // Bitboard of the bits that have changed from the initial to final bitboard

    let mut changed_bits = 0; // Stores the number of bits that have been changed from the inital to final bitboard

    let mut piece_move = algorithm::Move::new();

    for i in 0..64 {
        if bit_on(change_bitboard, i) {
            changed_bits += 1;

            if bit_on(init_bitboard, i) {
                piece_move.initial_piece_coordinates.bit = i; // Get initial piece bit
            } else {
                piece_move.final_piece_bit = i; // Get final piece bit
            }
        }
    }
    
    // Return an error if multiple pieces moved, or no pieces moved
    if changed_bits != 2 {
        return Err(());
    }

    // Find the board index of the piece
    piece_move.initial_piece_coordinates.board_index = find_board_index(init_board, piece_move.initial_piece_coordinates.bit).unwrap();

    Ok(piece_move)
}

// A struct containing bitboards which have the locations of all pieces on the friendly and enemy team
#[derive(PartialEq, Debug, Copy, Clone)]
pub struct TeamBitboards {
    pub friendly_team: u64,
    pub enemy_team: u64,
}

impl TeamBitboards {
    // Generate team bitborads relative to index team
    pub fn new(index: usize, board: &crate::board::board_representation::Board) -> Self {
        let all_white_bitboard = or_bitboards(0, 5, &board.board);
        let all_black_bitboard = or_bitboards(6, 11, &board.board);
    
        let friendly_bitboard;
        let enemy_bitboard;
        if board_index_white(index) {
            friendly_bitboard = all_white_bitboard;
            enemy_bitboard = all_black_bitboard;
        } else {
            friendly_bitboard = all_black_bitboard;
            enemy_bitboard = all_white_bitboard;
        }
    
        TeamBitboards {
            friendly_team: friendly_bitboard,
            enemy_team: enemy_bitboard,
        }
    }
}

pub mod embedded {
    use super::*;

    use embedded_hal::digital::v2::{InputPin, OutputPin};
    use stm32f1xx_hal as hal;
    use hal::gpio::{Pxx, PushPull, Output, Input, PullDown};
    use hal::{pac, pac::DWT, delay::Delay, prelude::*};

    use rtt_target::{rprintln, rtt_init_print};

    // Struct for shift register pins
    pub struct ShiftRegister {
        pub clock: Pxx<Output<PushPull>>, // Shift register serial clock pin
        pub data: Pxx<Output<PushPull>>, // Shift register serial data pin
        pub latch: Pxx<Output<PushPull>>, // Shift register data latch pin
        pub bits: usize, // Shift register bits
    }

    impl ShiftRegister {
        pub fn init(&mut self, delay: &mut Delay) {
            self.clock.set_low().ok();
            self.data.set_low().ok();
            self.latch.set_low().ok();

            self.shift_out(delay, 0, true);
        }

        // Shifts given number into a shift register
        fn shift_out(&mut self, delay: &mut Delay, num: u64, msbfirst: bool) {
            for i in 0..self.bits {
                
                // Write bit
                if !msbfirst {
                    digital_write(&mut self.data, bit_on(num, i)); 
                } else {
                    digital_write(&mut self.data, bit_on(num, (i as i16 - (self.bits as i16 - 1)).abs().try_into().unwrap())); 
                }
                
                delay.delay_us(1u32); // Data hold time

                pulse_pin(&mut self.clock, delay, 1); // Bit is shifted into the shift register with a clock pulse
            }
            delay.delay_us(1u32); // Data hold time
            pulse_pin(&mut self.latch, delay, 1); // Latch data into internal output register
        }
    }

    // Converts milliseconds to clock cycles
    pub fn ms_to_cycles(millis: u64, clock_mhz: u64) -> u64 {
        millis * clock_mhz * 1000
    }

    // Writes low or high state to given pin
    pub fn digital_write(pin: &mut Pxx<Output<PushPull>>, high: bool) {
        if high {
            pin.set_high().ok();
            return;
        }
        pin.set_low().ok();
    }

    // Returns the state of the digital pin
    pub fn digital_read<T: InputPin>(pin: &T) -> bool {
        let state = pin.is_high();

        match state {
            Ok(high) => return high, // Return pin state
            Err(_) => return false, // If there was an error assume the pin is low
        }
    }

    // Sets a pin high, waits micro_seconds, sets the pin low, waits micro_seconds
    fn pulse_pin(pin: &mut Pxx<Output<PushPull>>, delay: &mut Delay, micro_seconds: u32) {
        pin.set_high().ok();
        delay.delay_us(micro_seconds);
        pin.set_low().ok();
        delay.delay_us(micro_seconds);
    }
    
    // Writes to the led/hall sensor grid shift registers
    // Led/hall can be selected using a bitboard bit
    pub fn write_grid(shift_register: &mut ShiftRegister, delay: &mut Delay, bit: usize, leds_on: bool) {
        let grid_coordinates = bit_to_cartesian(bit as i8);

        let mut shift_num: u64 = 0;
        shift_num += grid_coordinates[1] as u64; // The led y coordinate occupies bits 0,1,2 of the shift register
        shift_num += (grid_coordinates[0] as u64) << 3; // The led x coordinate occupies bits 3,4,5 of the shift register

        // The hall sensor x and y coordinates occupy the same bits but on the most significant shift register
        shift_num += shift_num << 8;

        // Set bit 7 high to disable leds
        if !leds_on {
            shift_num += 1 << 7;
        }

        shift_register.shift_out(delay, shift_num, true);
    }

    // Turns on leds on the board according to the given bitboard
    // Simulates turning multiple leds on simultaneously by turning them off and on in quick succesion
    pub fn leds_from_bitboard(shift_register: &mut ShiftRegister, delay: &mut Delay, bitboard: u64, led_on_time_us: u32) {
        for i in 0..64 {
            if bit_on(bitboard, i) {
                write_grid(shift_register, delay, i, true); // Turn led on
                delay.delay_us(led_on_time_us);
                write_grid(shift_register, delay, i, false); // Turn led off
            }
        }
    }

    // Reads all hall effect sensors on the board, and returns a bitboard
    pub fn read_board_halls<T: InputPin>(shift_register: &mut ShiftRegister, hall_sensor: &T, delay: &mut Delay) -> u64 {
        let mut bitboard = 0;
        
        for i in 0..64 {
            write_grid(shift_register, delay, i, false); // Select hall effect sensor to read
            let magnet_detected = !digital_read(hall_sensor); // Read hall effect sensor

            // If the hall effect sensor is detecting a magenetic field then turn it's bit on
            if magnet_detected {
                bitboard |= 1 << i;
            }
        }

        bitboard
    }

    pub mod cycle_counter {
        use super::*;

        #[derive(Debug)]
        pub struct Counter {
            pub cycles: u64, // Total cycle count
            pub cycle_resets: u32, // Number of times the DWT cycle count has rolled over
            pub last_cycle_count: u32, // Last DWT cycle count
        }

        impl Counter {
            pub fn new() -> Self {
                Counter {
                    cycles: 0,
                    cycle_resets: 0,
                    last_cycle_count: 0,
                }
            }

            pub fn update(&mut self) {
                let dwt_cycles = DWT::cycle_count();

                // When the DWT cycle count resets increment cycle_resets
                if dwt_cycles < self.last_cycle_count {
                    self.cycle_resets += 1; 
                }
                self.last_cycle_count = dwt_cycles;

                self.cycles = (self.cycle_resets * u32::MAX) as u64 + dwt_cycles as u64; // Update cycle count
            }
        }
    }

    pub mod button {
        use super::*;

        pub struct Button {
            pub pin: Pxx<Input<PullDown>>, // Button pin
            pub press_raw: bool,
            pub press_start_cycle: Option<u64>, // The clock cycle that the button has been held down since
            pub long_press_cycles: u64, // Cycles that must elapse between currenct clock cycle and press_start_cycle for a press to be considered a long press
            pub long_press: bool,
            pub last_press_cycle: u64, // Processor cycles elapsed when the button was last pressed
            pub debounce_cycles: u64, // Minimum number of processor cycles between button pressed
            pub consecutive_cycles: u64, // After this many cycles have elapsed between the last button press and current button press the press is no longer sequential
            pub c_presses: u8, // Number of presses that have been made in quick succesion, updated during the consecutive presses
            pub consecutive_presses: u8, // Number of presses that have been made in quick succesion, updated after the consecutive presses
        }

        impl Button {

            // Returns true only when the button is pressed, so true will not be returned when the button is held down or bouncing
            // This functionality is dependant on the buttons debounce_cycles
            pub fn press(&mut self, counter: &mut cycle_counter::Counter) -> bool {
                self.long_press = false;
                counter.update();

                let mut pressed = false;
                let pin_high = digital_read(&self.pin);
                self.press_raw = pin_high;
                if pin_high {

                    // Update press start cycle
                    match self.press_start_cycle {
                        Some(_) => (),
                        None => self.press_start_cycle = Some(counter.cycles),
                    }
                    
                    // Return true if the button has been pressed and isn't bouncing
                    if counter.cycles > (self.last_press_cycle + self.debounce_cycles) {
                        pressed = true;
                    }
                    self.last_press_cycle = counter.cycles;
                } else {

                    // If the button is released check for a long press
                    match self.press_start_cycle {
                        Some(cycle) => {
                            if (counter.cycles - cycle) >= self.long_press_cycles {
                                self.long_press = true;
                            }
                        },
                        None => (),
                    }
                    self.press_start_cycle = None; // When the button is not pressed there is no press start cycle
                }

                // Detect consecutive presses
                if (counter.cycles - self.last_press_cycle) < self.consecutive_cycles {
                    if pressed {
                        self.c_presses += 1;
                    }
                } else {
                    self.consecutive_presses = self.c_presses;
                    self.c_presses = 0;
                }

                pressed
            }
        }
    }

    pub mod character_lcd {
        use super::*;

        pub struct Lcd {
            pub shift_register: ShiftRegister, // Shift register connecting to character lcd
            pub register_select: Pxx<Output<PushPull>>, // Register select pin
        }

        // Instructions derived from various datasheets
        // https://www.sparkfun.com/datasheets/LCD/ADM1602K-NSW-FBS-3.3v.pdf
        // https://www.openhacks.com/uploadsproductos/eone-1602a1.pdf
        // Non 1602 lcds might have different instructions, and definitely different ddram addresses
        impl Lcd {

            // Writes a byte to a character lcd, data_input sets register select pin
            pub fn write(&mut self, delay: &mut Delay, data_input: bool, data: u8) {
                digital_write(&mut self.register_select, data_input); // Set data_input / instruction input
                
                self.shift_register.shift_out(delay, data as u64, true);

                delay.delay_us(700u32); // Ensure there is time inbetween character lcd writes
            }

            // Initialze character lcd
            pub fn init(&mut self, delay: &mut Delay) {
                self.shift_register.clock.set_low().ok();
                self.shift_register.latch.set_low().ok();

                self.write(delay, false, 0b00111000); // Initialize lcd with 8-bit bus, 2 lines, and 5x8 dot format

                self.clear(delay); // Clear dispaly
                self.home(delay);  // Home cursor
                self.power(delay, true, false, false); // Power on display, and hide the cursor
            }

            // Turn on/off display, cursor, and cursor position
            pub fn power(&mut self, delay: &mut Delay, display_on: bool, cursor_on: bool, cursor_position_on: bool) {
                let mut write_byte: u8 = 8;

                if display_on {
                    write_byte += 4;
                }

                if cursor_on {
                    write_byte += 2;
                }

                if cursor_position_on {
                    write_byte += 1;
                }

                self.write(delay, false, write_byte);
            }

            // Clear display
            pub fn clear(&mut self, delay: &mut Delay) {
                self.write(delay, false, 0b00000001);
            }

            // Home cursor
            pub fn home(&mut self, delay: &mut Delay) {
                self.write(delay, false, 0b00000010);
            }

            // Shift cursor/display once in the specified direction
            pub fn shift(&mut self, delay: &mut Delay, shift_display: bool, shift_right: bool) {
                let mut write_byte: u8 = 0b00010000;

                if shift_display {
                    write_byte += 0b00001000;
                }

                if shift_right {
                    write_byte += 0b00000100;
                }

                self.write(delay, false, write_byte);
            }

            // Sets ddram (cursor) address
            pub fn set_ddram(&mut self, delay: &mut Delay, ddram_address: u8) {
                let mut write_byte: u8 = 0b10000000;
                write_byte ^= ddram_address;

                self.write(delay, false, write_byte);
            }

            // Sets the cursor position with cartesian coordinates
            pub fn set_cursor(&mut self, delay: &mut Delay, new_position: [u8; 2]) {
                // Character lcds that aren't a 1602 will have different and more/less ddram addresses
                let ddram_addresses: [[u8; 16]; 2] = [
                    [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F],
                    [0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4A, 0x4B, 0x4C, 0x4D, 0x4E, 0x4F],
                ];
                
                let address = ddram_addresses[new_position[1] as usize][new_position[0] as usize];
                self.set_ddram(delay, address);
            }

            // Prints a string to the lcd
            pub fn print(&mut self, delay: &mut Delay, string: &str) {
                for c in string.chars() {
                    self.write(delay, true, c as u8);
                }
            }
        }
    }
}