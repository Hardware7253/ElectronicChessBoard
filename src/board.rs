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
        turns_since_capture: usize,
        en_passant_target: usize,
    }

    // Layout of one bitboard
    // The top left square is bit 0 and the bottom right square is bit 63
    // Snakes likes this
    // 0, 1, 2
    // 3, 4, 5
    // 6, 7, 8
    // Bitboards do not need to be flipped to perspective of moving team

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
                turns_since_capture: 0,
                en_passant_target: 0,
            }
        }

        fn setup() {
        }
    }

    pub fn fen_decode(fen: &str, master: bool) -> Board {
        let fen_vec: Vec<char> = fen.chars().collect();
        let init_board: u64 = 0;

        let mut spaces = 0;
        let mut ccn = String::new();

        let mut board = Board::new();
        board.board[12] = u64::MAX;

        let mut piece_square = 0;
        for i in 0..fen_vec.len() {
            let mut skip_add_piece = false;

            let fen_char = fen_vec[i];

            // If the fen char is a number add it to the piece_index (because numbers in a fen string denote empty squares)
            let mut char_num = 0;
            match crate::char_to_num(fen_char, 0) {
                Ok(num) => {piece_square += num; char_num = num; skip_add_piece = true},
                Err(()) => (),
            }

            // Add pieces to bitboard
            if fen_char != '/' && spaces == 0 && !skip_add_piece {
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



            if fen_char == ' ' {
                spaces += 1;
            }

            // Set which team to move
            if fen_vec[i] == 'b' && spaces == 1 {
                board.whites_move = false;
            }

            // Set castling conditions
            if spaces == 2 {

                // Set king / queen side rook turns to 0
                let mut white_king_moved = true;
                let mut black_king_moved = true;
                if fen_vec[i] == 'K' {
                    board.board[12] ^= 1 << 63;
                    white_king_moved = false;
                }
                if fen_vec[i] == 'Q' {
                    board.board[12] ^= 1 << 56;
                    white_king_moved = false;
                }
                if fen_vec[i] == 'k' {
                    board.board[12] ^= 1 << 7;
                    black_king_moved = false;
                }
                if fen_vec[i] == 'q' {
                    board.board[12] ^= 1;
                    black_king_moved = false;
                }

                // Set white / black king turns to 0
                if !white_king_moved {
                    board.board[12] ^= 1 << 60;
                }
                if !black_king_moved {
                    board.board[12] ^= 1 << 4;
                }
            }

            // Set en passant conditions
            if spaces == 3 && fen_char != ' '{
                ccn.push(fen_char);
                match crate::ccn_to_i(&ccn) {
                    Ok(bit) =>  board.en_passant_target = bit,
                    Err(()) => (),
                }
            }

            // Set turns since last capture
            if spaces == 4 {
                board.turns_since_capture *= 10;
                board.turns_since_capture += <i8 as TryInto<usize>>::try_into(char_num).unwrap();
            }

            // Ignore the rest of the fen code
        }

        // Set pawn turns to 0 if they have not moved from their original position
        if master {
            let default_board = fen_decode("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", false);
            let pawns_0_move = {default_board.board[0] & board.board[0]} | {default_board.board[6] & board.board[6]};
            board.board[12] ^= pawns_0_move; // Set turns of pawns in defualt position to 0
        }

        if spaces < 5 {
            panic!("Invalid fen code");
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
        fn fen_decode_test() {
            let board = fen_decode("N7/4p3/8/8/8/8/8/7N w Kq e3 50 1", true);

            let mut expected = Board::new();

            // Piece positions
            expected.board[2] = 1 << 63;
            expected.board[2] |= 1;
            expected.board[6] = 1 << 12;
            expected.board[12] = u64::MAX;

            // Pawn turns
            expected.board[12] ^= 1 << 12;

            // Castle conditions
            expected.board[12] ^= 1 << 60;
            expected.board[12] ^= 1 << 63;

            expected.board[12] ^= 1 << 4;
            expected.board[12] ^= 1;

            // En passant conditions
            expected.en_passant_target = 44;

            // Half moves
            expected.turns_since_capture = 50;


            assert_eq!(board, expected);

        }
    }
}

pub mod move_generator {
    pub fn gen_piece() {

    }
}
