use serde::Serialize;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq)]
pub struct ParseBoardError;

#[derive(Serialize, Debug, Clone)]
pub enum Board {
    #[serde(rename = "seed")]
    SeedCustomJson,

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
            "seed" => Ok(Board::SeedCustomJson),
            "pod" => Ok(Board::Pod),
            "patch" => Ok(Board::Patch),
            "patch_init" => Ok(Board::PatchInit),
            "field" => Ok(Board::Field),
            "petal" => Ok(Board::Petal),
            _ => Err(ParseBoardError),
        }
    }
}

// TODO: there's got to be a more clever way to convert back and forth, maybe with `serde`
impl Board {
    pub fn to_str(&self) -> String {
        match self {
            Board::SeedCustomJson => "seed".to_string(),
            Board::Pod => "pod".to_string(),
            Board::Patch => "patch".to_string(),
            Board::PatchInit => "patch_init".to_string(),
            Board::Field => "field".to_string(),
            Board::Petal => "petal".to_string(),
        }
    }
}
