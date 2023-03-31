fn main() {

    /*
    let mut board = chess2::board::board_representation::fen_decode("8/1P6/8/8/1P6/P1P5/8/8 w - - 0 1", true).board[0];
    board |= chess2::board::board_representation::fen_decode("8/P5P1/1P3P2/2P1P3/8/2P1P3/1P3P2/6P1 w - - 0 1", true).board[0];
    board |= chess2::board::board_representation::fen_decode("8/4P1P1/3P3P/8/3P3P/4P1P1/8/8 w - - 0 1", true).board[0];
    println!("{:?}", board);
    */

    let board = chess2::board::board_representation::fen_decode("K7/8/4P3/8/8/8/8/k7 w - - 0 1", true).board[0];
    println!("{:?}", board);

    println!("{}", chess2::find_bit_on(1099511660544, 0));

    //[0, 1125899906842624, 0, 0, 0, 4, 0, 32769, 0, 0, 0, 128, 18446744073709551615]

    //[0, 2251799813685248, 0, 0, 0, 34359738368, 0, 8800387989504, 35184372088832, 0, 0, 128, 18446744073709551615]

    //[0, 8796093022208, 0, 0, 0, 4, 0, 4398046511104, 36028797018963968, 0, 0, 128, 18446744073709551615]

    // [0, 8796093022208, 0, 0, 0, 2048, 0, 256, 36028797018963968, 0, 0, 128, 18446744073709551615] 7k/r2K4/8/8/8/3R4/7n/8 w - - 0 1
    // [0, 1099511627776, 0, 0, 0, 1024, 0, 2, 36028797018963968, 0, 0, 128, 18446744073709551615]
    let board = chess2::board::board_representation::Board { board: [0, 8796093022208, 0, 0, 0, 0, 0, 4, 36028797018963968, 0, 0, 128, 18446744073709551615], whites_move: true, points: chess2::board::board_representation::Points { white_points: 5, black_points: 0 }, points_delta: 0, turns_since_capture: 3, en_passant_target: None };
}  
