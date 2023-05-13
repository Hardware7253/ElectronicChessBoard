use std::time::{Duration, Instant};

use crate::board::board_representation;
use crate::board::move_generator::EnemyAttacks;
use crate::TeamBitboards;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Move {
    pub initial_piece_coordinates: board_representation::BoardCoordinates,
    pub final_piece_bit: usize,
    pub value: i8,
    pub heatmap_value: i16,
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
            heatmap_value: 0,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct AlphaBeta {
    pub alpha: i8,
    pub beta: i8,
    pub piece_move: Option<Move>,
}

impl AlphaBeta {
    pub fn new() -> Self {
        AlphaBeta {
            alpha: i8::MIN, // -Infinity
            beta: i8::MAX, // +Infinity
            piece_move: None,
        }
    }
}

// If master team update alpha from child beta
// If not master team update beta from child alpha
pub fn update_alpha_beta(my_alpha_beta: &mut AlphaBeta, child_alpha_beta: &AlphaBeta, master_team: bool) {
    if master_team {
        if my_alpha_beta.alpha < child_alpha_beta.beta {
            my_alpha_beta.alpha = child_alpha_beta.beta;
            my_alpha_beta.piece_move = child_alpha_beta.piece_move;
        }
    } else {
        if my_alpha_beta.beta > child_alpha_beta.alpha {
            my_alpha_beta.beta = child_alpha_beta.alpha;
            my_alpha_beta.piece_move = child_alpha_beta.piece_move;
        }
    }
}


pub fn gen_best_move(
    master_team: bool,
    start_instant: &Instant,
    max_search_millis: &u16,
    search_depth: usize,
    current_depth: usize,
    init_value: i8,
    mut alpha_beta: AlphaBeta,
    opening_heatmap: &[[i16; 64]; 12],
    board: board_representation::Board,
    pieces_info: &[crate::piece::constants::PieceInfo; 12]
) -> AlphaBeta {
    use crate::board::move_generator;
    use crate::board::move_generator::TurnError;

    // If current depth and search depth are equal stop searching down the move tree
    // Or stop searching if the time elapsed is greater than the maximum allowed time
    if current_depth == search_depth || &(start_instant.elapsed().as_millis() as u16) > max_search_millis {
        return AlphaBeta {
            alpha: init_value,
            beta: init_value,
            piece_move: None,
        };
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
    let moves = &mut order_moves(true, &board, &enemy_attacks, &friendly_king, opening_heatmap, &team_bitboards, pieces_info);

    // Add pv move from lower search depth to the start of moves vec to increase alpha beta cuttoffs
    // Iterative deepening
    let pv_alpha_beta: Option<AlphaBeta>;
    if current_depth == 0 && search_depth > 1 {
        let alpha_beta = gen_best_move(
            true, 
            start_instant,
            max_search_millis,
            search_depth - 1,
            0,
            0,
            AlphaBeta::new(),
            opening_heatmap,
            board,
            pieces_info
        );

        moves.rotate_right(1);
        moves[0] = alpha_beta.piece_move.unwrap();
        pv_alpha_beta = Some(alpha_beta);
    } else {
        pv_alpha_beta = None;
    }

    for i in 0..moves.len() {
        let initial_piece_coordinates = moves[i].initial_piece_coordinates;
        let final_piece_bit = moves[i].final_piece_bit;

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

                let mut child_alpha_beta = gen_best_move(
                    !master_team,
                    start_instant,
                    max_search_millis,
                    search_depth,
                    current_depth + 1,
                    branch_value,
                    alpha_beta,
                    opening_heatmap,
                    new_board,
                    pieces_info
                );

                let piece_move = Move {
                    initial_piece_coordinates: initial_piece_coordinates,
                    final_piece_bit: final_piece_bit,
                    value: 0,
                    heatmap_value: 0,
                };

                child_alpha_beta.piece_move = Some(piece_move);

                update_alpha_beta(&mut alpha_beta, &child_alpha_beta, master_team);
            },
            Err(error) => {

                // Update alpha/beta with value of game ending if the game ended
                let mut branch_value;
                let mut valid_move;

                match error {
                    TurnError::Win => {branch_value = i8::MAX; valid_move = true},
                    TurnError::Draw => {branch_value = 0; valid_move = true},
                    TurnError::InvalidMove => {branch_value = 0; valid_move = false},
                    TurnError::InvalidMoveCheck => {branch_value = 0; valid_move = false},
                }

                if valid_move {
                    // If the current branch is not the master team then it's move values are negative (because they negatively impact the master team)
                    if !master_team {
                        branch_value *= -1;
                    }

                    let piece_move = Move {
                        initial_piece_coordinates: initial_piece_coordinates,
                        final_piece_bit: final_piece_bit,
                        value: 0,
                        heatmap_value: 0,
                    };

                    let child_alpha_beta = AlphaBeta {
                        alpha: branch_value,
                        beta: branch_value,
                        piece_move: Some(piece_move),
                    };

                    update_alpha_beta(&mut alpha_beta, &child_alpha_beta, master_team);
                }
            },
        }
        // Stop searching this branch if alpha >= beta
        if alpha_beta.alpha >= alpha_beta.beta {
            break;
        }
    }

    // If the time exceeded the maximum allowed time return the pv move from a lower search depth
    if current_depth == 0 && search_depth > 1 {
        if &(start_instant.elapsed().as_millis() as u16) > max_search_millis {
            return pv_alpha_beta.unwrap();
        }
    }

    alpha_beta    
}

// Returns a vec with potential moves
// If sort is true the moves will be ordered from best to worst
// All moves are valid apart from king moves
fn order_moves(sort: bool, board: &board_representation::Board, enemy_attacks: &EnemyAttacks, friendly_king: &board_representation::BoardCoordinates, opening_heatmap: &[[i16; 64]; 12], team_bitboards: &crate::TeamBitboards, pieces_info: &[crate::piece::constants::PieceInfo; 12]) -> [Move; 96]  {
    use crate::bit_on;
    
    let mut moves_index = 0;
    let mut moves: [Move; 96] = [Move::new(); 96];

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

            // If there is no piece on the board at this bit go to the next bit
            if !bit_on(board.board[i], initial_bit) {
                continue;
            }

            let piece_moves = crate::board::move_generator::gen_piece(&initial_piece_coordinates, None, team_bitboards, false, board, pieces_info);
            
            for final_bit in 0..64 {

                // The piece cannot move to final_bit if it is occupied by a friendly piece
                if bit_on(team_bitboards.friendly_team, final_bit) {
                    continue;
                }

                // Get the heatmap value as the difference of the final and initial bit values
                // This is to prevent pieces from moving to less advantageous positions than ones they are allready in
                let heatmap_value = opening_heatmap[i][final_bit] - opening_heatmap[i][initial_bit];

                // Check the piece can move to final_bit
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
                    moves[moves_index] = Move {
                        initial_piece_coordinates: initial_piece_coordinates,
                        final_piece_bit: final_bit,
                        value: move_value,
                        heatmap_value: heatmap_value,
                    };
                    moves_index += 1;
                } else if &initial_piece_coordinates == friendly_king && (final_bit as i8 - initial_bit as i8).abs() == 2 {

                    // If the piece can't move to the final bit, but is a king then add potential castling moves moves
                    // Because king castling moves aren't a part of gen_piece, so they cannot be ruled out
                    moves[moves_index] = Move {
                        initial_piece_coordinates: initial_piece_coordinates,
                        final_piece_bit: final_bit,
                        value: 0,
                        heatmap_value: heatmap_value,
                    };
                    moves_index += 1;
                }
            }
        }
    }

    // Sort moves and return
    if sort {

        // Sort moves by value first
        // Sort moves by heatmap_value if they have the same value
        // https://stackoverflow.com/questions/70193935/how-to-sort-a-vec-of-structs-by-2-or-multiple-fields

        moves.sort_by(| a, b | if a.value == b.value {
            b.heatmap_value.partial_cmp(&a.heatmap_value).unwrap()
        } else {
            b.value.partial_cmp(&a.value).unwrap()
        });
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

        let opening_heatmap = [[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 10, 1, 18, 10, 9, 9, 1, 0, 1, 33, 61, 475, 338, 22, 6, 5, 51, 142, 1144, 2288, 2246, 392, 88, 80, 88, 74, 361, 111, 276, 124, 322, 62, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 1, 0, 4, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 5, 35, 32, 94, 499, 3, 0], [1, 0, 0, 0, 0, 0, 0, 2, 0, 0, 2, 0, 0, 19, 0, 2, 0, 0, 15, 1, 2, 7, 0, 0, 1, 31, 0, 19, 145, 2, 79, 0, 9, 0, 11, 268, 58, 0, 1, 7, 16, 17, 1470, 1, 3, 2054, 9, 15, 0, 0, 2, 115, 62, 1, 0, 0, 0, 1, 0, 0, 5, 2, 2, 0], [1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 3, 20, 22, 1, 0, 0, 1, 35, 0, 0, 17, 0, 2, 0, 314, 1, 13, 2, 0, 292, 0, 139, 2, 509, 2, 0, 47, 0, 35, 6, 108, 1, 162, 124, 1, 2, 3, 0, 51, 19, 57, 148, 1, 205, 0, 1, 0, 2, 0, 0, 3, 0, 0], [0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 3, 2, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 1, 4, 1, 2, 0, 24, 22, 0, 13, 32, 3, 2, 24, 3, 0, 48, 7, 17, 6, 42, 0, 0, 0, 0, 66, 49, 67, 3, 0, 0, 0, 1, 0, 3, 3, 0, 0, 0], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 9, 4, 1, 0, 0, 0, 23, 4, 0, 26, 498, 6], [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 348, 125, 418, 716, 867, 40, 525, 86, 17, 238, 834, 1360, 1326, 216, 134, 18, 0, 13, 174, 512, 190, 170, 68, 4, 1, 0, 34, 3, 4, 37, 4, 0, 0, 6, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0], [0, 8, 3, 3, 17, 458, 5, 0, 1, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0], [0, 13, 0, 2, 1, 1, 8, 0, 0, 4, 3, 219, 58, 2, 1, 0, 21, 32, 1057, 15, 1, 1874, 4, 29, 56, 0, 8, 130, 31, 3, 1, 10, 0, 9, 4, 40, 190, 2, 21, 0, 0, 0, 31, 0, 2, 1, 3, 0, 0, 0, 1, 2, 0, 2, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0], [0, 0, 0, 0, 1, 3, 0, 1, 1, 74, 0, 44, 307, 0, 387, 2, 20, 31, 2, 44, 56, 5, 9, 5, 27, 0, 241, 0, 2, 79, 3, 1, 0, 297, 3, 5, 2, 0, 98, 4, 0, 0, 60, 3, 1, 8, 0, 3, 0, 0, 0, 5, 1, 3, 1, 1, 0, 1, 0, 1, 0, 3, 0, 0], [1, 1, 2, 4, 5, 0, 0, 0, 0, 0, 36, 10, 62, 0, 0, 0, 0, 28, 0, 10, 2, 36, 6, 0, 79, 0, 0, 53, 5, 4, 12, 2, 0, 1, 1, 9, 3, 2, 0, 51, 1, 0, 1, 0, 0, 0, 0, 2, 0, 2, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0], [0, 0, 2, 7, 0, 4, 458, 0, 0, 0, 0, 0, 5, 17, 2, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]];
        
        let board = board_representation::fen_decode("k7/8/8/8/4r3/3P4/8/7K w - - 0 1", true);

        let king = board_representation::BoardCoordinates {
            board_index: 5,
            bit: 63,
        };

        let team_bitboards = TeamBitboards::new(king.board_index, &board);

        let pieces_info = crate::piece::constants::gen();

        let enemy_attacks = move_generator::gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

        let result = order_moves(true, &board, &enemy_attacks, &king, &opening_heatmap, &team_bitboards, &pieces_info);

        let best_move = Move {
            initial_piece_coordinates: board_representation::BoardCoordinates {
                board_index: 0,
                bit: 43,
            },
            final_piece_bit: 36,
            value: 5,
            heatmap_value: result[0].heatmap_value,
        };

        assert_eq!(result[0], best_move);
    }

    #[test]
    fn gen_best_move_test1() {
        use crate::board::board_representation;

        let board = board_representation::fen_decode("7k/2K5/8/8/8/r2r4/3R3n/8 w - - 0 1", true);

        let pieces_info = crate::piece::constants::gen();
        
        let result = gen_best_move(true, &Instant::now(), &10000, 3, 0, 0, AlphaBeta::new(), &[[0i16; 64]; 12], board, &pieces_info);

        let expected = Move {
            initial_piece_coordinates: board_representation::BoardCoordinates {
                board_index: 1,
                bit: 51,
            },
            final_piece_bit: 55,
            value: 0,
            heatmap_value: 0,
        };

        assert_eq!(result.piece_move, Some(expected));
    }

    #[test]
    fn gen_best_move_test2() { // Test a capture with en passant being the best move
        use crate::board::board_representation;

        let board = board_representation::fen_decode("K7/8/8/4pP2/8/8/8/k7 w - e6 0 1", true);

        let pieces_info = crate::piece::constants::gen();
        
        let result = gen_best_move(true, &Instant::now(), &10000, 3, 0, 0, AlphaBeta::new(), &[[0i16; 64]; 12], board, &pieces_info);

        let expected = Move {
            initial_piece_coordinates: board_representation::BoardCoordinates {
                board_index: 0,
                bit: 29,
            },
            final_piece_bit: 20,
            value: 0,
            heatmap_value: 0,
        };

        assert_eq!(result.piece_move, Some(expected));
    }
    
    /*
    #[test]
    fn gen_best_move_test3() {
        use crate::board::board_representation;

        let board = board_representation::fen_decode("r1bq4/3kbB2/2p5/1p1n3p/1P1B4/2N1RN2/2PQ1PPP/5RK1 b - - 0 1", true);

        let pieces_info = crate::piece::constants::gen();
        
        let result = gen_best_move(true, &Instant::now(), &10000, 6, 0, 0, AlphaBeta::new(), &[[0i16; 64]; 12], board, &pieces_info);

        let expected = Move {
            initial_piece_coordinates: board_representation::BoardCoordinates {
                board_index: 0,
                bit: 0,
            },
            final_piece_bit: 0,
            value: 0,
            heatmap_value: 0,
        };

        println!("{:?}", result);
        assert_eq!(result.piece_move, Some(expected));
    }
    */
}
