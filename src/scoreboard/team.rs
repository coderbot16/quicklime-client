use text::flat::ChatBuf;
use text::style::PaletteColor;
use scoreboard::Identifier;

pub struct Team {
	identifier: Identifier,
	color: PaletteColor,
	display: ChatBuf,
	prefix: ChatBuf,
	suffix: ChatBuf
}