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

    }