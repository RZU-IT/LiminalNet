use super::*;

pub(super) fn overview(frame: &mut Frame, app: &App, area: Rect, p: Palette) {
    let Some(session) = &app.session else {
        return;
    };
    let layout = Layout::vertical([
        Constraint::Percentage(if app.compact { 100 } else { 46 }),
        Constraint::Percentage(if app.compact { 0 } else { 54 }),
    ])
    .split(area);
    let top = Layout::horizontal([Constraint::Percentage(27), Constraint::Percentage(73)])
        .split(layout[0]);
    let packets = app.config.unit == Unit::Packets;
    let totals = session.totals;
    let received = if packets {
        totals.received_packets
    } else {
        totals.received
    };
    let sent = if packets {
        totals.sent_packets
    } else {
        totals.sent
    };
    let summary_block = block(app.tr("statistics"), p, false);
    let inner = summary_block.inner(top[0]);
    frame.render_widget(summary_block, top[0]);
    let summary_layout = Layout::vertical([Constraint::Length(4), Constraint::Min(3)]).split(inner);
    let adapter = app
        .devices
        .iter()
        .find(|d| d.name == app.config.adapter)
        .map(|d| d.description.as_str())
        .unwrap_or(&app.config.adapter);
    frame.render_widget(
        Paragraph::new(format!(
            "{}\n{}\n[u] {:?} · BPF: {}",
            app.tr("network_adapter"),
            clean(adapter),
            app.config.unit,
            if app.filter.is_empty() {
                "—"
            } else {
                &app.filter
            }
        )),
        summary_layout[0],
    );
    let totals_layout =
        Layout::horizontal([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(summary_layout[1]);
    use ratatui::widgets::canvas::{Canvas, Points};
    let ratio = received as f64 / (received + sent).max(1) as f64;
    let mut rx_points = vec![];
    let mut tx_points = vec![];
    for point in 0..360 {
        let angle = (point as f64 - 90.0).to_radians();
        let coordinate = (angle.cos() * 8.0, angle.sin() * 8.0);
        if point as f64 / 360.0 < ratio {
            rx_points.push(coordinate);
        } else {
            tx_points.push(coordinate);
        }
    }
    frame.render_widget(
        Canvas::default()
            .background_color(p.panel)
            .marker(Marker::Braille)
            .x_bounds([-12.0, 12.0])
            .y_bounds([-10.0, 10.0])
            .paint(|context| {
                context.draw(&Points {
                    coords: &rx_points,
                    color: p.accent,
                });
                context.draw(&Points {
                    coords: &tx_points,
                    color: p.outgoing,
                });
                context.print(
                    -5.0,
                    0.0,
                    Line::styled(
                        amount(totals.value(packets), app.config.unit, false),
                        Style::default().fg(p.text),
                    ),
                );
            }),
        totals_layout[0],
    );
    frame.render_widget(
        Paragraph::new(vec![
            Line::styled(
                format!("↓ {}", amount(received, app.config.unit, false)),
                Style::default().fg(p.accent),
            ),
            Line::styled(
                format!("↑ {}", amount(sent, app.config.unit, false)),
                Style::default().fg(p.outgoing),
            ),
            Line::from(format!("× {} pkt", session.dropped)),
        ]),
        totals_layout[1],
    );
    let scale = if app.config.unit == Unit::Bits {
        8.0
    } else {
        1.0
    };
    let incoming: Vec<_> = session
        .history
        .iter()
        .map(|s| {
            (
                s.time,
                if packets {
                    s.received_packets
                } else {
                    s.received * scale
                },
            )
        })
        .collect();
    let outgoing: Vec<_> = session
        .history
        .iter()
        .map(|s| {
            (
                s.time,
                -if packets {
                    s.sent_packets
                } else {
                    s.sent * scale
                },
            )
        })
        .collect();
    let maximum = incoming.iter().map(|p| p.1).fold(1.0, f64::max) * 1.1;
    let minimum = outgoing.iter().map(|p| p.1).fold(-1.0, f64::min) * 1.1;
    let end = session.elapsed.max(30.0);
    let begin = (end - 30.0).max(0.0);
    let elapsed = |time: f64| format!("{:02}:{:02}", time as u64 / 60, time as u64 % 60);
    let y_label = |value: f64| {
        if packets {
            format!("{value:.0} pkt/s")
        } else {
            amount((value.abs() / scale) as u64, app.config.unit, true)
        }
    };
    let chart = Chart::new(vec![
        Dataset::default()
            .name(app.tr("incoming"))
            .marker(Marker::Braille)
            .graph_type(GraphType::Area)
            .fill_to_y(0.0)
            .style(Style::default().fg(p.accent))
            .data(&incoming),
        Dataset::default()
            .name(app.tr("outgoing"))
            .marker(Marker::Braille)
            .graph_type(GraphType::Area)
            .fill_to_y(0.0)
            .style(Style::default().fg(p.outgoing))
            .data(&outgoing),
    ])
    .style(Style::default().bg(p.panel).fg(p.text))
    .block(block(app.tr("traffic_rate_last_30_seconds"), p, false))
    .x_axis(
        Axis::default()
            .bounds([begin, end])
            .labels([elapsed(begin), elapsed((begin + end) / 2.0), elapsed(end)])
            .style(Style::default().fg(p.text)),
    )
    .y_axis(
        Axis::default()
            .bounds([minimum, maximum])
            .labels([y_label(minimum), "0".into(), y_label(maximum)])
            .style(Style::default().fg(p.text)),
    );
    frame.render_widget(chart, top[1]);
    let mut bottom: Vec<_> = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
    ])
    .split(layout[1])
    .to_vec();
    if area.width < 100 {
        for (index, rect) in bottom.iter_mut().enumerate() {
            *rect = if index == app.focus {
                layout[1]
            } else {
                Rect::default()
            };
        }
    }
    for index in 0..3 {
        if bottom[index].width == 0 {
            continue;
        }
        let rows: Vec<_> = app
            .panel_rows(index)
            .into_iter()
            .map(|(name, rx, tx, time)| (name, (rx, tx, time)))
            .collect();
        let max = rows
            .iter()
            .map(|(_, v)| v.0 + v.1)
            .max()
            .unwrap_or(1)
            .max(1);
        let visible = bottom[index].height.saturating_sub(2) as usize / 2;
        let offset = if app.focus == index {
            app.flow_index.saturating_sub(visible.saturating_sub(1))
        } else {
            0
        };
        let items = rows
            .iter()
            .enumerate()
            .skip(offset)
            .take(visible)
            .flat_map(|(row_index, (name, v))| {
                let width = (bottom[index].width.saturating_sub(4)) as usize;
                let n = ((v.0 + v.1) as f64 / max as f64 * width as f64).round() as usize;
                let rx = if v.0 + v.1 > 0 {
                    (n as f64 * v.0 as f64 / (v.0 + v.1) as f64).round() as usize
                } else {
                    0
                };
                let label = if index == 0 {
                    name.parse()
                        .ok()
                        .and_then(|ip| {
                            app.enrichment
                                .as_ref()
                                .map(|enrichment| enrichment.label(ip))
                        })
                        .unwrap_or_else(|| name.clone())
                } else {
                    name.clone()
                };
                let total = amount(v.0 + v.1, app.config.unit, false);
                let room = bottom[index].width.saturating_sub(6 + total.len() as u16) as usize;
                let mut short = String::new();
                let mut used = 0;
                for c in clean(&label).chars() {
                    let width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
                    if used + width > room {
                        break;
                    }
                    short.push(c);
                    used += width;
                }
                let mut bars = vec![];
                for (count, color) in [(rx, p.accent), (n.saturating_sub(rx), p.outgoing)] {
                    for point in 0..count {
                        let color = if app.config.gradients {
                            if let Color::Rgb(r, g, b) = color {
                                let scale = 0.4 + 0.6 * (point + 1) as f64 / count.max(1) as f64;
                                Color::Rgb(
                                    (r as f64 * scale) as u8,
                                    (g as f64 * scale) as u8,
                                    (b as f64 * scale) as u8,
                                )
                            } else {
                                color
                            }
                        } else {
                            color
                        };
                        bars.push(Span::styled("━", Style::default().fg(color)));
                    }
                }
                vec![
                    ListItem::new(format!(
                        "{} {}{} {}",
                        if app.config.favorites.contains(&format!("{index}:{name}")) {
                            "★"
                        } else {
                            "☆"
                        },
                        short,
                        " ".repeat(room.saturating_sub(used)),
                        total
                    ))
                    .style(
                        if app.focus == index && app.flow_index == row_index {
                            Style::default().bg(Color::Rgb(53, 69, 98))
                        } else {
                            Style::default()
                        },
                    ),
                    ListItem::new(Line::from(bars)),
                ]
            })
            .collect::<Vec<_>>();
        let title = match index {
            0 => app.tr("network_hosts"),
            1 => app.tr("services"),
            _ => app.tr("programs"),
        };
        frame.render_widget(
            List::new(items)
                .style(Style::default().bg(p.panel).fg(p.text))
                .block(
                    block("", p, app.focus == index)
                        .title(
                            Line::styled(
                                if app.only_favorites[index] {
                                    " ★ "
                                } else {
                                    " ▽ "
                                },
                                Style::default().fg(if app.only_favorites[index] {
                                    p.accent
                                } else {
                                    p.text
                                }),
                            )
                            .alignment(Alignment::Left),
                        )
                        .title(
                            Line::from(if index == 0 {
                                format!(
                                    "{title} · g:{}",
                                    ["IP", "Country", "ASN", "Domain"][app.host_group]
                                )
                            } else {
                                title.to_string()
                            })
                            .alignment(Alignment::Center),
                        )
                        .title(
                            Line::styled(
                                [" ↕ ", " ▼ ", " ▲ "][app.sorting[index]],
                                Style::default().fg(if app.sorting[index] > 0 {
                                    p.accent
                                } else {
                                    p.text
                                }),
                            )
                            .alignment(Alignment::Right),
                        ),
                ),
            bottom[index],
        );
    }
}
