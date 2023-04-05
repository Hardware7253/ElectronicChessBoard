use chess2::board::board_representation;

fn main() {

    let board = board_representation::fen_decode("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", true);

    let mut bug_board = board_representation::fen_decode("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", true);
    //bug_board.board = [64739244660228096, 9295429630892703744, 144115222435594240, 2594073385365405696, 576460752303423488, 1152921504606846976, 61184, 129, 35184372088896, 67108868, 4096, 16, 9086293723196625022];
    bug_board.board = [65020753979899904, 9295429630892703744, 4611686018427912192, 288230376151711744, 4503599627370496, 1152921504606846976, 33616640, 129, 2112, 536870944, 8, 16, 9086293723196623998];

    let bug_board_fen_encode = board_representation::fen_encode(&bug_board);
    println!("{}",bug_board_fen_encode);


}
