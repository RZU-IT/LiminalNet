use super::*;

pub(super) fn welcome(frame: &mut Frame, app: &App, area: Rect, p: Palette) {
    let panel = center(area, 72, 20);
    let border = block("", p, false);
    let inner = border.inner(panel);
    frame.render_widget(border, panel);
    let rows = Layout::vertical([
        Constraint::Length(9),
        Constraint::Length(3),
        Constraint::Min(1),
    ])
    .split(inner);
    let art = vec![
        Line::from("       /|"),
        Line::from("      / |"),
        Line::from("     |  |      LiminalNet"),
        Line::from(vec![
            Span::styled("  ~~~", Style::default().fg(p.accent)),
            Span::raw("|  |"),
        ]),
        Line::from(vec![
            Span::styled(" ~~~~", Style::default().fg(p.outgoing)),
            Span::raw("|  |      by rzu informatique"),
        ]),
        Line::from("     | /"),
        Line::from("     |/"),
    ];
    frame.render_widget(Paragraph::new(art), center(rows[0], 42, 7));
    frame.render_widget(
        Paragraph::new("Welcome to the Threshold")
            .alignment(Alignment::Center)
            .style(Style::default().fg(p.accent)),
        rows[1],
    );
    frame.render_widget(
        Paragraph::new(app.tr("press_enter_to_continue")).alignment(Alignment::Center),
        rows[2],
    );
}
