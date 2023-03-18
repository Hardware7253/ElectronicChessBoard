pub mod constants {

    pub struct PieceInfo {
        pub moves: [i8; 8], // Piece move set
        pub moves_no: usize, // Number of moves is moves array
        pub move_only: bool, // True if the piece cannot capture with its move set
        pub sliding: bool, // True if the piece can move more than one square at a time
    }

    pub fn gen() -> [PieceInfo; 12] {
        // How much the piece index changes when a move is made in a direction
        let knight_moves = [-17, -15, -6, 10, 17, 15, 6, -10];
        let straight_moves = [-8, 1, 8, -1, 0, 0, 0, 0];
        let diagonal_moves = [-9, -7, 9, 7, 0, 0, 0, 0];

        [   
            // White team
            PieceInfo { // pawn
                moves: [straight_moves[0], 0, 0, 0, 0, 0, 0, 0],
                moves_no: 1,
                move_only: true,
                sliding: false,
            },

            PieceInfo { // rook
                moves: straight_moves,
                moves_no: 4,
                move_only: false,
                sliding: true,
            },

            PieceInfo { // knight
                moves: knight_moves,
                moves_no: 8,
                move_only: false,
                sliding: false,
            },

            PieceInfo { // bishop
                moves: diagonal_moves,
                moves_no: 4,
                move_only: false,
                sliding: true,
            },

            PieceInfo { // queen
                moves: [straight_moves[0], straight_moves[1], straight_moves[2], straight_moves[3], diagonal_moves[0], diagonal_moves[1], diagonal_moves[2], diagonal_moves[3]],
                moves_no: 8,
                move_only: false,
                sliding: true,
            },

            PieceInfo { // king
                moves: [straight_moves[0], straight_moves[1], straight_moves[2], straight_moves[3], diagonal_moves[0], diagonal_moves[1], diagonal_moves[2], diagonal_moves[3]],
                moves_no: 8,
                move_only: false,
                sliding: false,
            },


            // Black team
            PieceInfo { // pawn
                moves: [straight_moves[3], 0, 0, 0, 0, 0, 0, 0],
                moves_no: 1,
                move_only: true,
                sliding: false,
            },

            PieceInfo { // rook
                moves: straight_moves,
                moves_no: 4,
                move_only: false,
                sliding: true,
            },

            PieceInfo { // knight
                moves: knight_moves,
                moves_no: 8,
                move_only: false,
                sliding: false,
            },

            PieceInfo { // bishop
                moves: diagonal_moves,
                moves_no: 4,
                move_only: false,
                sliding: true,
            },

            PieceInfo { // queen
                moves: [straight_moves[0], straight_moves[1], straight_moves[2], straight_moves[3], diagonal_moves[0], diagonal_moves[1], diagonal_moves[2], diagonal_moves[3]],
                moves_no: 8,
                move_only: false,
                sliding: true,
            },

            PieceInfo { // king
                moves: [straight_moves[0], straight_moves[1], straight_moves[2], straight_moves[3], diagonal_moves[0], diagonal_moves[1], diagonal_moves[2], diagonal_moves[3]],
                moves_no: 8,
                move_only: false,
                sliding: false,
            },
        ]
    }
}