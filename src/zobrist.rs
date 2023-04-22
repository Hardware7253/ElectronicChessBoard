extern crate rand;
use rand::thread_rng;
use rand::Rng;

// Indexes for pieces in bitstrings array
    // Moves = 1
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

    // Moves = 0
        // White pawn = 12
        // White rook = 13
        // White king = 14

        // Black pawn = 15
        // Black rook = 16
        // Black king = 17

    // En passant target squares = 18
    // Other = 19 (bit 0 used if it is whites turn)

// Generate a random number for every type of piece, at every square
pub fn gen_bitstrings_array() -> [[u64; 64]; 20] {

    let mut bitstrings_array = [[0u64; 64]; 20];
    for i in 0..20 {
        for bit in 0..64 {
            bitstrings_array[i][bit] = thread_rng().gen_range(0, std::u64::MAX);
        }
    }

    bitstrings_array
}

// Return a unique u64 value for the given board
pub fn hash_board(board: &crate::board::board_representation::Board, bitstrings_array: &[[u64; 64]; 20]) -> u64 {
    use crate::bit_on;

    let mut hash: u64 = 0;

    // Add team move to hash
    if board.whites_move {
        hash ^= bitstrings_array[19][0];
    }

    // Add en passant target to hash
    match board.en_passant_target {
        Some(bit) => hash ^= bitstrings_array[18][bit],
        None => ()
    };

    // Bitstring array indexes for pieces with 0 moves
    let zero_move_ids: [usize; 12] = [12, 13, 2, 3, 4, 14, 15, 16, 8, 9, 10, 17];

    for board_index in 0..12 {
        for board_bit in 0..64 {
            if bit_on(board.board[board_index], board_bit) { // Check piece exists on the board
                
                let mut board_index = board_index;
                if !bit_on(board.board[12], board_bit) { // If the piece has zero moves get the proper index for the bitstrings array
                    board_index = zero_move_ids[board_index];
                }

                // Add piece to hash
                hash ^= bitstrings_array[board_index][board_bit];
            }
        }
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_board_test() {
        use crate::board::board_representation;
    
        let bitstrings_array = gen_bitstrings_array();

        let board1 = board_representation::fen_decode("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", true);
        let board2 = board_representation::fen_decode("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w Qkq - 0 1", true);

        let board1_hash = hash_board(&board1, &bitstrings_array);
        let board2_hash = hash_board(&board2, &bitstrings_array);

        assert_ne!(board1_hash, board2_hash);
    }
}