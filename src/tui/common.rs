use ratatui::style::Color;

pub const FG_COLOR: Color = Color::from_u32(0x282828);
pub const PRIMARY_COLOR: Color = Color::from_u32(0xd3869b);
pub const SECONDARY_COLOR: Color = Color::from_u32(0x83a598);
pub const TITLE: &str = " bmm ";
//pub const UNEXPECTED_ERROR_MESSAGE: &str =
//    "something unexpected happened, please let @dhth know via https://github.com/dhth/shfl/issues";

#[derive(PartialEq, Debug)]
pub(crate) enum View {
    List,
    Help,
}
