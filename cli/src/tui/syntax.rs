use std::sync::LazyLock;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

pub static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
pub static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);
pub const THEME_NAME: &str = "base16-ocean.dark";
