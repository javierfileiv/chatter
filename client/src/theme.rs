use cursive::theme::{BorderStyle, Color, Palette, PaletteColor, Theme};

pub fn create_retro_theme() -> Theme {
    let mut theme = cursive::theme::Theme {
        shadow: true,
        borders: BorderStyle::Simple,
        ..Default::default()
    };

    let mut palette = Palette::default();
    palette[PaletteColor::Background] = Color::Rgb(0, 0, 20);
    palette[PaletteColor::View] = Color::Rgb(0, 0, 20);
    palette[PaletteColor::Primary] = Color::Rgb(0, 255, 0);
    palette[PaletteColor::TitlePrimary] = Color::Rgb(0, 255, 128);
    palette[PaletteColor::Secondary] = Color::Rgb(255, 191, 0);
    palette[PaletteColor::Highlight] = Color::Rgb(0, 255, 255);
    palette[PaletteColor::HighlightInactive] = Color::Rgb(0, 128, 128);
    palette[PaletteColor::Shadow] = Color::Rgb(0, 0, 40);
    theme.palette = palette;
    theme
}
