use crate::board::board_representation;
use crate::board::move_generator::EnemyAttacks;
use crate::TeamBitboards;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Move {
    pub initial_piece_coordinates: board_representation::BoardCoordinates,
    pub final_piece_bit: usize,
    pub value: i8,
}

impl Move {
    pub fn new() -> Self {
        Move {
            initial_piece_coordinates: board_representation::BoardCoordinates {
                board_index: 0,
                bit: 0,
            },
            final_piece_bit: 0,
            value: 0,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
struct MinMax {
    max_move: Option<Move>,
    min_value: Option<i8>,
}

// Update MinMax struct if new move has a value lesser or greater than min/max fields
// Initialize MinMax if it hasn't been allready
fn update_min_max(piece_move: Move, mut min_max: MinMax) -> MinMax {
    match min_max.max_move {
        Some(_) => (),
        None => {

            // If min_max has not yet been initialized then initialize it with piece_move
            return MinMax {
                max_move: Some(piece_move),
                min_value: Some(piece_move.value),
            };
        },
    }

    let max_value = min_max.max_move.unwrap().value;
    let min_value = min_max.min_value.unwrap();

    if piece_move.value > max_value {
        min_max.max_move = Some(piece_move);
    } else if piece_move.value < min_value {
        min_max.min_value = Some(piece_move.value);
    }

    min_max
}

fn update_prune_value(master_team: bool, min_max: &MinMax) -> Option<i8> {
    if master_team {
        match min_max.max_move {
            Some(max_move) => return Some(max_move.value),
            None => return None,
        }
    } else {
        return min_max.min_value;
    }
}

pub fn gen_best_move(master_team: bool, search_depth: usize, current_depth: usize, init_value: i8, parent_value: Option<i8>, board: board_representation::Board, pieces_info: &[crate::piece::constants::PieceInfo; 12]) -> Move {
    use crate::board::move_generator;
    use crate::board::move_generator::TurnError;

    let mut empty_move = Move::new();

    // If current depth and search depth are equal stop searching down the move tree
    if current_depth == search_depth {
        empty_move.value = init_value;
        return empty_move;
    }

    // Get friendly and enemy team BoardCoordinates
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
        bit: crate::find_bit_on(board.board[friendly_king_index], 0),
    };

    let enemy_king = board_representation::BoardCoordinates {
        board_index: enemy_king_index,
        bit: crate::find_bit_on(board.board[enemy_king_index], 0),
    };
    
    // Generate team bitboards
    let team_bitboards = TeamBitboards::new(friendly_king_index, &board);

    // Generate enemy attacks
    let enemy_attacks = move_generator::gen_enemy_attacks(&friendly_king, team_bitboards, &board, pieces_info);

    // Generate moves
    let moves = &order_moves(true, &board, &enemy_attacks, &friendly_king, team_bitboards, pieces_info);

    let mut min_max = MinMax {
        max_move: None,
        min_value: None,
    };

    let mut prune_value: Option<i8> = None;

    for i in 0..moves.len() {
        let moves_vec = &moves[i];
        for j in 0..moves_vec.len() {
            let initial_piece_coordinates = moves_vec[j].initial_piece_coordinates;
            let final_piece_bit = moves_vec[j].final_piece_bit;

            let new_turn_board = move_generator::new_turn(&initial_piece_coordinates, final_piece_bit, friendly_king, &enemy_king, &enemy_attacks, team_bitboards, board, &pieces_info);
            
            match new_turn_board {

                // Only continue searching down the move tree if the move didn't result in an invalid move or the end of the game
                Ok(new_board) => {
                    let mut move_value = new_board.points_delta;
                    
                    // If the current branch is not the master team then it's move values are negative (because they negatively impact the master team)
                    if !master_team {
                        move_value *= -1;
                    }

                    let branch_value = init_value + move_value;

                    let piece_move = gen_best_move(!master_team, search_depth, current_depth + 1, branch_value, prune_value, new_board, pieces_info);
                    let piece_move = Move {
                        initial_piece_coordinates: initial_piece_coordinates,
                        final_piece_bit: final_piece_bit,
                        value: piece_move.value,
                    };
                    
                    min_max = update_min_max(piece_move, min_max);
                    prune_value = update_prune_value(master_team, &min_max);
                },
                Err(error) => {

                    // Update min_max with value of game ending if the game ended
                    let mut branch_value;
                    let valid_move;

                    match error {
                        TurnError::Win => {branch_value = 127; valid_move = true},
                        TurnError::Draw => {branch_value = 0; valid_move = true},
                        TurnError::InvalidMove => {branch_value = 0; valid_move = false},
                        TurnError::InvalidMoveCheck => {branch_value = 0; valid_move = false},
                    }

                    // If the current branch is not the master team then it's move values are negative (because they negatively impact the master team)
                    if !master_team {
                        branch_value *= -1;
                    }

                    if valid_move {
                        let piece_move = Move {
                            initial_piece_coordinates: initial_piece_coordinates,
                            final_piece_bit: final_piece_bit,
                            value: branch_value,
                        };

                        min_max = update_min_max(piece_move, min_max);
                        prune_value = update_prune_value(master_team, &min_max);
                    }

                    continue;
                },
            }
        }
        

        // Alpha beta pruning
        match parent_value {
            Some(value) => {
                if master_team {
                    match min_max.max_move {
                        Some(max_move) => {
                            if max_move.value >= value {
                                break;
                            }
                        },
                        None => (),
                    }
                } else {
                    match min_max.min_value {
                        Some(min_value) => {
                            if min_value <= value {
                                break;
                            }
                        },
                        None => (),
                    }
                }
            },
            None => ()
        }
    }

    // Return min/max values depending on the team
    if master_team {
        return min_max.max_move.unwrap();
    } else {
        //println!("{:?}", board);
        //println!("{:?}", );
        empty_move.value = min_max.min_value.unwrap();
        return empty_move;
    }
}

// Returns a vec with potential moves
// If sort is true the moves will be ordered from best to worst
// All moves are valid apart from king moves
fn order_moves(sort: bool, board: &board_representation::Board, enemy_attacks: &EnemyAttacks, friendly_king: &board_representation::BoardCoordinates, team_bitboards: crate::TeamBitboards, pieces_info: &[crate::piece::constants::PieceInfo; 12]) -> [Vec<Move>; 19] {
    use crate::bit_on;
    
    const VEC_NEW: Vec<Move> = Vec::new();
    let mut moves: [Vec<Move>; 19] = [VEC_NEW; 19];

    // Get friendly and enemy board indexes
    let friendly_indexes;
    let enemy_index_bottom; // Inclusive
    let enemy_index_top; // Not inclusive
    if board.whites_move {
        friendly_indexes = 0..6;
        enemy_index_bottom = 6;
        enemy_index_top = 12;
    } else {
        friendly_indexes = 6..12;
        enemy_index_bottom = 0;
        enemy_index_top = 6;
    }

    for i in friendly_indexes {
        let piece_value = pieces_info[i].value;

        for initial_bit in 0..64 {
            let initial_piece_coordinates = board_representation::BoardCoordinates {
                board_index: i,
                bit: initial_bit,
            };

            // If there is no piece on the board at this bit got to the next bit
            if !bit_on(board.board[i], initial_bit) {
                continue;
            }

            let piece_moves = crate::board::move_generator::gen_piece(&initial_piece_coordinates, None, team_bitboards, false, board, pieces_info);
            
            for final_bit in 0..64 {

                // Check the piece can move to final_bit or piece is a king
                // Because this function does not account for castling those moves cannot be ruled out for the king
                if bit_on(piece_moves.moves_bitboard, final_bit) {
                    
                    // Get value of move based on value of captured piece
                    let mut move_value = 0;
                    if bit_on(team_bitboards.enemy_team, final_bit) { // If an enemy piece is in the same bit as the friendly pieces final_bit then it has been captured

                        for j in enemy_index_bottom..enemy_index_top {
                            if bit_on(board.board[j], final_bit) {
                                let capture_value = pieces_info[j].value;
    
                                // If an enemy can move to the captured square there will likely be a trade
                                if bit_on(enemy_attacks.enemy_attack_bitboard, final_bit) {
                                    move_value = piece_value - capture_value;
                                } else { // If an enemy can't move to the captured square then the friendly team gets the entire value of the captured piece
                                    move_value = capture_value;
                                }
    
                                // Once the piece that has been captured is found break the loop
                                break;
                            }
                        }
                    }

                    // Push move to moves vec
                    moves[(move_value - 7).abs() as usize].push(Move {
                        initial_piece_coordinates: initial_piece_coordinates,
                        final_piece_bit: final_bit,
                        value: 0,
                    });
                } else if &initial_piece_coordinates == friendly_king { // Add potentially invalid king moves to moves vec to account for castling
                    moves[7].push(Move {
                        initial_piece_coordinates: initial_piece_coordinates,
                        final_piece_bit: final_bit,
                        value: 0,
                    });
                }
            }
        }
    }

    // Sort moves and return
    if sort {
        //moves.sort_by(|a, b| b.value.cmp(&a.value));
    }
    moves
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_moves_test() {
        use crate::board::board_representation;
        use crate::board::move_generator;

        let board = board_representation::fen_decode("k7/8/8/8/4r3/3P4/8/7K w - - 0 1", true);

        let king = board_representation::BoardCoordinates {
            board_index: 5,
            bit: 63,
        };

        let team_bitboards = TeamBitboards::new(king.board_index, &board);

        let pieces_info = crate::piece::constants::gen();

        let enemy_attacks = move_generator::gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

        let result = order_moves(true, &board, &enemy_attacks, &king, team_bitboards, &pieces_info);

        let best_move = Move {
            initial_piece_coordinates: board_representation::BoardCoordinates {
                board_index: 0,
                bit: 43,
            },
            final_piece_bit: 36,
            value: 0,
        };

        assert_eq!(result[2][0], best_move);
    }

    #[test]
    fn update_min_max_test() {
        use crate::board::board_representation;

        let max_move = Move {
            initial_piece_coordinates: board_representation::BoardCoordinates {
                board_index: 0,
                bit: 43,
            },
            final_piece_bit: 36,
            value: 3,
        };

        let piece_move = Move {
            initial_piece_coordinates: board_representation::BoardCoordinates {
                board_index: 0,
                bit: 0,
            },
            final_piece_bit: 0,
            value: 5,
        };

        let min_max = MinMax {
            max_move: None,
            min_value: None,
        };

        let min_max = update_min_max(max_move, min_max);
        let min_max = update_min_max(piece_move, min_max);

        let expected = MinMax {
            max_move: Some(piece_move),
            min_value: Some(3),
        };

        assert_eq!(min_max, expected);
    }

    #[test]
    fn update_prune_value_test() {
        use crate::board::board_representation;

        let piece_move = Move {
            initial_piece_coordinates: board_representation::BoardCoordinates {
                board_index: 0,
                bit: 0,
            },
            final_piece_bit: 0,
            value: 5,
        };

        let min_max = MinMax {
            max_move: Some(piece_move),
            min_value: Some(3),
        };

        let result = update_prune_value(true, &min_max);

        assert_eq!(result, Some(5));
    }

    #[test]
    fn gen_best_move_test1() {
        use crate::board::board_representation;

        let board = board_representation::fen_decode("7k/2K5/8/8/8/r2r4/3R3n/8 w - - 0 1", true);

        let pieces_info = crate::piece::constants::gen();
        
        let result = gen_best_move(true, 3, 0, 0, None, board, &pieces_info);

        let expected = Move {
            initial_piece_coordinates: board_representation::BoardCoordinates {
                board_index: 1,
                bit: 51,
            },
            final_piece_bit: 55,
            value: 3,
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn gen_best_move_test2() { // Test a capture with en passant being the best move
        use crate::board::board_representation;

        let board = board_representation::fen_decode("K7/8/8/4pP2/8/8/8/k7 w - e6 0 1", true);

        let pieces_info = crate::piece::constants::gen();
        
        let result = gen_best_move(true, 3, 0, 0, None, board, &pieces_info);

        let expected = Move {
            initial_piece_coordinates: board_representation::BoardCoordinates {
                board_index: 0,
                bit: 29,
            },
            final_piece_bit: 20,
            value: 1,
        };

        assert_eq!(result, expected);
    }


    #[test]
    fn gen_best_move_test3() {
        use crate::board::board_representation;

        let board = board_representation::fen_decode("1nb1kb1r/8/2p3p1/1p1pP2p/7P/2P3Pn/4Bq1N/Q2K4 b - - 0 1", true);

        let pieces_info = crate::piece::constants::gen();
        
        let result = gen_best_move(true, 6, 0, 0, None, board, &pieces_info);

        let expected = Move {
            initial_piece_coordinates: board_representation::BoardCoordinates {
                board_index: 0,
                bit: 29,
            },
            final_piece_bit: 20,
            value: 1,
        };

        assert_eq!(result, expected);
    }
}