fn main() {

    /*
    let mut board = chess2::board::board_representation::fen_decode("8/1P6/8/8/1P6/P1P5/8/8 w - - 0 1", true).board[0];
    board |= chess2::board::board_representation::fen_decode("8/P5P1/1P3P2/2P1P3/8/2P1P3/1P3P2/6P1 w - - 0 1", true).board[0];
    board |= chess2::board::board_representation::fen_decode("8/4P1P1/3P3P/8/3P3P/4P1P1/8/8 w - - 0 1", true).board[0];
    println!("{:?}", board);
    */

    let board = chess2::board::board_representation::fen_decode("8/8/8/2PPP3/2P1P3/2PPP3/8/8 w - - 0 1", true).board[0];
    println!("{:?}", board);
}  
