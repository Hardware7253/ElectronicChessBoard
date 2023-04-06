use chess2::board::board_representation;

fn main() {

    let board = board_representation::fen_decode("P2PPPPP/PP1PPPPP/8/1P3P2/8/8/8/8 b - - 0 1", true);
    println!("{:?}", board.board);

    let mut bug_board = board_representation::fen_decode("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", true);
    bug_board.board = [15417791883686445072, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    //bug_board.board = [65020753979899904, 9295429630892703744, 4611686018427912192, 288230376151711744, 4503599627370496, 1152921504606846976, 33616640, 129, 2112, 536870944, 8, 16, 9086293723196623998];

    let bug_board_fen_encode = board_representation::fen_encode(&bug_board);
    println!("{}", bug_board_fen_encode);


}
