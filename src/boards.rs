use serde::Serialize;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseBoardError;

#[derive(Serialize, Debug, Clone)]
pub enum Board {
    #[serde(rename = "pod")]
    Pod,

    #[serde(rename = "patch")]
    Patch,

    #[serde(rename = "patch_init")]
    PatchInit,

    #[serde(rename = "field")]
    Field,

    #[serde(rename = "petal")]
    Petal,
}

impl FromStr for Board {
    type Err = ParseBoardError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pod" => Ok(Board::Pod),
            "patch" => Ok(Board::Patch),
            "patch_init" => Ok(Board::PatchInit),
            "field" => Ok(Board::Field),
            "petal" => Ok(Board::Petal),
            _ => Err(ParseBoardError),
        }
    }
}
