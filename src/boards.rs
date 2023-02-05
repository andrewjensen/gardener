use serde::Serialize;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseBoardError;

#[derive(Serialize, Debug, Clone)]
pub enum Board {
    Seed,
    Pod,
    // TODO: add other boards

    // From pd2dsy help: The supported boards are:
    // pod, patch, patch_init, field, petal
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
