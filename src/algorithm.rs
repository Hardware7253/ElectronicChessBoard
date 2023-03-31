use crate::board::board_representation;
use crate::board::move_generator::EnemyAttacks;

//pub fn gen_best_move(team_white: bool, board: board_representation::Board)

#[derive(Copy, Clone, PartialEq, Debug)]
struct Move {
    initial_piece_coordinates: board_representation::BoardCoordinates,
    final_piece_bit: usize,
    value: i8,
}

// Returns a vec with potential moves
// If sort is true the moves will be ordered from best to worst
// All moves are valid apart from king moves
fn order_moves(team_white: bool, sort: bool, board: &board_representation::Board, enemy_attacks: &EnemyAttacks, friendly_king: &board_representation::BoardCoordinates, team_bitboards: crate::TeamBitboards, pieces_info: &[crate::piece::constants::PieceInfo; 12]) -> Vec<Move> {
    use crate::bit_on;
    
    let mut moves: Vec<Move> = Vec::new();

    // Get friendly and enemy board indexes
    let friendly_indexes;
    let enemy_index_bottom; // Inclusive
    let enemy_index_top; // Not inclusive
    if team_white {
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

            let piece_moves = crate::board::move_generator::gen_piece(&initial_piece_coordinates, team_bitboards, false, board, pieces_info);
            
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
                    moves.push(Move {
                        initial_piece_coordinates: initial_piece_coordinates,
                        final_piece_bit: final_bit,
                        value: move_value,
                    });
                } else if &initial_piece_coordinates == friendly_king { // Add potentially invalid king moves to moves vec to account for castling
                    moves.push(Move {
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
        moves.sort_by(|a, b| b.value.cmp(&a.value));
    }
    moves
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_moves_test() {
        use crate::TeamBitboards;
        use crate::board::board_representation;
        use crate::board::move_generator;

            let board = board_representation::fen_decode("k7/8/8/8/4r3/3P4/8/7K w - - 0 1", true);

            let king = board_representation::BoardCoordinates {
                board_index: 5,
                bit: 63,
            };

            let enemy_king = board_representation::BoardCoordinates {
                board_index: 11,
                bit: 0,
            };

            let team_bitboards = TeamBitboards::new(king.board_index, &board);

            let pieces_info = crate::piece::constants::gen();

            let enemy_attacks = move_generator::gen_enemy_attacks(&king, team_bitboards, &board, &pieces_info);

            let result = order_moves(true, true, &board, &enemy_attacks, &king, team_bitboards, &pieces_info);

            let best_move = Move {
                initial_piece_coordinates: board_representation::BoardCoordinates {
                    board_index: 0,
                    bit: 43,
                },
                final_piece_bit: 36,
                value: 5,
            };

            assert_eq!(result[0], best_move);
    }
}