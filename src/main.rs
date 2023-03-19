fn main() {
    let board = chess2::board::board_representation::fen_decode("8/8/8/8/8/4P1P1/8/8 b - e3 0 1", true);
    println!("{:?}", board.board[0]);
}  
