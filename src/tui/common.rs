use ratatui::style::Color;

pub const FG_COLOR: Color = Color::from_u32(0x282828);
pub const PRIMARY_COLOR: Color = Color::from_u32(0xd3869b);
pub const HELP_COLOR: Color = Color::from_u32(0xfabd2f);
pub const COLOR_TWO: Color = Color::from_u32(0x83a598);
pub const COLOR_THREE: Color = Color::from_u32(0xfabd2f);
pub const TAGS_COLOR: Color = Color::from_u32(0xb8bb26);
pub const INFO_MESSAGE_COLOR: Color = Color::from_u32(0x83a598);
pub const ERROR_MESSAGE_COLOR: Color = Color::from_u32(0xfb4934);
pub const TITLE: &str = " bmm ";
pub const MIN_TERMINAL_WIDTH: u16 = 96;
pub const MIN_TERMINAL_HEIGHT: u16 = 24;

#[derive(PartialEq, Debug, Clone, Copy)]
pub(crate) enum ActivePane {
    List,
    TagsList,
    SearchInput,
    Help,
}

pub(super) struct TerminalDimensions {
    pub(super) width: u16,
    pub(super) height: u16,
}
