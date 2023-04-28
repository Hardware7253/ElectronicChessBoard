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
        pub half_moves: i16, // The total number of half moves
        pub half_move_clock: i16, // The number of half moves since the last capture or pawn move
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
                half_moves: 0,
                half_move_clock: 0,
                en_passant_target: None,
            }
        }
    }

    // Decodes a fen string into engine board format
    pub fn fen_decode(fen: &str, master: bool) -> Board {
        let fen_vec: Vec<char> = fen.chars().collect();

        let mut spaces = 0;
        let mut ccn = String::new();

        let mut board = Board::new();
        board.board[12] = u64::MAX;

        let mut piece_square = 0;
        let mut white_king_moved = true;
        let mut black_king_moved = true;
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
            }

            // Set en passant conditions
            if spaces == 3 && fen_char != ' '{
                ccn.push(fen_char);
                match crate::ccn_to_bit(&ccn) {
                    Ok(bit) =>  board.en_passant_target = Some(bit),
                    Err(()) => (),
                }
            }

            // Set turns since last capture
            if spaces == 4 {
                board.half_move_clock *= 10;
                board.half_move_clock += char_num as i16;
            }

            // Ignore the rest of the fen code
        }

        // Set white / black king turns to 0
        if !white_king_moved {
            board.board[12] ^= 1 << 60;
        }
        if !black_king_moved {
            board.board[12] ^= 1 << 4;
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

    // Takes a board and encodes it into a fen string
    // Castling conditions, en passant target, etc is not added to the resuling fen string
    pub fn fen_encode(board: &Board) -> String {
        let char_ids = ['P', 'R', 'N', 'B', 'Q', 'K', 'p', 'r', 'n', 'b', 'q', 'k'];
        let mut char_board = ['0'; 64];

        let mut fen_string = String::new();

        // Add all char ids to a board array
        for i in 0..char_ids.len() {
            for j in 0..char_board.len() {
                if crate::bit_on(board.board[i], j) {
                    char_board[j] = char_ids[i];
                }
            }
        }

        // Turn board array into fen
        let mut empty_square_count = 0;
        for i in 0..char_board.len() {
            let current_char = char_board[i];

            let row_end = (i as isize - 7) % 8 == 0;

            if current_char == '0' {
                empty_square_count += 1; // Increment empty squares
            } else {
                
                // If there are empty squares before the piece add them to the fen string
                if empty_square_count > 0 {
                    fen_string.push_str(&crate::num_to_char(empty_square_count).unwrap().to_string());
                    empty_square_count = 0;
                }
                fen_string.push_str(&current_char.to_string());
            }

            // Add a / to seperate rows once the end of the row is reached
            if row_end {

                // Add empty square count if necassary
                if empty_square_count > 0 {
                    fen_string.push_str(&crate::num_to_char(empty_square_count).unwrap().to_string());
                }

                if i != char_board.len() - 1 { // Don't add / on the end of the fen string
                    fen_string.push_str(&'/'.to_string());
                    empty_square_count = 0;
                }
            }
        }

        fen_string
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
            expected.half_move_clock = 50;


            assert_eq!(board, expected);
        }

        // Reliant on fen_decode working
        #[test]
        fn fen_encode_test() {
            let board = fen_decode("rn1qkbnr/pp2pppp/2p5/5b2/3PN3/8/PPP2PPP/R1BQKBNR w KQkq - 0 1", true);

            let result = fen_encode(&board);
            let expected = String::from("rn1qkbnr/pp2pppp/2p5/5b2/3PN3/8/PPP2PPP/R1BQKBNR");

            assert_eq!(result, expected);
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
        team_bitboards: crate::TeamBitboards,
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
            moves = gen_pawn_captures(&piece, only_gen_attacks, team_bitboards, &board);
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
                let piece_moves = gen_piece(&board_coordinates, Some(king), team_bitboards, true, board, pieces_info);
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
        use crate::common_bit;
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
                    let piece_attacks = gen_piece(&piece_coordinates, None, team_bitboards, use_checking_piece, board, pieces_info);

                    // Pawn moves shouldn't be included in the attacks bitboard
                    // But they are still necasary for checking if the piece can block a checking pieces path
                    let pawn_moves_bitboard;
                    if board_index == 0 || board_index == 6 {
                        pawn_moves_bitboard = gen_piece(&piece_coordinates, None, team_bitboards, false, board, pieces_info).moves_bitboard;
                    } else {
                        pawn_moves_bitboard = 0;
                    }

                    // Get checking piece attacks
                    let enemy_team_bitboards = crate::TeamBitboards {
                        friendly_team: team_bitboards.enemy_team,
                        enemy_team: team_bitboards.friendly_team,
                    };  
                    let checking_piece_attacks = gen_piece(&checking_piece, None, enemy_team_bitboards, true, board, pieces_info).moves_bitboard;
                    
                    
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
                                        let checking_piece_attacks = gen_piece(&checking_piece, None, enemy_team_bitboards, true, board, pieces_info).moves_bitboard;
                                        
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
                                            let king_check_squares = gen_piece(&sliding_king, None, team_bitboards, false, board, pieces_info);

                                            //println!("{:?}", team_bitboards);
                                            //println!("{:?}", king_check_squares);
                                            
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

                                                            println!("{}, {}", initial_bit, final_bit);
                                                            println!("{:?}", checking_piece);

                                                            let enemy_team_bitboards = crate::TeamBitboards {
                                                                friendly_team: team_bitboards.enemy_team,
                                                                enemy_team: team_bitboards.friendly_team,
                                                            };

                                                            // Get checking piece attacks
                                                            let checking_piece_attacks = gen_piece(&checking_piece, None, enemy_team_bitboards, true, board, pieces_info);

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
            None => piece_moves = gen_piece(piece, None, team_bitboards, false, &board, pieces_info), // Gen non castle moves
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

    #[cfg(test)]
    mod tests {
        use board_representation::BoardCoordinates;
        use board_representation::fen_decode;
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

            let result = gen_piece(&piece, None, team_bitboards, false, &board, &pieces_info);

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

            let result = gen_piece(&piece, None, team_bitboards, false, &board, &pieces_info);

            let expected: u64 = 8830452760576;
            
            assert_eq!(result.moves_bitboard, expected);
        }

        #[test]
        fn gen_pawn_captures_test1() {
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

            let board = fen_decode("r3k2r/8/8/8/8/8/8/8 b kq - 0 1", true);

            let king = BoardCoordinates {
                board_index: 11,
                bit: 4,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let expected = Moves {
                moves_bitboard: 1 << 2,
                en_passant_target_bit: Some(3),
                en_passant_capture_bit: Some(0),
            };
            
            assert_eq!(castle(&king, 2, &team_bitboards, 0, &board), expected); // Valid castle
            assert_eq!(castle(&king, 2, &team_bitboards, 4, &board), Moves::new()); // Test an enemy attack blocking the path
        }

        #[test]
        fn gen_enemy_attacks_test1() {
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
                enemy_attack_bitboard: 4621350219176104832,
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
        fn gen_enemy_attacks_test3() {
            use crate::TeamBitboards;

            let board = fen_decode("7k/8/2P5/8/8/5B2/8/8 b - - 0 1", true);

            let king = BoardCoordinates {
                board_index: 11,
                bit: 7,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let result = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let checking_pieces: [Option<BoardCoordinates>; 2] = [None, None];

            let expected = EnemyAttacks {
                enemy_attack_bitboard: 9822351133174401536,
                checking_pieces: checking_pieces,
                checking_pieces_no: 0,
            };
            
            assert_eq!(result, expected);
        }

        #[test]
        fn king_can_move_test1() {
            use crate::TeamBitboards;
            
            let board = fen_decode("6R1/8/8/8/6N1/7k/6P1/4BP2 w - - 0 1", true);

            let king = BoardCoordinates {
                board_index: 11,
                bit: 47,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let enemy_attacks = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let result = king_can_move(&king, &enemy_attacks, team_bitboards, &pieces_info);
            
            assert_eq!(result, false);
        }

        #[test]
        fn king_can_move_test2() {
            use crate::TeamBitboards;
            
            let board = fen_decode("r1b2knr/4b3/2n1p2p/pNP2p1p/8/4BN2/PP3PqP/R2Q1RK1 b - - 0 1", true);

            let king = BoardCoordinates {
                board_index: 5,
                bit: 62,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let enemy_attacks = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let result = king_can_move(&king, &enemy_attacks, team_bitboards, &pieces_info);
            
            assert_eq!(result, true);
        }

        #[test]
        fn is_mate_test1() { // Test check mate
            use crate::TeamBitboards;
            
            let board = fen_decode("6R1/8/8/8/6N1/7k/6P1/4BP2 b - - 0 1", true);

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
        fn is_mate_test4() { // Test check mate
            use crate::TeamBitboards;

            let board = fen_decode("r1K4k/7r/8/8/8/8/2R5/8 w - - 0 1", true);

            let king = BoardCoordinates {
                board_index: 5,
                bit: 2,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let enemy_attacks = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let result = is_mate(&king, &enemy_attacks, team_bitboards, &board, &pieces_info);
            
            assert_eq!(result, true);
        }

        #[test]
        fn is_mate_test5() {
            use crate::TeamBitboards;

            let board = fen_decode("rn1qkbnr/pp2pppp/2p2N2/5b2/3P4/8/PPP2PPP/R1BQKBNR b KQkq - 0 1", true);

            let king = BoardCoordinates {
                board_index: 11,
                bit: 4,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let enemy_attacks = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let result = is_mate(&king, &enemy_attacks, team_bitboards, &board, &pieces_info);
            
            assert_eq!(result, false);
        }

        #[test]
        fn is_mate_test6() { // Test check mate
            use crate::TeamBitboards;

            let board = fen_decode("r2qkbnr/pp1npppp/3N4/1p3b2/3P4/8/PPP1QPPP/R1B1K1NR b - - 0 1", true);

            let king = BoardCoordinates {
                board_index: 11,
                bit: 4,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let enemy_attacks = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let result = is_mate(&king, &enemy_attacks, team_bitboards, &board, &pieces_info);
            
            assert_eq!(result, true);
        }

        #[test]
        fn is_mate_test7() {
            use crate::TeamBitboards; // Test friendly piece blocking the checking pieces path

            let board = fen_decode("r2qkbnr/pp1n1ppp/3p4/1p3b2/3P4/8/PPP1QPPP/R1B1K1NR b - - 0 1", true);

            let king = BoardCoordinates {
                board_index: 11,
                bit: 4,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let enemy_attacks = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let result = is_mate(&king, &enemy_attacks, team_bitboards, &board, &pieces_info);
            
            assert_eq!(result, false);
        }

        #[test]
        fn is_mate_test8() { // Test a no mate where a king is able to capture the checking piece
            use crate::TeamBitboards;

            let board = fen_decode("r1b2knr/4b3/2n1p2p/pNP2p1p/8/4BN2/PP3PqP/R2Q1RK1 w - - 0 1", true);

            let king = BoardCoordinates {
                board_index: 5,
                bit: 62,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let enemy_attacks = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let result = is_mate(&king, &enemy_attacks, team_bitboards, &board, &pieces_info);
            
            assert_eq!(result, false);
        }

        #[test]
        fn is_mate_test9() { // Test king not being in mate
            use crate::TeamBitboards;

            let board = fen_decode("2n2bR1/2k5/7p/p1p1p3/2p1P1P1/8/PPP2PKP/6r1 w - - 0 1", true);

            let king = BoardCoordinates {
                board_index: 5,
                bit: 54,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let enemy_attacks = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let result = is_mate(&king, &enemy_attacks, team_bitboards, &board, &pieces_info);
            
            assert_eq!(result, false);
        }

        #[test]
        fn is_mate_test10() {
            use crate::TeamBitboards;

            let board = fen_decode("1n2k2r/8/2p3pb/1B1pP2p/6bP/2P2NPn/3q4/Q2K4 w - - 0 1", true);

            let king = BoardCoordinates {
                board_index: 5,
                bit: 59,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let enemy_attacks = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let result = is_mate(&king, &enemy_attacks, team_bitboards, &board, &pieces_info);
            
            assert_eq!(result, true);
        }

        #[test]
        fn is_mate_test11() {
            use crate::TeamBitboards;

            let board = fen_decode("1n2k2r/8/2p3pb/1p1pP2p/6bP/2P3Pn/4q2N/1QK5 w - - 0 1", true);

            let king = BoardCoordinates {
                board_index: 5,
                bit: 58,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let enemy_attacks = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let result = is_mate(&king, &enemy_attacks, team_bitboards, &board, &pieces_info);
            
            assert_eq!(result, true);
        }

        #[test]
        fn is_mate_test12() {
            use crate::TeamBitboards;

            let board = fen_decode("r3kb1r/1p1Ppp2/pB4p1/3R3p/8/8/P1q1QPPP/5RK1 b kq - 0 1", true);

            let king = BoardCoordinates {
                board_index: 11,
                bit: 4,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let enemy_attacks = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let result = is_mate(&king, &enemy_attacks, team_bitboards, &board, &pieces_info);
            
            assert_eq!(result, true);
        }

        #[test]
        fn is_mate_test13() {
            use crate::TeamBitboards;

            let board = fen_decode("r1bkq3/4bB2/1Bp5/1p1nN2p/1P6/2N1R3/2PQ1PPP/5RK1 b - - 0 1", true);

            let king = BoardCoordinates {
                board_index: 11,
                bit: 3,
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

            let mut expected_board = fen_decode("4k3/8/8/1N3B2/8/8/8/K7 b - - 0 1", true);

            let new_turn_board = new_turn(&piece, 29, king, &enemy_king, &enemy_attacks, team_bitboards, board, &pieces_info);

        
            expected_board.points.white_points = 9;
            expected_board.points_delta = 9;
            expected_board.board[12] |= 1 << 29;
            expected_board.half_moves = 1;

            assert_eq!(new_turn_board, Ok(expected_board));            
        }

        #[test]
        fn new_turn_test2() { // Test moving a piece to win a game with checkmate
            use crate::TeamBitboards;

            let board = fen_decode("k7/7Q/8/2N5/8/8/8/7K w - - 0 1", true);

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
                bit: 15,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let enemy_attacks = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let new_turn_board = new_turn(&piece, 9, king, &enemy_king, &enemy_attacks, team_bitboards, board, &pieces_info);

            assert_eq!(new_turn_board, Err(TurnError::Win));            
        }

        #[test]
        fn new_turn_test3() { // Test moving out of check
            use crate::TeamBitboards;

            let board = fen_decode("7k/2K5/8/8/8/r2r4/7R/8 b - - 0 1", true);

            let enemy_king = BoardCoordinates {
                board_index: 5,
                bit: 10,
            };

            let king = BoardCoordinates {
                board_index: 11,
                bit: 7,
            };

            let piece = BoardCoordinates {
                board_index: 11,
                bit: 7,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let enemy_attacks = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let mut expected_board = fen_decode("6k1/2K5/8/8/8/r2r4/7R/8 w - - 0 1", true);

            let new_turn_board = new_turn(&piece, 6, king, &enemy_king, &enemy_attacks, team_bitboards, board, &pieces_info);

            expected_board.board[12] |= 1 << 6;
            expected_board.half_move_clock += 1;
            expected_board.half_moves = 1;

            assert_eq!(new_turn_board, Ok(expected_board));            
        }

        #[test]
        fn new_turn_test4() { // Test getting an error not moving out of check
            use crate::TeamBitboards;

            let board = fen_decode("8/2K5/8/8/8/2rR4/8/7k w - - 0 1", true);

            let king = BoardCoordinates {
                board_index: 5,
                bit: 10,
            };

            let enemy_king = BoardCoordinates {
                board_index: 11,
                bit: 63,
            };

            let piece = BoardCoordinates {
                board_index: 5,
                bit: 10,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let enemy_attacks = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let new_turn_board = new_turn(&piece, 2, king, &enemy_king, &enemy_attacks, team_bitboards, board, &pieces_info);

            assert_eq!(new_turn_board, Err(TurnError::InvalidMoveCheck));            
        }

        #[test]
        fn new_turn_test5() { // Test moving a piece to win a game with checkmate
            use crate::TeamBitboards;

            let board = fen_decode("r1b1k1nr/ppppqppp/8/P1b1n3/3N4/8/1PP2PPP/RNBQKB1R b - - 0 1", true);

            let king = BoardCoordinates {
                board_index: 11,
                bit: 4,
            };

            let enemy_king = BoardCoordinates {
                board_index: 5,
                bit: 60,
            };

            let piece = BoardCoordinates {
                board_index: 8,
                bit: 28,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let enemy_attacks = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let new_turn_board = new_turn(&piece, 45, king, &enemy_king, &enemy_attacks, team_bitboards, board, &pieces_info);

            assert_eq!(new_turn_board, Err(TurnError::Win));            
        }

        #[test]
        fn new_turn_test6() { // Test pawn not being able to capture another pawn when moving forwards
            use crate::TeamBitboards;

            let board = fen_decode("rnbqkbnr/ppp1pppp/8/3p4/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1", true);

            let king = BoardCoordinates {
                board_index: 5,
                bit: 60,
            };

            let enemy_king = BoardCoordinates {
                board_index: 11,
                bit: 4,
            };

            let piece = BoardCoordinates {
                board_index: 0,
                bit: 35,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let enemy_attacks = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let new_turn_board = new_turn(&piece, 27, king, &enemy_king, &enemy_attacks, team_bitboards, board, &pieces_info);

            assert_eq!(new_turn_board, Err(TurnError::InvalidMove));            
        }

        #[test]
        fn new_turn_test7() { // Test pawn invalid move
            use crate::TeamBitboards;

            let board = fen_decode("r2qkb1r/ppp1pppp/5n2/1B1PNb2/1n6/2N5/PPP2PPP/R1BQK2R b KQkq - 0 1", true);

            let king = BoardCoordinates {
                board_index: 11,
                bit: 4,
            };

            let enemy_king = BoardCoordinates {
                board_index: 5,
                bit: 60,
            };

            let piece = BoardCoordinates {
                board_index: 6,
                bit: 12,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let enemy_attacks = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let new_turn_board = new_turn(&piece, 11, king, &enemy_king, &enemy_attacks, team_bitboards, board, &pieces_info);

            assert_eq!(new_turn_board, Err(TurnError::InvalidMove));            
        }

        #[test]
        fn new_turn_test8() {
            use crate::TeamBitboards;

            let board = fen_decode("rnbqkbnr/pppppppp/8/8/P7/8/1PPPPPPP/RNBQKBNR b - - 0 1", true);

            let king = BoardCoordinates {
                board_index: 11,
                bit: 4,
            };

            let enemy_king = BoardCoordinates {
                board_index: 5,
                bit: 60,
            };

            let piece = BoardCoordinates {
                board_index: 6,
                bit: 12,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let enemy_attacks = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let new_turn_board = new_turn(&piece, 28, king, &enemy_king, &enemy_attacks, team_bitboards, board, &pieces_info);

            let mut expected_board = fen_decode("rnbqkbnr/pppp1ppp/8/4p3/P7/8/1PPPPPPP/RNBQKBNR w - - 0 1", true);
            expected_board.board[12] = 18375249429624979711;
            expected_board.en_passant_target = Some(20);
            expected_board.half_moves = 1;

            assert_eq!(new_turn_board, Ok(expected_board));            
        }

        #[test]
        fn new_turn_test9() {
            use crate::TeamBitboards;

            let board = fen_decode("r1b2knr/4b1q1/2n1p2p/pNP2p1p/8/4BN2/PP3PPP/R2Q1RK1 b - - 0 1", true);

            let king = BoardCoordinates {
                board_index: 11,
                bit: 5,
            };

            let enemy_king = BoardCoordinates {
                board_index: 5,
                bit: 62,
            };

            let piece = BoardCoordinates {
                board_index: 10,
                bit: 14,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let enemy_attacks = gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let new_turn_board = new_turn(&piece, 54, king, &enemy_king, &enemy_attacks, team_bitboards, board, &pieces_info);

            let mut expected_board = fen_decode("r1b2knr/4b3/2n1p2p/pNP2p1p/8/4BN2/PP3PqP/R2Q1RK1 w - - 0 1", true);
            expected_board.board[12] |= 1 << 54;
            expected_board.points_delta = 1;
            expected_board.points.black_points = 1;
            expected_board.half_moves = 1;

            assert_eq!(new_turn_board, Ok(expected_board));            
        }
    }
}
