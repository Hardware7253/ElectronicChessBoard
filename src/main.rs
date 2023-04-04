use chess2::board::board_representation;

fn main() {

    let board = board_representation::fen_decode("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", true);

    let mut bug_board = board_representation::fen_decode("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", true);
    bug_board.board = [64739244660228096, 9295429630892703744, 144115222435594240, 2594073385365405696, 576460752303423488, 1152921504606846976, 61184, 129, 35184372088896, 67108868, 4096, 16, 9086293723196625022];

    let bug_board_fen_encode = board_representation::fen_encode(&bug_board);
    println!("{}",bug_board_fen_encode);


}
