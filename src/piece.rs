pub mod constants {

    pub stuct PieceInfo {
        moves: [i8; 8],
        sliding: bool,
    }

    pub fn gen() -> [PieceInfo; 12] {
        let knight_moves = [-17, -15, -6, 10, 17, 15, 6, -10];
        let straight_moves = [-8, 1, 8, -1, 0, 0, 0, 0];
        let diagonal_moves = [-9, -7, 9, 7, 0, 0, 0, 0];

        [   
            // White team
            PieceInfo {
                moves: [straight_moves[0], 0, 0, 0, 0, 0, 0, 0],
                sliding: false,
            },

            PieceInfo {
                moves: straight_moves,
                sliding: true,
            },

            PieceInfo {
                moves: knight_moves,
                sliding: false,
            },

            PieceInfo {
                moves: diagonal_moves,
                sliding: true,
            },

            PieceInfo {
                moves: [straight_moves[0], straight_moves[1], straight_moves[2], straight_moves[3], diagonal_moves[0], diagonal_moves[1], diagonal_moves[2], diagonal_moves[3]],
                sliding: true,
            },

            PieceInfo {
                moves: [straight_moves[0], straight_moves[1], straight_moves[2], straight_moves[3], diagonal_moves[0], diagonal_moves[1], diagonal_moves[2], diagonal_moves[3]],
                sliding: false,
            },

            // Black team
            PieceInfo {
                moves: [straight_moves[3], 0, 0, 0, 0, 0, 0, 0],
                sliding: false,
            },

            PieceInfo {
                moves: straight_moves,
                sliding: true,
            },

            PieceInfo {
                moves: knight_moves,
                sliding: false,
            },

            PieceInfo {
                moves: diagonal_moves,
                sliding: true,
            },

            PieceInfo {
                moves: [straight_moves[0], straight_moves[1], straight_moves[2], straight_moves[3], diagonal_moves[0], diagonal_moves[1], diagonal_moves[2], diagonal_moves[3]],
                sliding: true,
            },

            PieceInfo {
                moves: [straight_moves[0], straight_moves[1], straight_moves[2], straight_moves[3], diagonal_moves[0], diagonal_moves[1], diagonal_moves[2], diagonal_moves[3]],
                sliding: false,
            },
        ];
    }
}