#![no_std]
#![no_main]

use core::cmp::PartialOrd;
use rtt_target::{rprintln, rtt_init_print};

use crate::board::board_representation;
use crate::board::move_generator::EnemyAttacks;
use crate::TeamBitboards;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Move {
    pub initial_piece_coordinates: board_representation::BoardCoordinates,
    pub final_piece_bit: usize,
    pub value: i8,
    pub heatmap_value: i16,
}

impl Move {
    pub fn new() -> Self {
        Move {
            initial_piece_coordinates: board_representation::BoardCoordinates::new(),
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
    cycle_counter: &mut crate::embedded::cycle_counter::Counter,
    start_cycles: &u64,
    max_elapsed_cycles: &u64,
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
    use stm32f1xx_hal::pac::DWT;

    // If current depth and search depth are equal stop searching down the move tree
    // Or stop searching if the time elapsed is greater than the maximum allowed time
    cycle_counter.update();
    if current_depth == search_depth || cycle_counter.cycles > start_cycles + max_elapsed_cycles {
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
            cycle_counter,
            start_cycles,
            max_elapsed_cycles,
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
                    cycle_counter,
                    start_cycles,
                    max_elapsed_cycles,
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
        if cycle_counter.cycles > start_cycles + max_elapsed_cycles {
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

        moves.sort_unstable_by(| a, b | if a.value == b.value {
            b.heatmap_value.partial_cmp(&a.heatmap_value).unwrap()
        } else {
            b.value.partial_cmp(&a.value).unwrap()
        });
    }
    moves
}
