//! The ready-made themes from the template's `themes/` folder, built in. `library.json`'s
//! `"preset"` picks one by name; the frame draws it after its own styles, and `"theme"` tokens
//! still win over it. When `library.json` lists `"presets"`, the player chooses between them in
//! the Settings tab, and the choice is saved as `frame.preset`.

/// Each theme's name, the label the player sees, and its stylesheet.
pub const THEMES: &[(&str, &str, &str)] = &[
    ("win98", "Windows 98", include_str!("../../../templates/library-page/themes/win98.css")),
    ("xp", "Windows XP", include_str!("../../../templates/library-page/themes/xp.css")),
    ("vista", "Windows Vista", include_str!("../../../templates/library-page/themes/vista.css")),
    ("system7", "System 7", include_str!("../../../templates/library-page/themes/system7.css")),
    ("aqua", "Mac OS X Aqua", include_str!("../../../templates/library-page/themes/aqua.css")),
    ("beos", "BeOS", include_str!("../../../templates/library-page/themes/beos.css")),
    ("nextstep", "NeXTSTEP", include_str!("../../../templates/library-page/themes/nextstep.css")),
    ("cde", "CDE", include_str!("../../../templates/library-page/themes/cde.css")),
    ("gnome2", "GNOME 2", include_str!("../../../templates/library-page/themes/gnome2.css")),
    ("amiga", "Amiga Workbench", include_str!("../../../templates/library-page/themes/amiga.css")),
    ("atari", "Atari ST GEM", include_str!("../../../templates/library-page/themes/atari.css")),
];

/// A theme's stylesheet by name, or `None` for a name that is not built in.
pub fn theme(name: &str) -> Option<&'static str> {
    THEMES.iter().find(|(n, _, _)| *n == name).map(|(_, _, css)| *css)
}

/// A theme's label, or its name for one that is not built in.
pub fn theme_label(name: &str) -> &str {
    match name {
        "none" => "The frame's own",
        _ => THEMES.iter().find(|(n, _, _)| *n == name).map_or(name, |(_, label, _)| *label),
    }
}

/// The theme to draw: the player's choice when `library.json` offers it, else its `preset`.
pub fn active_theme(spec: &gameframe_library::Spec, chosen: Option<&str>) -> Option<String> {
    let name = match chosen {
        Some(c) if spec.presets.iter().any(|p| p == c) => Some(c.to_string()),
        _ => spec.preset.clone(),
    };
    name.filter(|n| n != "none")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_players_choice_wins_only_when_offered() {
        let mut spec = gameframe_library::Spec::from_json(include_str!("../../../templates/library-page/library.json")).unwrap();
        spec.preset = Some("win98".into());
        assert_eq!(active_theme(&spec, Some("amiga")).as_deref(), Some("win98"), "not offered");
        spec.presets = vec!["amiga".into(), "none".into()];
        assert_eq!(active_theme(&spec, Some("amiga")).as_deref(), Some("amiga"));
        assert_eq!(active_theme(&spec, Some("none")), None);
        assert_eq!(active_theme(&spec, None).as_deref(), Some("win98"));
        assert!(THEMES.iter().all(|(n, _, css)| theme(n) == Some(*css) && theme_label(n) != *n));
    }
}
