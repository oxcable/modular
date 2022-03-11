use eframe::egui::{self, FontData, FontDefinitions, FontFamily, FontId, Style, TextStyle};

pub(crate) fn configure_fonts(ctx: &egui::Context) {
    // Load font data:
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "SpaceMonoBoldItalic".to_owned(),
        FontData::from_static(include_bytes!("../../assets/SpaceMono-BoldItalic.ttf")),
    );
    fonts.font_data.insert(
        "SpaceGrotesk".to_owned(),
        FontData::from_static(include_bytes!("../../assets/SpaceGrotesk-Medium.ttf")),
    );

    // Define font families:
    let title = FontFamily::Name("title".into());
    let regular = FontFamily::Name("regular".into());
    fonts
        .families
        .insert(title.clone(), vec!["SpaceMonoBoldItalic".to_owned()]);
    fonts
        .families
        .insert(regular.clone(), vec!["SpaceGrotesk".to_owned()]);
    ctx.set_fonts(fonts);

    // Configure default fonts:
    let mut style: Style = (*ctx.style()).clone();
    style
        .text_styles
        .insert(TextStyle::Heading, FontId::new(28.0, title));
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(18.0, regular.clone()));
    style
        .text_styles
        .insert(TextStyle::Small, FontId::new(14.0, regular));
    ctx.set_style(style);
}
