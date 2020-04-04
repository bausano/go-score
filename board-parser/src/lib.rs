use score_counter::Board;
use score_counter::Stone;

pub fn parse_image() -> Board<Stone> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        match parse_image() {
            Board::NineByNine(board) => assert_eq!(9, board.len()),
            Board::ThirteenByThirteen(board) => assert_eq!(13, board.len()),
            Board::NineteenByNineteen(board) => assert_eq!(19, board.len()),
        }
    }
}
