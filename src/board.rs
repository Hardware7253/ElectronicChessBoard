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
        pub en_passant_target: Option<usize>,
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
                en_passant_target: None,
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
                    Ok(bit) =>  board.en_passant_target = Some(bit),
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
            expected.en_passant_target = Some(44);

            // Half moves
            expected.turns_since_capture = 50;


            assert_eq!(board, expected);

        }
    }
}

pub mod move_generator {
    use super::*;

    #[derive(PartialEq, Debug)]
    pub struct Moves {
        pub moves_bitboard: u64,
        pub en_passant_target_bit: Option<usize>,
        pub en_passant_capture_bit: Option<usize>,
    }

    impl Moves {
        fn new() -> Self {
            Moves {
                moves_bitboard: 0,
                en_passant_target_bit: None,
                en_passant_capture_bit: None,
            }
        }
    }

    pub fn gen_piece(piece: board_representation::BoardCoordinates, team_bitboards: crate::TeamBitboards, only_gen_attacks: bool, board: &board_representation::Board, pieces_info: &[crate::piece::constants::PieceInfo; 12]) -> Moves {
        use crate::bit_on;

        let piece_info = &pieces_info[piece.board_index];
        let initial_piece_bitboard = 1 << piece.bit; // Stores a bitboard where the piece is isolated

        let mut moves = Moves::new();

        let mut piece_pawn = false;
        if piece.board_index == 0 || piece.board_index == 6 {
            piece_pawn = true;
        }

        // Initialize moves with pawn capture moves if the piece is a pawn
        if piece_pawn {
            moves = gen_pawn_captures(&piece, team_bitboards, &board);
        }

        if only_gen_attacks && piece_pawn // Do not generate regular moves for pawns if only_gen_attacks is true
        {        
        } else {
            for i in 0..piece_info.moves_no {
                let mut piece_bitboard = initial_piece_bitboard;
                
                let move_delta_bit = piece_info.moves[i];
    
                let mut piece_bit = piece.bit;

                let mut move_repeated = 0;
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
                            moves.moves_bitboard |= bitboard;
                            piece_bitboard = bitboard;
                            move_repeated += 1;
    
                            if break_after_move {
                                break;
                            }
                            
    
                            // Update piece_bit to make it current with the pieces location after the move
                            piece_bit = piece_move_bit;
                        },
                        Err(()) => break,
                    }

                    // Do not let a pawn move more then 2 times in a turn
                    if piece_pawn && move_repeated == 2 {
                        break;
                    }

                    // Allow a pawn to move 2 times in a turn if it has moved 0 times
                    if piece_pawn && !bit_on(board.board[12], piece.bit) {
                        moves.en_passant_target_bit = Some(piece_bit);
                        continue;
                    }  

                    // If the piece doesn't slide break the while loop so it can only move once in each direction
                    if !piece_info.sliding {
                        break;
                    }
                }
            }
        }
        
        moves
    }

    // Generates pawn capture moves including en passant, assumes the given piece is a pawn
    fn gen_pawn_captures(piece: &board_representation::BoardCoordinates, mut team_bitboards: crate::TeamBitboards, board: &board_representation::Board) -> Moves {
        use crate::bit_on;

        let initial_piece_bitboard: u64 = 1 << piece.bit; // Stores a bitboard where the piece is isolated

        let mut moves_bitboard = 0; // Stores the capture moves for the pawns

        // Add an imaginary piece at the en passant target so a friendly pawn can capture it
        let en_passant_target_bit = board.en_passant_target.unwrap_or(0);
        let mut en_passant_target_bitboard: u64 = 0;
        if en_passant_target_bit != 0 {
            en_passant_target_bitboard = 1 << en_passant_target_bit;
        }
        team_bitboards.enemy_team |= en_passant_target_bitboard;

        // Get capture moves for pawn
        let capture_moves: [i8; 2];
        let team_white = crate::board_index_white(piece.board_index);
        if team_white {
            capture_moves = [-9, -7];
        } else {
            capture_moves = [9, 7];
        }

        let mut en_passant_capture_bit = None;
        let piece_bit_i8: i8 = piece.bit.try_into().unwrap();
        for i in 0..capture_moves.len() {
            
            let move_delta_bit = capture_moves[i];
            let piece_move_bit_i8 = piece_bit_i8 + move_delta_bit;
            let piece_move_bit = usize::try_from(piece_move_bit_i8).unwrap();
            //println!("{}", en_passant_target_bit);

            if bit_on(team_bitboards.enemy_team, piece_move_bit) { // Only try to move in an enemy occupies the square that will be moved to
                match move_piece(piece.bit, move_delta_bit) {
                    Ok(bitboard) => {
                        moves_bitboard |= bitboard;

                        // Set en passant capture bit if the piece captured an en passant target bit
                        if piece_move_bit == en_passant_target_bit {
                            if team_white {
                                en_passant_capture_bit = Some(usize::try_from(piece_move_bit_i8 + 8).unwrap());
                            } else {
                                en_passant_capture_bit = Some(usize::try_from(piece_move_bit_i8 - 8).unwrap());
                            }
                        }
                    },
    
                    Err(()) => continue,
                }
            }
            
        }

        Moves {
            moves_bitboard: moves_bitboard,
            en_passant_target_bit: None,
            en_passant_capture_bit: en_passant_capture_bit,
        } 

    }

    // Assumes the given piece is a king, and that the moves bitboard is accurate
    fn castle(king: board_representation::BoardCoordinates, king_move_bit: usize, team_bitboards: crate::TeamBitboards, enemy_attack_bitboard: u64, board: &board_representation::Board) -> Result<Moves, ()> {
        use crate::bit_on;

        // If the king is in check, or has moved don't castle
        if bit_on(enemy_attack_bitboard, king.bit) || !bit_on(board.board[12], king.bit) {
            return Err(());
        }

        let all_pieces_bitboard = team_bitboards.friendly_team | team_bitboards.enemy_team;

        let king_castle_moves: [i8; 2] =         [1, -1];
        let rook_relative_coordinates: [i8; 2] = [3, -4];
        let rook_castle_moves: [i8; 2] =          [-2, 3];
        
        let mut king_moves_bitboard: u64 = 0;

        for i in 0..king_castle_moves.len() {
            let mut piece_bit = king.bit;

            // If the rook for this direction has moved the king cannot castle in this direction
            let rook_bit = (piece_bit as i8 + rook_relative_coordinates[i]).try_into().unwrap();
            if bit_on(board.board[12], rook_bit) {
                continue;
            }

            for j in 0..2 {
                let piece_move_bit = usize::try_from(piece_bit as i8 + king_castle_moves[i]).unwrap();
                
                // Do not allow the king to castle through squares that are attacked or to castle through pieces
                if bit_on(enemy_attack_bitboard, piece_move_bit) || bit_on(all_pieces_bitboard, piece_move_bit)  {
                    break;
                }

                piece_bit = piece_move_bit;

                if i > 0 && piece_move_bit == king_move_bit {
                    return Ok(Moves {
                        moves_bitboard: 0,
                        en_passant_target_bit: Some((rook_bit as i8 + rook_castle_moves[i]).try_into().unwrap()), // Use en passant target to show where a rook should be added on the board
                        en_passant_capture_bit: Some(rook_bit), // Use en passant capture bit to show where a rook should be removed from the borad
                    });
                }
            }
        }

        Err(())
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
        fn gen_piece_test1() { // Test queen sliding and being blocked by friendly and enemy pieces
            use crate::TeamBitboards;

            let board = fen_decode("3p4/8/6p1/1P6/8/3Q3P/8/5p2 w - - 0 1", true);

            let piece = BoardCoordinates {
                board_index: 4,
                bit: 43,
            };

            let team_bitboards = TeamBitboards::new(piece.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let result = gen_piece(piece, team_bitboards, false, &board, &pieces_info);

            let expected: u64 = 3034431211759470600;
            
            assert_eq!(result.moves_bitboard, expected);
        }

        #[test]
        fn gen_piece_test2() {
            use crate::TeamBitboards;

            let board = fen_decode("8/8/8/8/8/8/3P4/8 w - - 0 1", true);

            let piece = BoardCoordinates {
                board_index: 0,
                bit: 51,
            };

            let team_bitboards = TeamBitboards::new(piece.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let result = gen_piece(piece, team_bitboards, false, &board, &pieces_info);

            let expected: u64 = 8830452760576;
            
            assert_eq!(result.moves_bitboard, expected);
        }

        #[test]
        fn gen_pawn_captures_test() {
            use crate::TeamBitboards;

            let board = fen_decode("rnbqkbnr/ppppp1pp/8/8/4Pp2/6P1/PPPP1P1P/RNBQKBNR b KQkq e3 0 1", true);

            let piece = BoardCoordinates {
                board_index: 6,
                bit: 37,
            };

            let team_bitboards = TeamBitboards::new(piece.board_index, &board);

            let result = gen_pawn_captures(&piece, team_bitboards, &board);

            let expected = Moves {
                moves_bitboard: 87960930222080,
                en_passant_target_bit: None,
                en_passant_capture_bit: Some(36),
            };
            
            assert_eq!(result, expected);
        }

        #[test]
        fn castle_test() {
            use crate::TeamBitboards;

            let board = fen_decode("r3k2r/8/8/8/8/8/8/8 w KQkq - 0 1", true);

            let piece = BoardCoordinates {
                board_index: 5,
                bit: 4,
            };

            let team_bitboards = TeamBitboards::new(piece.board_index, &board);

            let result = castle(piece, 2, team_bitboards, 0, &board);

            let expected = Moves {
                moves_bitboard: 0,
                en_passant_target_bit: Some(3),
                en_passant_capture_bit: Some(0),
            };
            
            assert_eq!(result, Ok(expected));

        }
    }
}
