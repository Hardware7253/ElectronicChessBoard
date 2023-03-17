pub mod board;

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

// Chess coordinate notation to bitboard bit index
pub fn ccn_to_i(ccn: &str) -> Result<usize, ()> {
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

// Converts a bit number (e.g. bit 7 in a u64) to a cartesian coordinates on the board
fn bit_to_cartesian(bit: i8) -> [i8; 2] {
    [bit % 8, bit / 8]
}

// Checks if moving piece from inital by delta bit causes it to go outside of the board space in cartesian coordinates
pub fn bit_move_valid(initial_bit: usize, delta_bit: i8) -> bool {
    let initial_bit: i8 = initial_bit.try_into().unwrap();
    let initial_coordinates = bit_to_cartesian(initial_bit);

    let control_coordinates = bit_to_cartesian(27); // A set of coordinates that every piece can perform a move from, without going out of bounds
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

#[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn char_to_num_test() {
            assert_eq!(char_to_num('4', 0), Ok(4));
        }

        #[test]
        fn ccn_to_i_test() {
            assert_eq!(ccn_to_i("a1"), Ok(56));
            assert_eq!(ccn_to_i("a9"), Err(()));
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

    }