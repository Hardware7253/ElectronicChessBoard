fn main() {
    let board = chess2::board::board_representation::fen_decode("3P4/3P4/3P2P1/3P1P2/2PPP3/PPP1PPP1/2PPP3/1P1P1P2 w - - 0 1", true);
    println!("{:?}", board.board[0]);
}  
