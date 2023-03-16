pub mod board_representation {

    #[derive(PartialEq, Debug)]
    pub struct Points { 
        white_points: usize,
        black_points: usize,
    }
    
    // IDS defines how indexes on the board array corresponds to piece bitboards
    // Last index is for a moves board
    //                            PP, RR, NN, BB, QQ, KK, pp, rr, nn, bb, qq, kk, 00 
    pub const IDS: [usize; 13] = [00, 01, 02, 03, 04, 05, 06, 07, 08, 09, 10, 11, 12];

    // Char identifier for a piece, uses the same order as IDS
    pub const CHAR_IDS: [char; 12] = ['P', 'R', 'N', 'B', 'Q', 'K', 'p', 'r', 'n', 'b', 'q', 'k'];

    #[derive(PartialEq, Debug)]
    pub struct Board {
        board: [u64; 13],
        whites_move: bool,
        points: Points,
    }

    impl Board {

        // Create empty board
        fn new() -> Self {
            Board {
                board: [0; 13],
                whites_move: true,
                points: Points {
                    white_points: 0,
                    black_points: 0,
                },
            }
        }

        fn setup() {
        }
    }

    pub fn fen_decode(fen: &str) -> Board {
        let fen_vec: Vec<char> = fen.chars().collect();
        let init_board: u64 = 0;

        let mut spaces = 0;

        let mut board = Board::new();
        let mut piece_square = 0;
        for i in 0..fen_vec.len() {

            let fen_char = fen_vec[i];

            // If the fen char is a number add it to the piece_index (because numbers in a fen string denote empty squares)
            match crate::char_to_num(fen_char) {
                Ok(num) => {piece_square += num; continue},
                Err(()) => (),
            }

            // Add pieces to bitboard
            if fen_char != '/' && spaces == 0{
                let piece_index = piece_index_from_char(fen_char);

                match piece_index {
                    Ok(index) => {
                        // Turn bit on at piece_square on piece_index bitboard
                        let bitboard_value = 1 << piece_square;
                        board.board[index] |= bitboard_value;
                    }
                    Err(()) => ()
                }
                piece_square += 1;
            }

            // Set which team to move
            if fen_char == ' ' {
                spaces += 1;

                if fen_vec[i + 1] == 'b' {
                    board.whites_move = false;
                }
            }

            // Ignore the rest of the fen code
        }
        board
    }

    fn piece_index_from_char(character: char) -> Result<usize, ()> {
        for i in 0..CHAR_IDS.len() {
            if character == CHAR_IDS[i] {
                return Ok(i);
            }
        }
        Err(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        pub fn fen_decode_test() {
            let board = fen_decode("N6p/8/8/8/8/8/8/7N w - - 0 1");

            let mut expected = Board::new();
            expected.board[2] = 1 << 63;
            expected.board[2] |= 1;
            expected.board[6] = 1 << 7;


            assert_eq!(board, expected);

        }
    }
}

pub mod move_generator {

}
