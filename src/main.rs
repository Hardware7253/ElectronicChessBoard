fn main() {
    let board = chess2::board::board_representation::fen_decode("8/8/8/8/3P4/3P4/8/8 w - - 0 1", true);
    println!("{:?}", board.board[0]);
}  
