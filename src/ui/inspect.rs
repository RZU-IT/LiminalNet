use super::*;

pub(super) fn inspect(frame: &mut Frame, app: &App, area: Rect, p: Palette) {
    let layout = Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).split(area);
    frame.render_widget(
        Paragraph::new(format!(
            "/ {} · [f] ★ {} · [l] IP blacklist {}",
            clean(&app.search),
            app.inspect_favorites,
            app.inspect_blacklist
        ))
        .block(block(app.tr("search"), p, false)),
        layout[0],
    );
    let flows = app.flows();
    let visible = layout[1].height.saturating_sub(3) as usize;
    let offset = app.flow_index.saturating_sub(visible.saturating_sub(1));
    let rows = flows
        .iter()
        .enumerate()
        .skip(offset)
        .take(visible)
        .map(|(i, f)| {
            Row::new(vec![
                format!("{}:{}", f.key.source, f.key.source_port),
                format!("{}:{}", f.key.destination, f.key.destination_port),
                f.key.protocol.clone(),
                f.service.clone(),
                amount(
                    if app.config.unit == Unit::Packets {
                        f.packets
                    } else {
                        f.bytes
                    },
                    app.config.unit,
                    false,
                ),
            ])
            .style(if i == app.flow_index {
                Style::default().bg(p.accent).fg(p.background)
            } else {
                Style::default().fg(p.text)
            })
        });
    frame.render_widget(
        Table::new(
            rows,
            [
                Constraint::Percentage(32),
                Constraint::Percentage(32),
                Constraint::Length(7),
                Constraint::Length(9),
                Constraint::Min(10),
            ],
        )
        .header(
            Row::new(["Source", "Destination", "Proto", "Service", "Volume"])
                .style(Style::default().fg(p.accent)),
        )
        .block(block(
            format!("{} {} · Enter details", flows.len(), app.tr("connections")),
            p,
            true,
        )),
        layout[1],
    );
}
