use super::*;

pub(super) fn settings(frame: &mut Frame, app: &App, area: Rect, p: Palette) {
    let state = |value| {
        app.tr(if value { "enabled" } else { "disabled" })
            .to_string()
    };
    let database = |path: &str| {
        if path.is_empty() {
            app.tr("embedded_database").to_string()
        } else {
            path.to_string()
        }
    };
    let text = crate::i18n::template(
        "settings_template",
        app.config.french,
        &[
            (
                "language",
                if app.config.french {
                    "Français"
                } else {
                    "English"
                }
                .into(),
            ),
            (
                "theme",
                ["LiminalNet", "Dracula", "Nord", "Light"][app.config.theme].into(),
            ),
            ("palette", app.config.custom_colors.join(",")),
            ("gradients", state(app.config.gradients)),
            ("welcome", state(app.config.show_welcome)),
            ("autostart", state(app.config.autostart)),
            ("capture", state(app.config.auto_capture)),
            ("threshold", app.config.threshold.to_string()),
            ("threshold_unit", format!("{:?}", app.config.threshold_unit)),
            ("blacklist_alert", state(app.config.alert_blacklist)),
            ("favorite_alert", state(app.config.alert_favorites)),
            ("sound", state(app.config.alert_sound)),
            (
                "webhook",
                app.tr(if app.config.webhook.is_empty() {
                    "disabled"
                } else {
                    "configured_hidden"
                })
                .into(),
            ),
            ("country", database(&app.config.country_database)),
            ("asn", database(&app.config.asn_database)),
            (
                "blacklist",
                if app.config.blacklist.is_empty() {
                    app.tr("none").into()
                } else {
                    app.config.blacklist.clone()
                },
            ),
        ],
    );
    let visible = area.height.saturating_sub(2) as usize;
    let offset = app
        .settings_choice
        .saturating_sub(visible.saturating_sub(1));
    let items = text.lines().enumerate().skip(offset).map(|(index, line)| {
        ListItem::new(clean(line)).style(if index == app.settings_choice {
            Style::default().bg(p.accent).fg(p.background)
        } else {
            Style::default().fg(p.text)
        })
    });
    frame.render_widget(
        List::new(items).block(block(app.tr("settings_general_appearance"), p, true)),
        area,
    );
}
