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
    use hal::{pac, delay::Delay, prelude::*};

    // Struct for shift register pins
    pub struct ShiftRegister {
        pub clock: Pxx<Output<PushPull>>, // Shift register serial clock pin
        pub data: Pxx<Output<PushPull>>, // Shift register serial data pin
        pub latch: Pxx<Output<PushPull>>, // Shift register data latch pin
        pub bits: usize, // Shift register bits
    }

    impl ShiftRegister {
        pub fn init(&mut self) {
            self.clock.set_low().ok();
            self.data.set_low().ok();
            self.latch.set_low().ok();
        }
    }

    // Converts milliseconds to cpu clocks
    pub fn ms_to_clocks(millis: u32, clock_mhz: u32) -> u32 {
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

    // Returns true if the given digital pin is high
    pub fn pin_high(pin: &mut Pxx<Input<PullDown>>) -> bool {
        if pin.is_high().unwrap() {
            return true;
        }
        false
    }
    
    // Shifts given number into a shift register
    pub fn shift_out(shift_register: &mut ShiftRegister, delay: &mut Delay, num: u64, msbfirst: bool) {
        for i in 0..shift_register.bits {
            
            // Write bit
            if !msbfirst {
                digital_write(&mut shift_register.data, bit_on(num, i)); 
            } else {
                digital_write(&mut shift_register.data, bit_on(num, (i as i16 - (shift_register.bits as i16 - 1)).abs().try_into().unwrap())); 
            }
            
            delay.delay_us(1u32); // Data hold time

            // Shift in bit with clock pulse
            shift_register.clock.set_high().ok();
            delay.delay_us(1u32);
            shift_register.clock.set_low().ok();
            delay.delay_us(1u32);
        }

        delay.delay_us(1u32); // Data hold time

        // Latch data into internal output register
        shift_register.latch.set_high().ok();
        delay.delay_us(1u32);
        shift_register.latch.set_low().ok();
        delay.delay_us(1u32);
    }

    // Writes to the led/hall sensor grid shift registers
    // Led/hall can be selected using a bitboard bit
    pub fn write_grid(shift_register: &mut ShiftRegister, delay: &mut Delay, bit: usize, leds_on: bool) {
        let grid_coordinates = bit_to_cartesian(bit as i8);

        let mut shift_num: u64 = 0;
        shift_num += grid_coordinates[1] as u64;      // The led y coordinate occupies bit 0,1,2 of the shift register
        shift_num += (grid_coordinates[0] as u64) << 3; // The led x coordinate occupies bit 3,4,5 of the shift register

        // The hall sensor x and y coordinates occupy the same bits but on the most significant shift register
        shift_num += shift_num << 8;

        // Set bit 7 high to disable leds
        if !leds_on {
            shift_num += 1 << 7;
        }
    }
}