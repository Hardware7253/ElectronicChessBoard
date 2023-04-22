pub mod board;
pub mod piece;
pub mod algorithm;
pub mod zobrist;

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

// Chess coordinate notation to bitboard bit
pub fn ccn_to_bit(ccn: &str) -> Result<usize, ()> {
    let ccn_vec: Vec<char> = ccn.chars().collect();
    if ccn_vec.len() < 2 {
        return Err(());
    }

    let x = char_to_num(ccn_vec[0], 48);
    let y = char_to_num(ccn_vec[1], 0);

    let x = match x {
        Ok(num) => num,
        Err(()) => return Err(())
    };

    let y = match y {
        Ok(num) => num,
        Err(()) => return Err(())
    };

    if x > 8 || y > 8 {
        return Err(())
    }

    let i = {{y - 8}.abs() * 8} + x - 1;

    Ok(i.try_into().unwrap())
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

#[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn char_to_num_test() {
            assert_eq!(char_to_num('4', 0), Ok(4));
        }

        #[test]
        fn num_to_char_test() {
            assert_eq!(num_to_char(8), Ok('8'));
        }

        #[test]
        fn ccn_to_bit_test() {
            assert_eq!(ccn_to_bit("a1"), Ok(56));
            assert_eq!(ccn_to_bit("a9"), Err(()));
        }
        
        #[test]
        fn bit_to_ccn_test() {
            assert_eq!(bit_to_ccn(28), "e5");
        }

        #[test]
        fn bit_to_cartesian_test() {
            assert_eq!(bit_to_cartesian(27), [3, 3]);  
        }

        #[test]
        fn bit_move_valid_test() {
            assert_eq!(bit_move_valid(56, -17), false);
            assert_eq!(bit_move_valid(60, -8), true);
        }

        #[test]
        fn or_bitboards_test() {
            let board = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];
            assert_eq!(or_bitboards(0, 1, &board), 3);
        }

        #[test]
        fn board_index_white_test() {
            assert_eq!(board_index_white(7), false);
        }

        #[test]
        fn bit_on_test() {
            assert_eq!(bit_on(4, 2), true);
            assert_eq!(bit_on(0, 1), false);
        }

        #[test]
        fn find_bit_on_test() {
            assert_eq!(find_bit_on(8, 0), 3);
        }

        #[test]
        fn flip_bitboard_bit_test() {
            assert_eq!(flip_bitboard_bit(49), 14);
        }

        #[test]
        fn common_bit_test() {
            assert_eq!(common_bit(5, 6), true);
            assert_eq!(common_bit(3, 1), true);
            assert_eq!(common_bit(9, 8), true);
            assert_eq!(common_bit(10, 5), false);
            assert_eq!(common_bit(7, 8), false);
        }

        #[test]
        fn gen_team_bitboards_test() {
            let board = crate::board::board_representation::fen_decode("r2qkbnr/pp1npppp/3N4/1p3b2/3P4/8/PPP1QPPP/R1B1K1NR b - - 0 1", true);

            let result = TeamBitboards::new(10, &board);

            let expected = TeamBitboards {
                friendly_team: 570489849,
                enemy_team: 15417791883686445056,
            };

            assert_eq!(result, expected);
        }
    }