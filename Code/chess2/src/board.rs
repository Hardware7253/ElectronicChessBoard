#![no_std]
#![no_main]

use core::convert::TryFrom;
use core::convert::TryInto;

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
    
    #[derive(Copy, Clone, PartialEq, Debug)]
    pub struct Board {
        pub board: [u64; 13], // Bitboards for every type of piece
        pub whites_move: bool, 
        pub points: Points, // White and black team points
        pub points_delta: i8, // Change in points for team after the last move
        pub half_moves: i16, // The total number of half moves
        pub half_move_clock: i16, // The number of half moves since the last capture or pawn move
        pub en_passant_target: Option<usize>, // En passant target bit
    }

    // Coordinates used to reference a single piece on the board
    #[derive(Copy, Clone, PartialEq, Debug)]
    pub struct BoardCoordinates {
        pub board_index: usize, // Index of a board array the piece occupies
        pub bit: usize, // Bit the piece occupies
    }

    impl BoardCoordinates {
        pub fn new() -> Self {
            BoardCoordinates {
                board_index: 0,
                bit: 0,
            }
        }
    }

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
                half_moves: 0,
                half_move_clock: 0,
                en_passant_target: None,
            }
        }

        // Converts the entire board into a single bitboard
        pub fn to_bitboard(&self) -> u64 {
            let mut bitboard = 0;

            // Only use first 12 bitboards as they correspond to piece positions
            for i in 0..12 {
                bitboard |= self.board[i];
            }

            bitboard
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

    // Returns a move struct of semi legal moves that the piece can make
    // Set only_gen_attacks to true to turn off pawn moves
    // If an enemy king is provided it will be ignored as a piece, so sliding pieces moves will go through the enemy king
    pub fn gen_piece(
        piece: &board_representation::BoardCoordinates,
        enemy_king: Option<&board_representation::BoardCoordinates>,
        team_bitboards: &crate::TeamBitboards,
        only_gen_attacks: bool,
        board: &board_representation::Board,
        pieces_info: &[crate::piece::constants::PieceInfo; 12]
    ) -> Moves {
        use crate::bit_on;

        let piece_info = &pieces_info[piece.board_index];

        let mut moves = Moves::new();

        let piece_pawn;
        if piece.board_index == 0 || piece.board_index == 6 {
            piece_pawn = true;
        } else {
            piece_pawn = false
        }

        // Initialize moves with pawn capture moves if the piece is a pawn
        if piece_pawn {
            moves = gen_pawn_captures(&piece, only_gen_attacks, *team_bitboards, &board);
        }

        if only_gen_attacks && piece_pawn // Do not generate regular moves for pawns if only_gen_attacks is true
        {        
        } else {
            for i in 0..piece_info.moves_no {
                let move_delta_bit = piece_info.moves[i];
    
                let mut piece_bit = piece.bit;

                let mut move_repeated = 0;
                loop {
                    match move_piece(piece_bit, move_delta_bit) {
                        Ok(bitboard) => {
                            let piece_bit_i8: i8 = piece_bit as i8;
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

                                // If an enemy king is supplied, and only_gen_attacks is true, and the piece_move_bit == enemy_king_bit then continue moving through the enemy king.
                                // Because the squares behind the king are technically attacked

                                let mut continue_in_direction = false;
                                match enemy_king {
                                    Some(enemy_king) => {
                                        if enemy_king.bit == piece_move_bit && only_gen_attacks {
                                            continue_in_direction = true;
                                        }
                                    },
                                    None => {
                                        
                                    },
                                }
    
                                // If the piece can only move don't allow it to capture
                                if piece_info.move_only {
                                    break;
                                }

                                if !continue_in_direction {
                                    break_after_move = true;
                                }
                            }
    
                            // Update bitboards
                            moves.moves_bitboard |= bitboard;
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

            // Check moving the piece here is a valid move
            let moved_piece = move_piece(piece.bit, move_delta_bit);
            match moved_piece {
                Ok(_) => (),
                Err(_) => continue,
            }

            let piece_move_bit_i8 = piece_bit_i8 + move_delta_bit;
            let piece_move_bit = usize::try_from(piece_move_bit_i8).unwrap();

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

    // Returns a Moves struct
    // Uses en passant target to show where a rook should be added on the board
    // Uses en passant capture bit to show where a rook should be removed from the board
    fn castle(king: &board_representation::BoardCoordinates, king_move_bit: usize, team_bitboards: &crate::TeamBitboards, enemy_attack_bitboard: u64, board: &board_representation::Board) -> Moves {
        use crate::bit_on;

        // If the king is in check, or has moved don't castle
        if bit_on(enemy_attack_bitboard, king.bit) || bit_on(board.board[12], king.bit) {
            return Moves::new();
        }

        let all_pieces_bitboard = team_bitboards.friendly_team | team_bitboards.enemy_team;

        let king_castle_moves: [i8; 2] =         [1, -1];
        let rook_relative_coordinates: [i8; 2] = [3, -4];
        let rook_castle_moves: [i8; 2] =         [-2, 3];
        
        for i in 0..king_castle_moves.len() {
            let mut piece_bit = king.bit;

            // Get rook bit
            let rook_bit = (piece_bit as i8 + rook_relative_coordinates[i]).try_into();
            let rook_bit = match rook_bit {
                Ok(bit) => bit,
                Err(_) => continue,
            };

            // A castle cannot be performed in this direction if the rook is:
            // Outside the range of the board
            // Doesn't exist on the board
            // Not at moves = 0
            if rook_bit > 63 || !bit_on(board.board[king.board_index - 4], rook_bit) || bit_on(board.board[12], rook_bit) {
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
                        return Moves::new();
                    } else {
                        return Moves {
                            moves_bitboard: 1 << king_move_bit,
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
        pub enemy_attack_bitboard: u64,
        checking_pieces: [Option<board_representation::BoardCoordinates>; 2],
        checking_pieces_no: usize,
    }

    // Generates atacks of enemys to the kings team, stores enemy pieces that put the king in check
    pub fn gen_enemy_attacks(king: &board_representation::BoardCoordinates, team_bitboards: crate::TeamBitboards, board: &board_representation::Board, pieces_info: &[crate::piece::constants::PieceInfo; 12]) -> EnemyAttacks {
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
                let piece_moves = gen_piece(&board_coordinates, Some(king), &team_bitboards, true, board, pieces_info);
                enemy_attack_bitboard |= piece_moves.moves_bitboard;

                // If an enemy attack and king are in the same bit then the king is put in check by that piece
                if bit_on(piece_moves.moves_bitboard, king.bit) && checking_pieces_no < 2{
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

    // Returns true if the given king a move safely
    fn king_can_move(king: &board_representation::BoardCoordinates, enemy_attacks: &EnemyAttacks, team_bitboards: crate::TeamBitboards, pieces_info: &[crate::piece::constants::PieceInfo; 12]) -> bool {
        use crate::bit_on;
        
        let king_no_move_bitboard = enemy_attacks.enemy_attack_bitboard | team_bitboards.friendly_team; // Bitboard containing squares the king can't move to
        let king_piece_info = &pieces_info[5];

        let mut king_can_move = false;
        for i in 0..king_piece_info.moves_no { // Loop through all kings moves to see if there is a safe square to move to
            if crate::bit_move_valid(king.bit, king_piece_info.moves[i]) { // Check move is valid on the board space
                let king_move_bit = usize::try_from(king.bit as i8 + king_piece_info.moves[i]).unwrap();

                if !bit_on(king_no_move_bitboard, king_move_bit) { // Check if the king can safely move to the new coordinates
                    king_can_move = true;
                    break;
                }
            }
        }

        king_can_move
    }

    // Is mate expects the king team, and the current teams turn on the board to be the same
    // Returns true if the given king is in mate (checkmate or stalemate)
    fn is_mate(king: &board_representation::BoardCoordinates, enemy_attacks: &EnemyAttacks, team_bitboards: crate::TeamBitboards, board: &board_representation::Board, pieces_info: &[crate::piece::constants::PieceInfo; 12]) -> bool {
        use crate::bit_on;
        use board_representation::BoardCoordinates;

        // True if the king can safely move
        let king_can_move = king_can_move(king, enemy_attacks, team_bitboards, pieces_info);

        if !king_can_move {

            // If the king is in double check and the king can't move then it is mate
            if enemy_attacks.checking_pieces_no == 2 {
                return true;
            }

            // Get checking piece
            let checking_piece;
            let use_checking_piece;
            match enemy_attacks.checking_pieces[0] {
                Some(piece) => {
                    checking_piece = piece;
                    use_checking_piece = true;
                },
                None => {
                    checking_piece = BoardCoordinates {
                        board_index: 0,
                        bit: 0,
                    };
                    use_checking_piece = false;
                }
            }

            // Get friendly indexes
            // Don't include king
            let friendly_indexes;
            if board.whites_move {
                friendly_indexes = 0..5;
            } else {
                friendly_indexes = 6..11;
            }
            
            for board_index in friendly_indexes {
                for initial_bit in 0..64 {

                    // If a piece doesn't exist at these coordinates continue to the next iteration
                    if !bit_on(board.board[board_index], initial_bit) {
                        continue;
                    }

                    let piece_coordinates = BoardCoordinates {
                        board_index: board_index,
                        bit: initial_bit,
                    };
                                        
                    // If using checking piece generate the piece attacks
                    // Else generate the piece moves
                    // This is because the attacks are needed to check for checkmate, and moves are needed to check for stalemate
                    let piece_attacks = gen_piece(&piece_coordinates, None, &team_bitboards, use_checking_piece, board, pieces_info);

                    // Pawn moves shouldn't be included in the attacks bitboard
                    // But they are still necasary for checking if the piece can block a checking pieces path
                    let pawn_moves_bitboard;
                    if board_index == 0 || board_index == 6 {
                        pawn_moves_bitboard = gen_piece(&piece_coordinates, None, &team_bitboards, false, board, pieces_info).moves_bitboard;
                    } else {
                        pawn_moves_bitboard = 0;
                    }

                    // Get checking piece attacks
                    let enemy_team_bitboards = crate::TeamBitboards {
                        friendly_team: team_bitboards.enemy_team,
                        enemy_team: team_bitboards.friendly_team,
                    };  
                    let checking_piece_attacks = gen_piece(&checking_piece, None, &enemy_team_bitboards, true, board, pieces_info).moves_bitboard;
                    
                    
                    for final_bit in 0..64 {
                        if bit_on(piece_attacks.moves_bitboard | pawn_moves_bitboard, final_bit) { // true if final_bit can be moved to
                            if use_checking_piece {
                                if bit_on(piece_attacks.moves_bitboard, checking_piece.bit) && final_bit == checking_piece.bit { // true if the checking piece can be captured
    
                                    // If the king is not in check after capturing the checking piece then it is not mate
                                    let mut team_bitboards = team_bitboards;
                                    team_bitboards.friendly_team ^= 1 << initial_bit | 1 << final_bit; // Move piece on friendly team bitboard
                                    team_bitboards.enemy_team ^= 1 << checking_piece.bit; // Remove captured piece from enemy teams bitboard
    
                                    let enemy_attacks = gen_enemy_attacks(&king, team_bitboards, board, pieces_info);
    
                                    // If the checking piece is the same as the original checking piece then there is no mate
                                    let mut new_checking_piece = None;
                                    for i in 0..enemy_attacks.checking_pieces_no {
                                        if enemy_attacks.checking_pieces[i] != Some(checking_piece) {
                                            new_checking_piece = enemy_attacks.checking_pieces[i];
                                        }
                                    }

                                    match new_checking_piece {
                                        Some(piece) => {
                                            // If there is a new checking piece then the move put the king into check, so it is still mate
                                            continue;
                                        },
                                        None => return false, // If there is no checking piece then the king is not in mate
                                    };
                                } else { // If the checking piece cannot be captured, check if its attack can be blocked
                                    if bit_on(checking_piece_attacks, final_bit) { // True if the friendly piece moved into the checking pieces path
                                        
                                        // If the current piece is a pawn and the the current move isn't also on it's move bitboard then the move isn't valid
                                        if board_index == 0 || board_index == 6 {
                                            if !bit_on(pawn_moves_bitboard, final_bit) {
                                                continue;
                                            }
                                        }

                                        // Update enemy teams bitboard
                                        let mut enemy_team_bitboards = enemy_team_bitboards;

                                        // Remove captured piece
                                        if bit_on(enemy_team_bitboards.friendly_team, final_bit) {
                                            enemy_team_bitboards.friendly_team ^= 1 << checking_piece.bit; 
                                        }
                                        
                                        enemy_team_bitboards.enemy_team ^= 1 << initial_bit | 1 << final_bit; // Move piece

                                        // Get checking piece attacks after the move
                                        let checking_piece_attacks = gen_piece(&checking_piece, None, &enemy_team_bitboards, true, board, pieces_info).moves_bitboard;
                                        
                                        // True if the original checking piece is no longer putting the king in check
                                        if !bit_on(checking_piece_attacks, king.bit) {
                                            let team_bitboards = crate::TeamBitboards {
                                                friendly_team: enemy_team_bitboards.enemy_team,
                                                enemy_team: enemy_team_bitboards.friendly_team,
                                            };

                                            let sliding_king = BoardCoordinates {
                                                board_index: 4,
                                                bit: king.bit,
                                            };

                                            // Get squares that could potentially be putting the king in check afer the move
                                            let king_check_squares = gen_piece(&sliding_king, None, &team_bitboards, false, board, pieces_info);
                                            
                                            // Loop through every king_check_square to check for enemy pieces
                                            // Generate moves from enemy pieces to see if any are putting the king in check

                                            let mut mate = false;
                                            for i in 0..64 {
                                                if bit_on(king_check_squares.moves_bitboard, i) && bit_on(team_bitboards.enemy_team, i) {
                                                    for j in 0..12 {
                                                        if bit_on(board.board[j], i) {

                                                            // Potential checking piece
                                                            let checking_piece = BoardCoordinates {
                                                                board_index: j,
                                                                bit: i,
                                                            };

                                                            let enemy_team_bitboards = crate::TeamBitboards {
                                                                friendly_team: team_bitboards.enemy_team,
                                                                enemy_team: team_bitboards.friendly_team,
                                                            };

                                                            // Get checking piece attacks
                                                            let checking_piece_attacks = gen_piece(&checking_piece, None, &enemy_team_bitboards, true, board, pieces_info);

                                                            // If the king is in check after the move then it is mate
                                                            if bit_on(checking_piece_attacks.moves_bitboard, king.bit) {
                                                                mate = true;
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            if !mate {
                                                return false;
                                            }
                                        }
                                    }
                                }
                            } else { // Check for stalemate
    
                                // If the any piece can move then it is not mate
                                if piece_attacks.moves_bitboard > 0 {
                                    return false;
                                }
                            }
                        }
                    }
                }
            }
        } else {
            return false;
        }

        true
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
    pub fn new_turn(
        piece: &board_representation::BoardCoordinates,
        piece_move_bit: usize,
        mut friendly_king: board_representation::BoardCoordinates,
        enemy_king: &board_representation::BoardCoordinates,
        enemy_attacks: &EnemyAttacks,
        mut team_bitboards: crate::TeamBitboards,
        mut board: board_representation::Board,
        pieces_info: &[crate::piece::constants::PieceInfo; 12]
    ) -> Result<board_representation::Board, TurnError> {
        use crate::TeamBitboards;
        use crate::board_index_white;

        // If the piece is a king generate castle moves
        let mut piece_moves = Moves::new();
        if piece == &friendly_king {
            piece_moves = castle(piece, piece_move_bit, &team_bitboards, enemy_attacks.enemy_attack_bitboard, &board);
        }

        // Get piece team
        let piece_white = board_index_white(piece.board_index);

        // If the piece is on the wrong team return an error
        if piece_white != board_index_white(friendly_king.board_index) {
            return Err(TurnError::InvalidMove);
        }

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
            None => piece_moves = gen_piece(piece, None, &team_bitboards, false, &board, pieces_info), // Gen non castle moves
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
        let piece_move_bitboard = 1 << piece_move_bit;
        let piece_move_xor_bitboard = 1 << piece.bit | piece_move_bitboard;
        team_bitboards.friendly_team ^= piece_move_xor_bitboard;
        board.board[12] |= piece_move_bitboard;

        // Promote pawns to queens if they are in the top row (for their respective team)
        // Give value for promoting pawns to queens
        let mut value = 0;
        if piece_white && piece.board_index == 0 && piece_move_bit < 8 {
            board.board[piece.board_index] ^= 1 << piece.bit;
            board.board[4] |= piece_move_bitboard;
            value += 8;
        } else if !piece_white && piece.board_index == 6 && piece_move_bit > 55 {
            board.board[piece.board_index] ^= 1 << piece.bit;
            board.board[10] |= piece_move_bitboard;
            value += 8;
        } else {
            board.board[piece.board_index] ^= piece_move_xor_bitboard; // Else move piece on its bitboard to the new coordinates
        }

        // Update friendly king bit if it was moved
        if piece.board_index == friendly_king.board_index {
            friendly_king.bit = piece_move_bit;
        }
        
        // If an enemy piece is captured get the value of the piece
        if crate::bit_on(team_bitboards.enemy_team, piece_move_bit) {
            team_bitboards.enemy_team ^= piece_move_bitboard; // Removed captured piece on enemy team bitboard

            // Get enemy board indexes
            let enemy_indexes;
            if piece_white {
                enemy_indexes = 6..12;
            } else {
                enemy_indexes = 0..6;
            }

            // If a piece was captured remove it on the appropriate enemy bitboard and store the captured pieces value
            for i in enemy_indexes {
                let new_piece_bitboard = board.board[i] ^ piece_move_bitboard ;
                if new_piece_bitboard < board.board[i] {
                    board.board[i] = new_piece_bitboard;
                    value = pieces_info[i].value;
                    break;
                }
            }
        }

        // If a piece was captured with en passant its value is 1
        // Don't set value to 1 if piece castled, because en_passant_capture_bit is also set when castled = true
        if !castled {
            match piece_moves.en_passant_capture_bit {
                Some(_) => value = 1,
                None => (),
            }
        }
        
        // Don't keep en passant target from the piece moves if the pawn didn't move 2 squares
        if (piece_move_bit as i8 - piece.bit as i8).abs() != 16 {
            board.en_passant_target = None;
        }

        // Increment half move clock if no capture was made and a pawn did not move, otherwise reset the half move clock
        if value == 0 && !(piece.board_index == 0 || piece.board_index == 6) {
            board.half_move_clock += 1;
        } else {
            board.half_move_clock = 0;
            team_bitboards.enemy_team ^= 1 << piece_move_bit; // Remove captured piece from enemy team bitboard
        }

        // Increment total half moves
        board.half_moves += 1;

        // If the king is in check after the move return an error
        let enemy_attacks = gen_enemy_attacks(&friendly_king, team_bitboards, &board, pieces_info);
        if enemy_attacks.checking_pieces_no != 0 {
            return Err(TurnError::InvalidMoveCheck);
        }

        // Generate necassary values to check if the enemy king has been mated
        let mut enemy_team_bitboards = TeamBitboards {
            friendly_team: team_bitboards.enemy_team,
            enemy_team: team_bitboards.friendly_team,
        };

        // If a piece was captured during the move remove it from the team bitboard
        if crate::bit_on(enemy_team_bitboards.friendly_team, piece_move_bit) {
            enemy_team_bitboards.friendly_team ^= piece_move_bitboard;
        }

        // The team to move is now opposite
        board.whites_move = !board.whites_move;

        let friendly_attacks = gen_enemy_attacks(enemy_king, enemy_team_bitboards, &board, pieces_info); // Get friendly attacks to use as enemy attacks for enemy king
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
        board.points_delta = value;

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
}
