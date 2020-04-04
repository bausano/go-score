pub enum Stone {
    White,
    Black,
    None,
}

pub enum Territory {
    White,
    Black,
    None,
}

pub enum Board<T> {
    NineByNine([[T; 9]; 9]),
    ThirteenByThirteen([[T; 13]; 13]),
    NineteenByNineteen([[T; 19]; 19]),
}

pub fn count_score(_board: Board<Stone>) -> Board<Territory> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
