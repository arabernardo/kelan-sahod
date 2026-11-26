//! Window layout modes. Pure module: no Tauri, no I/O.
//!
//! - `Full`: wide landscape window with the two-column layout (countdown + holidays).
//! - `Compact`: small portrait window with just the countdown, so it can sit in a corner.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LayoutMode {
    Compact,
    #[default]
    Full,
}

/// Window sizes in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowSpec {
    pub width: f64,
    pub height: f64,
    pub min_width: f64,
    pub min_height: f64,
    /// `None` means no upper limit.
    pub max: Option<(f64, f64)>,
}

pub fn window_spec(mode: LayoutMode) -> WindowSpec {
    match mode {
        LayoutMode::Full => WindowSpec {
            width: 960.0,
            height: 640.0,
            min_width: 720.0,
            min_height: 520.0,
            max: None,
        },
        // The max keeps a compact window compact: it can't be dragged wide enough to
        // switch the UI into the two-column layout.
        LayoutMode::Compact => WindowSpec {
            width: 380.0,
            height: 540.0,
            min_width: 320.0,
            min_height: 460.0,
            max: Some((560.0, 760.0)),
        },
    }
}

/// Reads the saved layout out of the settings file's text (`settings.json`, a flat JSON
/// object written by tauri-plugin-store). Anything missing or unreadable means `Full`.
pub fn layout_from_settings_json(text: &str) -> LayoutMode {
    serde_json::from_str::<serde_json::Value>(text)
        .ok()
        .and_then(|settings| settings.get("layout").cloned())
        .and_then(|value| serde_json::from_value::<LayoutMode>(value).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_is_the_default() {
        assert_eq!(LayoutMode::default(), LayoutMode::Full);
    }

    #[test]
    fn serializes_as_lowercase_names_for_the_frontend() {
        assert_eq!(serde_json::to_string(&LayoutMode::Compact).unwrap(), "\"compact\"");
        assert_eq!(serde_json::to_string(&LayoutMode::Full).unwrap(), "\"full\"");
        assert_eq!(serde_json::from_str::<LayoutMode>("\"compact\"").unwrap(), LayoutMode::Compact);
    }

    #[test]
    fn full_is_landscape_and_compact_is_portrait() {
        let full = window_spec(LayoutMode::Full);
        let compact = window_spec(LayoutMode::Compact);
        assert!(full.width > full.height);
        assert!(compact.width < compact.height);
        assert!(compact.width < full.width && compact.height <= full.height);
    }

    #[test]
    fn sizes_respect_their_own_limits() {
        for mode in [LayoutMode::Full, LayoutMode::Compact] {
            let s = window_spec(mode);
            assert!(s.min_width <= s.width && s.min_height <= s.height, "{mode:?} min");
            if let Some((max_w, max_h)) = s.max {
                assert!(s.width <= max_w && s.height <= max_h, "{mode:?} max");
            }
        }
    }

    #[test]
    fn compact_never_reaches_the_two_column_breakpoint() {
        // style.css switches to two columns at 720px wide.
        let compact = window_spec(LayoutMode::Compact);
        assert!(compact.max.unwrap().0 < 720.0);
        // ...and the full window can never be smaller than that.
        assert!(window_spec(LayoutMode::Full).min_width >= 720.0);
    }

    #[test]
    fn reads_the_saved_layout_from_settings_text() {
        assert_eq!(
            layout_from_settings_json(r#"{ "schedule": { "firstDay": 15, "secondDay": 31 }, "layout": "compact" }"#),
            LayoutMode::Compact
        );
        assert_eq!(layout_from_settings_json(r#"{ "layout": "full" }"#), LayoutMode::Full);
    }

    #[test]
    fn missing_or_bad_settings_fall_back_to_full() {
        assert_eq!(layout_from_settings_json("{}"), LayoutMode::Full);
        assert_eq!(layout_from_settings_json(r#"{ "layout": "tiny" }"#), LayoutMode::Full);
        assert_eq!(layout_from_settings_json(r#"{ "layout": 3 }"#), LayoutMode::Full);
        assert_eq!(layout_from_settings_json("not json"), LayoutMode::Full);
        assert_eq!(layout_from_settings_json(""), LayoutMode::Full);
    }
}
