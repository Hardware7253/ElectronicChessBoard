pub mod constants {

    pub struct PieceInfo {
        pub moves: [i8; 8], // Piece move set
        pub moves_no: usize, // Number of moves is moves array
        pub move_only: bool, // True if the piece cannot capture with its move set
        pub sliding: bool, // True if the piece can move more than one square at a time

        pub value: i8, // Material value of the piece
    }

    pub fn gen() -> [PieceInfo; 12] {
        // How much the piece index changes when a move is made in a direction
        const KNIGHT_MOVES: [i8; 8] = [-17, -15, -6, 10, 17, 15, 6, -10];
        const STRAIGHT_MOVES: [i8; 8] = [-8, 1, 8, -1, 0, 0, 0, 0];
        const DIAGONAL_MOVES: [i8; 8] = [-9, -7, 9, 7, 0, 0, 0, 0];

        [   
            // White team
            PieceInfo { // pawn
                moves: [STRAIGHT_MOVES[0], 0, 0, 0, 0, 0, 0, 0],
                moves_no: 1,
                move_only: true,
                sliding: false,

                value: 1,
            },

            PieceInfo { // rook
                moves: STRAIGHT_MOVES,
                moves_no: 4,
                move_only: false,
                sliding: true,

                value: 5,
            },

            PieceInfo { // knight
                moves: KNIGHT_MOVES,
                moves_no: 8,
                move_only: false,
                sliding: false,

                value: 3,
            },

            PieceInfo { // bishop
                moves: DIAGONAL_MOVES,
                moves_no: 4,
                move_only: false,
                sliding: true,

                value: 3,
            },

            PieceInfo { // queen
                moves: [STRAIGHT_MOVES[0], STRAIGHT_MOVES[1], STRAIGHT_MOVES[2], STRAIGHT_MOVES[3], DIAGONAL_MOVES[0], DIAGONAL_MOVES[1], DIAGONAL_MOVES[2], DIAGONAL_MOVES[3]],
                moves_no: 8,
                move_only: false,
                sliding: true,

                value: 9,
            },

            PieceInfo { // king
                moves: [STRAIGHT_MOVES[0], STRAIGHT_MOVES[1], STRAIGHT_MOVES[2], STRAIGHT_MOVES[3], DIAGONAL_MOVES[0], DIAGONAL_MOVES[1], DIAGONAL_MOVES[2], DIAGONAL_MOVES[3]],
                moves_no: 8,
                move_only: false,
                sliding: false,

                value: 0,
            },


            // Black team
            PieceInfo { // pawn
                moves: [STRAIGHT_MOVES[3], 0, 0, 0, 0, 0, 0, 0],
                moves_no: 1,
                move_only: true,
                sliding: false,

                value: 1,
            },

            PieceInfo { // rook
                moves: STRAIGHT_MOVES,
                moves_no: 4,
                move_only: false,
                sliding: true,

                value: 5,
            },

            PieceInfo { // knight
                moves: KNIGHT_MOVES,
                moves_no: 8,
                move_only: false,
                sliding: false,

                value: 3,
            },

            PieceInfo { // bishop
                moves: DIAGONAL_MOVES,
                moves_no: 4,
                move_only: false,
                sliding: true,

                value: 3,
            },

            PieceInfo { // queen
                moves: [STRAIGHT_MOVES[0], STRAIGHT_MOVES[1], STRAIGHT_MOVES[2], STRAIGHT_MOVES[3], DIAGONAL_MOVES[0], DIAGONAL_MOVES[1], DIAGONAL_MOVES[2], DIAGONAL_MOVES[3]],
                moves_no: 8,
                move_only: false,
                sliding: true,

                value: 9,
            },

            PieceInfo { // king
                moves: [STRAIGHT_MOVES[0], STRAIGHT_MOVES[1], STRAIGHT_MOVES[2], STRAIGHT_MOVES[3], DIAGONAL_MOVES[0], DIAGONAL_MOVES[1], DIAGONAL_MOVES[2], DIAGONAL_MOVES[3]],
                moves_no: 8,
                move_only: false,
                sliding: false,

                value: 0,
            },
        ]
    }
}