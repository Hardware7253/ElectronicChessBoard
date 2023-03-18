pub mod board_representation {

    #[derive(PartialEq, Debug)]
    pub struct Points { 
        white_points: u8,
        black_points: u8,
    }

    // Indexes for bitboards
    // White pawn = 0
    // White rook = 1
    // White knight = 2
    // White bishop = 3
    // White queen = 4
    // White king = 5

    // Black pawn = 6
    // Black rook = 7
    // Black knight = 8
    // Black bishop = 9
    // Black queen = 10
    // Black king = 11

    // Last bitboard (index 12) is for moves
    
    #[derive(PartialEq, Debug)]
    pub struct Board {
        pub board: [u64; 13],
        pub whites_move: bool,
        pub points: Points,
        pub turns_since_capture: u8,
        pub en_passant_target: usize,
    }

    // Coordinates used to reference a single piece on the board
    pub struct BoardCoordinates {
        pub board_index: usize,
        pub bit: usize,
    }

    // Layout of one bitboard
    // The top left square is bit 0 and the bottom right square is bit 63
    // y is flipped for easier math throughout the program
    // Snakes likes this
    //   0 | 0, 1, 2
    //   1 | 3, 4, 5
    //   2 | 6, 7, 8
    //   y |--------
    //     x 0, 1, 2
    // Bitboards do not need to be flipped to perspective of moving team

    impl Board {

        // Create empty board
        pub fn new() -> Self {
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
                board.turns_since_capture += char_num as u8;
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
        // Chars correspond to piece index on bitboard array
        let char_ids = ['P', 'R', 'N', 'B', 'Q', 'K', 'p', 'r', 'n', 'b', 'q', 'k'];
        for i in 0..char_ids.len() {
            if character == char_ids[i] {
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
    use super::*;

    pub fn gen_piece(piece: board_representation::BoardCoordinates, team_bitboards: crate::TeamBitboards, only_gen_attacks: bool, pieces_info: &[crate::piece::constants::PieceInfo; 12]) -> u64 {
        use crate::bit_on;
        let piece_info = &pieces_info[piece.board_index];
        let initial_piece_bitboard = 1 << piece.bit;

        // Stores all the valid moves for the given piece
        let mut moves_bitboard = 0;

        if !only_gen_attacks && {piece.board_index != 0 || piece.board_index != 6} // Do not generate regular moves for pawns if only_gen_attacks is true
        {
            for i in 0..piece_info.moves_no {

                // Stores the current piece bitboard (where the piece is isolated so that it is the only thing on the bitboard)
                let mut piece_bitboard = initial_piece_bitboard;
                
                let move_delta_bit = piece_info.moves[i];
    
                let mut piece_bit = piece.bit;
                loop {
                    match move_piece(piece_bit, move_delta_bit) {
                        Ok(bitboard) => {
                            let piece_bit_i8: i8 = piece_bit.try_into().unwrap();
                            let piece_move_bit = usize::try_from(piece_bit_i8 + move_delta_bit).unwrap();
    
                            // If a friendly piece is in the way of the move break the loop so the piece cannot move there
                            if bit_on(team_bitboards.friendly_team, piece_move_bit) {
                                break;
                            }
    
                            let mut break_after_move = false;
                            // If an enemy piece is in the way of the move break after the move, so the piece can capture it and then stop moving in that direction
                            if bit_on(team_bitboards.enemy_team, piece_move_bit) {
    
                                // If the piece can only move don't allow it to capture
                                if piece_info.move_only {
                                    break;
                                }
                                break_after_move = true;
                            }
    
                            // Update bitboards
                            moves_bitboard |= bitboard;
                            piece_bitboard = bitboard;
    
                            if break_after_move {
                                break;
                            }
                            
    
                            // Update piece_bit to make it current with the pieces location after the move
                            piece_bit = piece_move_bit;
                        },
                        Err(()) => break,
                    }
    
                    // If the piece doesn't slide break the while loop so it can only move once in each direction
                    if !piece_info.sliding {
                        break;
                    }
                }
            }
        }
        
        moves_bitboard
    }

    // Returns a bitboard where a piece is moved from inital by delta bit
    // Only moves if the piece will still be on the board
    fn move_piece(initial_bit: usize, delta_bit: i8) -> Result<u64, ()> {
        use crate::bit_move_valid;

        if bit_move_valid(initial_bit, delta_bit) {
            let mut move_bitboard = 1 << initial_bit; // Set move bitboard to have piece at initial_bit

            // Shift left / right dependign on delta_bit
            if delta_bit > 0 {
                move_bitboard = move_bitboard << delta_bit.abs();
            } else {
                move_bitboard = move_bitboard >> delta_bit.abs();
            }
            return Ok(move_bitboard);
        }
        Err(())
    }

    #[cfg(test)]
    mod tests {
        use board_representation::BoardCoordinates;
        use board_representation::fen_decode;
        use crate::TeamBitboards;
        use super::*;
        
        #[test]
        fn move_piece_test() {
            assert_eq!(move_piece(60, -8), Ok(1 << 52));
            assert_eq!(move_piece(63, -15), Err(()));
        }

        #[test]
        fn gen_piece_test() {
            use crate::TeamBitboards;
            //use crate::
            let board = fen_decode("3p4/8/6p1/1P6/8/3Q3P/8/5p2 w - - 0 1", true);

            let piece = BoardCoordinates {
                board_index: 4,
                bit: 43,
            };

            let team_bitboards = TeamBitboards::new(piece.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let result = gen_piece(piece, team_bitboards, false, &pieces_info);

            let expected: u64 = 3034431211759470600;
            
            assert_eq!(result, expected);
        }
    }
}
