use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseBoardError;

#[derive(Debug)]
pub enum Board {
    Seed,
    Pod,
    // TODO: add other boards
}

impl FromStr for Board {
    type Err = ParseBoardError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "seed" => Ok(Board::Seed),
            "pod" => Ok(Board::Pod),
            _ => Err(ParseBoardError),
        }
    }
}
