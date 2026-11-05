//! The item itself: its row in ItemParam and its text.

use crate::param::{Param, ParamError};

/// The item's row. Using an item applies its Effect ID through `ApplySpEffect` after the animation, even
/// when no such effect exists, so the row's Effect ID is set to this same number and the DLL watches for it.
pub const ITEM: u32 = 60_595_010;
/// Reusable and limited to one, which is what the item needs.
pub const AGED_FEATHER: u32 = 60_355_000;
/// Lends the item its icon.
pub const DUNG_PIE: u32 = 60_595_000;
const OFF_ICON_ID: usize = 0x00;
const OFF_EFFECT_ID: usize = 0x04;

pub const NAME: &str = "Easy Go";
pub const SIMPLE: &str = "Saves your game and returns you to the desktop.";
/// The detail panel doesn't wrap, so the line breaks are part of the text: 57 characters were cut off
/// in play, and 51 fit.
pub const DETAILED: &str = "Easy come, easy go. A foul parting gift for those\nwho have somewhere better to be.\n\nUse it to save your game and quit to the desktop.\nThere is no confirmation.\n\nDon't go hollow, I love you.\nhttps://mutantfactory.net/feedback";

/// Returns ItemParam with the item's row added, or `None` if it's already there.
pub fn add_row(item_param: &[u8]) -> Result<Option<Vec<u8>>, AddRowError> {
    let mut param = Param::parse(item_param).map_err(AddRowError::Param)?;
    if param.row(ITEM).is_some() {
        return Ok(None);
    }
    let mut row = param.row(AGED_FEATHER).ok_or(AddRowError::Missing("Aged Feather"))?.to_vec();
    let icon = param.row(DUNG_PIE).ok_or(AddRowError::Missing("Dung Pie"))?;
    let icon_id: [u8; 4] = icon[OFF_ICON_ID..OFF_ICON_ID + 4].try_into().expect("four bytes");
    row[OFF_ICON_ID..OFF_ICON_ID + 4].copy_from_slice(&icon_id);
    row[OFF_EFFECT_ID..OFF_EFFECT_ID + 4].copy_from_slice(&ITEM.to_le_bytes());
    param.insert(ITEM, row).map_err(AddRowError::Param)?;
    param.to_bytes().map(Some).map_err(AddRowError::Param)
}

#[derive(Debug, PartialEq, Eq)]
pub enum AddRowError {
    Param(ParamError),
    Missing(&'static str),
}

impl std::fmt::Display for AddRowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddRowError::Param(e) => write!(f, "{e}"),
            AddRowError::Missing(row) => write!(f, "ItemParam has no {row} row to copy"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An ItemParam holding the two source rows, with 8-byte rows.
    fn item_param() -> Vec<u8> {
        const HEADER: usize = 0x40;
        const ENTRY: usize = 24;
        let rows = [(AGED_FEATHER, [1u8, 2, 3, 4, 5, 6, 7, 8]), (DUNG_PIE, [9, 9, 9, 9, 0, 0, 0, 0])];
        let data = HEADER + rows.len() * ENTRY;
        let mut b = vec![0u8; HEADER];
        b[0..4].copy_from_slice(&((data + rows.len() * 8) as u32).to_le_bytes());
        b[0x0a..0x0c].copy_from_slice(&(rows.len() as u16).to_le_bytes());
        b[0x30..0x38].copy_from_slice(&(data as u64).to_le_bytes());
        for (i, (id, _)) in rows.iter().enumerate() {
            b.extend_from_slice(&(*id as u64).to_le_bytes());
            b.extend_from_slice(&((data + i * 8) as u64).to_le_bytes());
            b.extend_from_slice(&[0u8; 8]);
        }
        for (_, row) in &rows {
            b.extend_from_slice(row);
        }
        b
    }

    #[test]
    fn row_is_a_feather_with_the_dung_icon_and_its_own_effect() {
        let out = add_row(&item_param()).unwrap().unwrap();
        let param = Param::parse(&out).unwrap();
        let row = param.row(ITEM).unwrap();
        assert_eq!(row[..4], [9, 9, 9, 9]);
        assert_eq!(row[4..8], ITEM.to_le_bytes());
        assert_eq!(param.row(AGED_FEATHER).unwrap(), [1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn a_param_that_already_has_the_row_is_left_alone() {
        let once = add_row(&item_param()).unwrap().unwrap();
        assert_eq!(add_row(&once), Ok(None));
    }

    #[test]
    fn detailed_text_fits_the_panel() {
        assert!(DETAILED.lines().all(|l| l.chars().count() <= 52));
    }
}
