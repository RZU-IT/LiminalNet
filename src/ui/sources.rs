use super::*;

pub(super) fn sources(frame: &mut Frame, app: &App, area: Rect, p: Palette) {
    let panels =
        Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).split(area);
    let start = app
        .selected
        .saturating_sub((panels[0].height.saturating_sub(2) as usize / 3).saturating_sub(1));
    let rows = app
        .devices
        .iter()
        .enumerate()
        .skip(start)
        .flat_map(|(i, d)| {
            let selected = i == app.selected;
            let style = Style::default().fg(if selected { p.accent } else { p.text });
            vec![
                ListItem::new(Line::styled(
                    format!(
                        "{} {}",
                        if selected { "▶" } else { " " },
                        clean(&d.description)
                    ),
                    style,
                )),
                ListItem::new(format!(
                    "  {}",
                    d.addresses
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(" · ")
                )),
                ListItem::new({
                    let history = app.previews.get(&d.name).or_else(|| {
                        d.addresses
                            .iter()
                            .find_map(|ip| app.previews.get(&ip.to_string()))
                    });
                    if let Some(history) = history {
                        let max = history.iter().copied().max().unwrap_or(1).max(1);
                        let bars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
                        let graph: String = history
                            .iter()
                            .map(|value| {
                                bars[((*value as f64 / max as f64) * 7.0).round() as usize]
                            })
                            .collect();
                        format!(
                            "  {graph}  {}",
                            amount(history.back().copied().unwrap_or(0), Unit::Bytes, true)
                        )
                    } else {
                        app.tr("measuring_activity").into()
                    }
                })
                .style(Style::default().fg(p.accent)),
            ]
        })
        .collect::<Vec<_>>();
    frame.render_widget(
        List::new(rows).block(block(app.tr("traffic_source_network_adapters"), p, true)),
        panels[0],
    );
    let text = format!(
        "{}\n\n[f] BPF: {}\n\n[i] File: {}\n\n[e] Export: {}\n\n[Enter] {}\n\n{}",
        app.tr("select_an_interface_or_capture_file"),
        if app.filter.is_empty() {
            "—"
        } else {
            &app.filter
        },
        if app.file.is_empty() {
            "—"
        } else {
            &app.file
        },
        if app.export.is_empty() {
            "—"
        } else {
            &app.export
        },
        app.tr("start"),
        app.tr("traffic_previews_are_available_per_interface")
    );
    frame.render_widget(
        Paragraph::new(text).wrap(Wrap { trim: true }).block(block(
            app.tr("capture_options"),
            p,
            false,
        )),
        panels[1],
    );
}
