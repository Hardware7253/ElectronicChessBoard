use std::io;
use chess2::board::board_representation;
use chess2::algorithm::Move;
use chess2::board::move_generator;
use chess2::board::move_generator::TurnError;

fn main() {

    let board = board_representation::fen_decode("P2PPPPP/PP1PPPPP/8/1P3P2/8/8/8/8 b - - 0 1", true);
    println!("{:?}", board.board);

    let mut bug_board = board_representation::fen_decode("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", true);
    bug_board.board = [13828032701116865358, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    //bug_board.board = [65020719754379264, 9295429630892703744, 4398314946560, 288230376185266176, 576460752303423488, 1152921504606846976, 61184, 129, 8592031744, 536870944, 8, 16, 9086293723196622974];

    let bug_board_fen_encode = board_representation::fen_encode(&bug_board);
    println!("{}", bug_board_fen_encode);

    // Get team of the player
    print!("Player White? ");
    let player_white = get_user_bool(true);
    println!("");
    
    let mut board = board_representation::fen_decode("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", true); // Initialise board with starting position

    let pieces_info = chess2::piece::constants::gen();

    loop {
        println!("Turn start, Whites turn: {}", board.whites_move);
        println!("{:?}", board);
        println!("");

        if board.turns_since_capture == 50 {
            println!("Draw, too many moves since last capture");
            break;
        }

        let player_turn = player_white == board.whites_move;

        let mut piece_move = Move::new();

        // Get Turn from player / ai
        if player_turn {

            // Get user initial and final bit for the move
            piece_move.initial_piece_coordinates.bit = get_user_bit("Move piece from coordinates:");
            piece_move.final_piece_bit = get_user_bit("To new coordinates:");

            // Flip bitboard piece bits to the white teams perspective if the player is black team
            if !player_white {
                piece_move.initial_piece_coordinates.bit = chess2::flip_bitboard_bit(piece_move.initial_piece_coordinates.bit);
                piece_move.final_piece_bit = chess2::flip_bitboard_bit(piece_move.final_piece_bit);
            }

            // Get board index for piece
            for i in 0..12 {
                if chess2::bit_on(board.board[i], piece_move.initial_piece_coordinates.bit) {
                    piece_move.initial_piece_coordinates.board_index = i;
                }
            }
        } else {
            // Get ai piece move if it is not the players turn
            piece_move = chess2::algorithm::gen_best_move(true, 6, 0, 0, None, board, &pieces_info);
        }

        // Get friendly and enemy kings
        let friendly_king_index;
        let enemy_king_index;
        if board.whites_move {
            friendly_king_index = 5;
            enemy_king_index = 11;
        } else {
            friendly_king_index = 11;
            enemy_king_index = 5;
        }

        let friendly_king = board_representation::BoardCoordinates {
            board_index: friendly_king_index,
            bit: chess2::find_bit_on(board.board[friendly_king_index], 0),
        };

        let enemy_king = board_representation::BoardCoordinates {
            board_index: enemy_king_index,
            bit: chess2::find_bit_on(board.board[enemy_king_index], 0),
        };

        // Get new_turn_board
        let team_bitboards = chess2::TeamBitboards::new(friendly_king.board_index, &board);
        let enemy_attacks = move_generator::gen_enemy_attacks(&friendly_king, team_bitboards, &board, &pieces_info);
        let new_turn_board = move_generator::new_turn(&piece_move.initial_piece_coordinates, piece_move.final_piece_bit, friendly_king, &enemy_king, &enemy_attacks, team_bitboards, board, &pieces_info);

        let mut initial_bit = piece_move.initial_piece_coordinates.bit;
        let mut final_bit = piece_move.final_piece_bit;
        
        // Flip bitboard bits back to the right perspective if the player is on the black team
        if !player_white {
            initial_bit = chess2::flip_bitboard_bit(initial_bit);
            final_bit = chess2::flip_bitboard_bit(final_bit);
        }

        // Convert moves to ccn
        let initial_ccn = chess2::bit_to_ccn(initial_bit);
        let final_ccn = chess2::bit_to_ccn(final_bit);

        match new_turn_board {
            Ok(new_board) => {
                if !player_turn { // If ai move        
                    println!("Ai moves from {}, to {}", initial_ccn, final_ccn);
                    println!("");
                }

                // Update board
                board = new_board
            },
            Err(error) => { // End the game, or let the game continue depending on the error
                match error {
                    TurnError::Win => {

                        if !player_turn { // If ai move        
                            println!("Ai moves from {}, to {}", initial_ccn, final_ccn);
                            println!("");
                        }

                        if board.whites_move {
                            println!("White team wins");
                        } else {
                            println!("Black team wins");
                        }
                        break
                    }
                    TurnError::Draw => {println!("Game over, draw"); break},
                    TurnError::InvalidMove => {println!("Invalid move, try again"); continue}
                    TurnError::InvalidMoveCheck => {println!("Invalid move, the king is in check"); continue}
                }
            },
        }
    }
}

// Get ccn input from user (a2, g4, etc) and convert it into a bitboard bit
// Loop until the user inputs a value with the correct formatting
fn get_user_bit(message: &str) -> usize {
    let bit = loop {
        println!("{}", message);

        // Get user input as a string
        let mut ccn = String::new();
        io::stdin()
            .read_line(&mut ccn)
            .expect("Failed to read input");

        let bit = chess2::ccn_to_bit(&ccn);

        // If the chess_to_engine function fails, provide an error and allow the user to try again
        let bit = match bit {
            Ok(c) => c,
            Err(_) => {
                println!("Please use correct coordinate formatting (E.g. a1). This is case sensetive.");
                println!();
                continue;
            },
        };

        println!();
        break bit;
    };

    bit
}

// Get y/n bool input
fn get_user_bool(default_yes: bool) -> bool {
    if default_yes {
        println!("[Y, n]");
    } else {
        println!("[y, N]");
    }
    
    // Get user input as a string
    let mut yn = String::new();
    io::stdin()
        .read_line(&mut yn)
        .expect("Failed to read input");

    // Get first character of the input string
    let yn: Vec<char> = yn.chars().collect();
    let yn = yn[0];

    if default_yes {
        if yn == 'n' || yn == 'N' {
            return false;
        }
        return true;
    }

    if yn == 'y' || yn == 'Y' {
        return true;
    }
    false
}