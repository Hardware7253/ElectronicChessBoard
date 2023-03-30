pub mod board_representation {

    #[derive(Copy, Clone, PartialEq, Debug)]
    pub struct Points { 
        pub white_points: i8,
        pub black_points: i8,
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

    // Last bitboard (index 12) is for moves (0 for moved 0 times, 1 for moved > 0 times)
    
    #[derive(Copy, Clone, PartialEq, Debug)]
    pub struct Board {
        pub board: [u64; 13], // Bitboards for every type of piece
        pub whites_move: bool, 
        pub points: Points, // White and black team points
        pub points_delta: i8, // Change in points for team after the last move
        pub turns_since_capture: u8,
        pub en_passant_target: Option<usize>, // En passant target bit
    }

    // Coordinates used to reference a single piece on the board
    #[derive(Copy, Clone, PartialEq, Debug)]
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
                points_delta: 0,
                turns_since_capture: 0,
                en_passant_target: None,
            }
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

    pub fn gen_piece(piece: &board_representation::BoardCoordinates, team_bitboards: crate::TeamBitboards, only_gen_attacks: bool, board: &board_representation::Board, pieces_info: &[crate::piece::constants::PieceInfo; 12]) -> Moves {
        use crate::bit_on;

        let piece_info = &pieces_info[piece.board_index];
        let initial_piece_bitboard = 1 << piece.bit; // Stores a bitboard where the piece is isolated

        let mut moves = Moves::new();

        let piece_pawn;
        if piece.board_index == 0 || piece.board_index == 6 {
            piece_pawn = true;
        } else {
            piece_pawn = false
        }

        // Initialize moves with pawn capture moves if the piece is a pawn
        if piece_pawn {
            moves = gen_pawn_captures(&piece, only_gen_attacks, team_bitboards, &board);
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

                            let mut break_after_move = false;
    
                            // If a friendly piece is in the way of the move break the loop so the piece cannot move there
                            if bit_on(team_bitboards.friendly_team, piece_move_bit) {
                                if only_gen_attacks {
                                    break_after_move = true; // Considered friendly sqaures attacked if only_gen_attacks is true
                                } else {
                                    break;
                                }
                            }
    
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
    fn gen_pawn_captures(piece: &board_representation::BoardCoordinates, force_attacks: bool, mut team_bitboards: crate::TeamBitboards, board: &board_representation::Board) -> Moves {
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

            if bit_on(team_bitboards.enemy_team, piece_move_bit) || force_attacks { // Only try to move if an enemy occupies the square that will be moved to or force attacks is true
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
    fn castle(king: &board_representation::BoardCoordinates, king_move_bit: usize, team_bitboards: &crate::TeamBitboards, enemy_attack_bitboard: u64, board: &board_representation::Board) -> Moves {
        use crate::bit_on;

        // If the king is in check, or has moved don't castle
        if bit_on(enemy_attack_bitboard, king.bit) || !bit_on(board.board[12], king.bit) {
            Moves::new();
        }

        let all_pieces_bitboard = team_bitboards.friendly_team | team_bitboards.enemy_team;

        let king_castle_moves: [i8; 2] =         [1, -1];
        let rook_relative_coordinates: [i8; 2] = [3, -4];
        let rook_castle_moves: [i8; 2] =         [-2, 3];
        
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

                if j == 1 && piece_move_bit == king_move_bit {

                    // Check there is no piece next to the rook on the queen side
                    if i == 1 && bit_on(all_pieces_bitboard, usize::try_from(piece_bit as i8 + king_castle_moves[i]).unwrap()) {
                        Moves::new();
                    } else {
                        return Moves {
                            moves_bitboard: 0,
                            en_passant_target_bit: Some((rook_bit as i8 + rook_castle_moves[i]).try_into().unwrap()), // Use en passant target to show where a rook should be added on the board
                            en_passant_capture_bit: Some(rook_bit), // Use en passant capture bit to show where a rook should be removed from the board
                        };
                    }
                    
                }
            }
        }

        Moves::new()
    }

    #[derive(PartialEq, Debug)]
    pub struct EnemyAttacks {
        enemy_attack_bitboard: u64,
        checking_pieces: [Option<board_representation::BoardCoordinates>; 2],
        checking_pieces_no: usize,
    }

    // Generates atacks of enemys to the kings team, stores enemy pieces that put the king in check
    fn gen_enemy_attacks(king: &board_representation::BoardCoordinates, team_bitboards: crate::TeamBitboards, board: &board_representation::Board, pieces_info: &[crate::piece::constants::PieceInfo; 12]) -> EnemyAttacks {
        use board_representation::BoardCoordinates;
        use crate::bit_on;

        // Get enemy board indexes
        let enemy_indexes;
        if king.board_index < 6 {
            enemy_indexes = 6..12
        } else {
            enemy_indexes = 0..6
        }

        // Swap team bitboard to the persepctive of the enemy team
        let team_bitboards = crate::TeamBitboards {
            friendly_team: team_bitboards.enemy_team,
            enemy_team: team_bitboards.friendly_team,
        };

        let mut checking_pieces_no = 0; // Used to index checking_pieces
        let mut checking_pieces: [Option<BoardCoordinates>; 2] = [None; 2]; // Stores enemy pieces putting the king in check

        let mut enemy_attack_bitboard: u64 = 0;

        // Loop through all enemy pieces
        for i in enemy_indexes {
            for j in 0..64 {

                // Skip if there is no piece on the square
                if !bit_on(board.board[i], j) {
                    continue;
                }

                let board_coordinates = BoardCoordinates {
                    board_index: i,
                    bit: j,
                };

                // Get enemy piece moves (attack moves only) and or them into the enemy moves bitboard
                let piece_moves = gen_piece(&board_coordinates, team_bitboards, true, board, pieces_info);
                enemy_attack_bitboard |= piece_moves.moves_bitboard;

                // If an enemy attack and king are in the same bit then the king is put in check by that piece
                if bit_on(piece_moves.moves_bitboard, king.bit) {
                    checking_pieces[checking_pieces_no] = Some(board_coordinates); // Add piece board coordinates to checking pieces array
                    checking_pieces_no += 1;
                }
            }
        }

        EnemyAttacks {
            enemy_attack_bitboard: enemy_attack_bitboard,
            checking_pieces: checking_pieces,
            checking_pieces_no: checking_pieces_no,
        }
    }

    fn is_mate(king: &board_representation::BoardCoordinates, enemy_attacks: &EnemyAttacks, team_bitboards: crate::TeamBitboards, board: &board_representation::Board, pieces_info: &[crate::piece::constants::PieceInfo; 12]) -> bool {
        use crate::bit_on;
        use board_representation::BoardCoordinates;

        let king_piece_info = &pieces_info[5];

        // Check if king can move to any available squares
        let mut king_can_move = false;
        let king_no_move_bitboard = enemy_attacks.enemy_attack_bitboard | team_bitboards.friendly_team; // Bitboard containing squares the king can't move to
        for i in 0..king_piece_info.moves_no {
            if crate::bit_move_valid(king.bit, king_piece_info.moves[i]) { // Check move is valid on the board space
                let king_move_bit = usize::try_from(king.bit as i8 + king_piece_info.moves[i]).unwrap();
                if !bit_on(king_no_move_bitboard, king_move_bit) { // Check if the king can safely move to the new coordinates
                    king_can_move = true;
                    break;
                }
            }
        }

        if !king_can_move {

            // If the king is in double check and the king can't move then it is mate
            if enemy_attacks.checking_pieces_no > 1 {
                return true;
            }

            let checking_piece;
            let use_checking_piece;
            if enemy_attacks.checking_pieces_no == 1 {
                checking_piece = enemy_attacks.checking_pieces[0].unwrap();
                use_checking_piece = true;
            } else {
                checking_piece = BoardCoordinates {
                    board_index: 0,
                    bit: 0,
                };
                use_checking_piece = false;
            }

            // Get friendly board indexes
            // Don't include kings
            let friendly_indexes;
            if crate::board_index_white(king.board_index) {
                friendly_indexes = 0..5
            } else {
                friendly_indexes = 6..11
            }

            let mut friendly_moves: u64 = 0;
            for i in friendly_indexes {
                for j in 0..64 {
                    let board_coordinates = BoardCoordinates {
                        board_index: i,
                        bit: j,
                    };

                    // Skip if there is no piece on the square
                    if !bit_on(board.board[i], j) {
                        continue;
                    }

                    let piece_moves = gen_piece(&board_coordinates, team_bitboards, false, board, pieces_info);
                    
                    // If the piece that is putting the king in check can be captured then it is not mate
                    if use_checking_piece {
                        if bit_on(piece_moves.moves_bitboard, checking_piece.bit) {
                            return false;
                        }
                    } else { // If there is no piece putting the king in check at piece moves to friendly moves bitboard
                        friendly_moves |= piece_moves.moves_bitboard;
                    }
                }
            }

            // If there are no valid moves for the king or any other pieces then it is mate
            if friendly_moves == 0 {
                return true;
            }
        }
        false
    }

    // Errors that can be encountered when a team makes a turn
    #[derive(PartialEq, Debug)]
    pub enum TurnError {
        Win, // The team wins
        Draw, // The team draws
        InvalidMove, // The piece cannot move to the new coordinates
        InvalidMoveCheck, // The piece cannot move to the new coordinates because the king is in check
    }

    // Move piece to piece_move_bit if the move is valid
    // If move is valid update the board, else return an error
    pub fn new_turn(piece: &board_representation::BoardCoordinates, piece_move_bit: usize, friendly_king: &board_representation::BoardCoordinates, enemy_king: &board_representation::BoardCoordinates, enemy_attacks: &EnemyAttacks, team_bitboards: crate::TeamBitboards, mut board: board_representation::Board, pieces_info: &[crate::piece::constants::PieceInfo; 12]) -> Result<board_representation::Board, TurnError> {
        use crate::TeamBitboards;

        // If the piece is a king generate castle moves
        let mut piece_moves = Moves::new();
        if piece == friendly_king {
            piece_moves = castle(piece, piece_move_bit, &team_bitboards, enemy_attacks.enemy_attack_bitboard, &board);
        }

        // Get piece team
        let piece_white = crate::board_index_white(piece.board_index);

        // If the castle was valid move the rook
        // If the castle was not valid generate regular piece moves
        let mut castled = false;
        match piece_moves.en_passant_target_bit {
            Some(rook_add_bit) => {
                let rook_remove_bit = piece_moves.en_passant_capture_bit.unwrap();

                // Get friendly rook board index
                let friendly_rook_board_index;
                if piece_white {
                    friendly_rook_board_index = 1;
                } else {
                    friendly_rook_board_index = 7;
                }

                // Remove rook rook_remove_bit and add rook at rook_add_bit
                board.board[friendly_rook_board_index] ^= 1 << rook_remove_bit | 1 << rook_add_bit;
                castled = true;
            },
            None => piece_moves = gen_piece(piece, team_bitboards, false, &board, pieces_info), // Gen non castle moves
        }
        
        // Return an error if the piece_move bit is not on in the piece_moves bitboard
        if !crate::bit_on(piece_moves.moves_bitboard, piece_move_bit) {
            return Err(TurnError::InvalidMove);
        }

        if !castled {

            // Remove en passant capture from the board
            match piece_moves.en_passant_capture_bit {
                Some(capture_bit) => {
                    let en_passant_capture_xor_bitboard: u64 = 1 << capture_bit;
                        
                        // Get enemy pawn board index
                        let enemy_pawn_board_index;
                        if piece_white {
                            enemy_pawn_board_index = 6;
                        } else {
                            enemy_pawn_board_index = 0;
                        }
                        
                        board.board[enemy_pawn_board_index] ^= en_passant_capture_xor_bitboard; // Remove en passant capture piece from the board
                },
                None => (),
            }

            // Update board en passant target
            board.en_passant_target = piece_moves.en_passant_target_bit;
        }
        
        // Move piece on board to new coordinates (bit)
        let piece_move_bitoard = 1 << piece_move_bit;
        let piece_move_xor_bitboard = 1 << piece.bit | piece_move_bitoard;
        board.board[piece.board_index] ^= piece_move_xor_bitboard;

        // Update piece moves bitboard
        board.board[12] |= piece_move_bitoard;

        // Get enemy board indexes
        let enemy_indexes;
        let enemy_index;
        if piece_white {
            enemy_indexes = 6..12;
            enemy_index = 6;
        } else {
            enemy_indexes = 0..6;
            enemy_index = 0;
        }

        // If a piece was captured remove it on the appropriate enemy bitboard and store the captured pieces value
        let mut value = 0;
        for i in enemy_indexes {
            let new_piece_bitboard = board.board[i] ^ piece_move_bitoard ;
            if new_piece_bitboard < board.board[i] {
                board.board[i] = new_piece_bitboard;
                value = pieces_info[i].value;
                break;
            }
        }
        board.points_delta = value;

        if value == 0 {
            board.turns_since_capture += 1;
        } else {
            board.turns_since_capture = 0;
        }

        // If the king is in check after the move return an error
        let enemy_attacks = gen_enemy_attacks(friendly_king, team_bitboards, &board, pieces_info);
        if enemy_attacks.checking_pieces_no != 0 {
            return Err(TurnError::InvalidMoveCheck);
        }

        // Generate necassary values to check if the enemy king has been mated
        let enemy_team_bitboards = TeamBitboards {
            friendly_team: team_bitboards.enemy_team ^ piece_move_bitoard,
            enemy_team: team_bitboards.friendly_team ^ piece_move_xor_bitboard,
        };

        let friendly_attacks = gen_enemy_attacks(enemy_king, enemy_team_bitboards, &board, pieces_info);
        let enemy_mate =  is_mate(enemy_king, &friendly_attacks, enemy_team_bitboards, &board, pieces_info);

        // Return errors for the end of the game if there is a mate
        if enemy_mate {
            if friendly_attacks.checking_pieces_no == 0 {
                return Err(TurnError::Draw); // Stalemate
            }
            return Err(TurnError::Win); // Checkmate
        }

        // Update points
        if piece_white {
            board.points.white_points += value;
        } else {
            board.points.black_points += value;
        }

        Ok(board)
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

            let result = gen_piece(&piece, team_bitboards, false, &board, &pieces_info);

            let expected: u64 = 3034431211759470600;
            
            assert_eq!(result.moves_bitboard, expected);
        }

        #[test]
        fn gen_piece_test2() { // Test pawn double move
            use crate::TeamBitboards;

            let board = fen_decode("8/8/8/8/8/8/3P4/8 w - - 0 1", true);

            let piece = BoardCoordinates {
                board_index: 0,
                bit: 51,
            };

            let team_bitboards = TeamBitboards::new(piece.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let result = gen_piece(&piece, team_bitboards, false, &board, &pieces_info);

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

            let result = gen_pawn_captures(&piece, false, team_bitboards, &board);

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

            let board = fen_decode("r3k2r/8/8/8/8/8/8/8 w kq - 0 1", true);

            let piece = BoardCoordinates {
                board_index: 5,
                bit: 4,
            };

            let team_bitboards = TeamBitboards::new(piece.board_index, &board);

            let expected = Moves {
                moves_bitboard: 0,
                en_passant_target_bit: Some(3),
                en_passant_capture_bit: Some(0),
            };
            
            assert_eq!(castle(&piece, 2, &team_bitboards, 0, &board), expected); // Valid castle
            assert_eq!(castle(&piece, 2, &team_bitboards, 4, &board), Moves::new()); // Test an enemy attack blocking the path
        }

        #[test]
        fn gen_enemy_attacks_test() {
            use crate::TeamBitboards;

            let board = fen_decode("7r/p5k1/P7/5N2/3B4/P7/1P6/8 w - - 0 1", true);

            let king = BoardCoordinates {
                board_index: 11,
                bit: 14,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let result = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let checking_pieces: [Option<BoardCoordinates>; 2] = [
                Some(BoardCoordinates {
                    board_index: 2,
                    bit: 29,
                }),

                Some(BoardCoordinates {
                    board_index: 3,
                    bit: 35,
                }),
            ];

            let expected = EnemyAttacks {
                enemy_attack_bitboard: 4621350219176104704,
                checking_pieces: checking_pieces,
                checking_pieces_no: 2,
            };
            
            assert_eq!(result, expected);
        }

        #[test]
        fn gen_enemy_attacks_test2() {
            use crate::TeamBitboards;

            let board = fen_decode("K7/8/8/8/3k4/8/8/8 w - - 0 1", true);

            let king = BoardCoordinates {
                board_index: 5,
                bit: 0,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let result = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let checking_pieces: [Option<BoardCoordinates>; 2] = [None, None];

            let expected = EnemyAttacks {
                enemy_attack_bitboard: 30872694685696,
                checking_pieces: checking_pieces,
                checking_pieces_no: 0,
            };
            
            assert_eq!(result, expected);
        }

        #[test]
        fn is_mate_test1() { // Test check mate
            use crate::TeamBitboards;
            
            let board = fen_decode("6R1/8/8/8/6N1/7k/6P1/4BP2 w - - 0 1", true);

            let king = BoardCoordinates {
                board_index: 11,
                bit: 47,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let enemy_attacks = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let result = is_mate(&king, &enemy_attacks, team_bitboards, &board, &pieces_info);
            
            assert_eq!(result, true);
        }

        #[test]
        fn is_mate_test2() { // Test stale mate
            use crate::TeamBitboards;

            let board = fen_decode("K7/2q5/8/8/5p2/5P2/8/8 w - - 0 1", true);

            let king = BoardCoordinates {
                board_index: 5,
                bit: 0,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let enemy_attacks = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let result = is_mate(&king, &enemy_attacks, team_bitboards, &board, &pieces_info);
            
            assert_eq!(result, true);
        }

        #[test]
        fn is_mate_test3() { // Test check mate where a king can't move next to enemy king
            use crate::TeamBitboards;

            let board = fen_decode("8/8/8/8/3b4/8/P7/K1k5 w - - 0 1", true);

            let king = BoardCoordinates {
                board_index: 5,
                bit: 56,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let enemy_attacks = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let result = is_mate(&king, &enemy_attacks, team_bitboards, &board, &pieces_info);
            
            assert_eq!(result, true);
        }

        #[test]
        fn new_turn_test1() { // Test board piece positions, moves bitboard, and points being updated after a move
            use crate::TeamBitboards;

            let board = fen_decode("4k3/8/8/1N3q2/8/3B4/8/K7 w - - 0 1", true);

            let king = BoardCoordinates {
                board_index: 5,
                bit: 56,
            };

            let enemy_king = BoardCoordinates {
                board_index: 11,
                bit: 4,
            };

            let piece = BoardCoordinates {
                board_index: 3,
                bit: 43,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let enemy_attacks = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let mut expected_board = fen_decode("4k3/8/8/1N3B2/8/8/8/K7 w - - 0 1", true);

            let new_turn_board = new_turn(&piece, 29, &king, &enemy_king, &enemy_attacks, team_bitboards, board, &pieces_info);

        
            expected_board.points.white_points = 9;
            expected_board.points_delta = 9;
            expected_board.board[12] |= 1 << 29;

            assert_eq!(new_turn_board, Ok(expected_board));            
        }

        #[test]
        fn new_turn_test2() { // Test moving a piece to win a game with checkmate
            use crate::TeamBitboards;

            let board = fen_decode("k7/6Q1/8/2N5/8/8/8/7K w - - 0 1", true);

            let king = BoardCoordinates {
                board_index: 5,
                bit: 63,
            };

            let enemy_king = BoardCoordinates {
                board_index: 11,
                bit: 0,
            };

            let piece = BoardCoordinates {
                board_index: 4,
                bit: 14,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let enemy_attacks = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let new_turn_board = new_turn(&piece, 9, &king, &enemy_king, &enemy_attacks, team_bitboards, board, &pieces_info);

            assert_eq!(new_turn_board, Err(TurnError::Win));            
        }
    }
}
