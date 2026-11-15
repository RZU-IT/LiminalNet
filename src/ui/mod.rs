mod welcome;
use welcome::welcome;
mod sources;
use sources::sources;
mod overview;
use overview::overview;
mod inspect;
use inspect::inspect;
mod settings;
use settings::settings;
mod about;
use about::about;

use crate::{
    app::{App, Input, Screen},
    config::{Unit, amount, clean},
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    symbols::Marker,
    text::{Line, Span},
    widgets::{
        Axis, Block, Borders, Chart, Clear, Dataset, GraphType, List, ListItem, Paragraph, Row,
        Table, Wrap,
    },
};

#[derive(Clone, Copy)]
struct Palette {
    background: Color,
    panel: Color,
    accent: Color,
    outgoing: Color,
    text: Color,
}
fn palette(theme: usize) -> Palette {
    match theme {
        1 => Palette {
            background: Color::Rgb(40, 42, 54),
            panel: Color::Rgb(53, 55, 69),
            accent: Color::Rgb(255, 121, 198),
            outgoing: Color::Rgb(139, 233, 253),
            text: Color::Rgb(248, 248, 242),
        },
        2 => Palette {
            background: Color::Rgb(46, 52, 64),
            panel: Color::Rgb(59, 66, 82),
            accent: Color::Rgb(163, 190, 140),
            outgoing: Color::Rgb(136, 192, 208),
            text: Color::Rgb(236, 239, 244),
        },
        3 => Palette {
            background: Color::Rgb(245, 245, 245),
            panel: Color::White,
            accent: Color::Rgb(170, 30, 30),
            outgoing: Color::Rgb(0, 78, 204),
            text: Color::Rgb(8, 16, 32),
        },
        _ => Palette {
            background: Color::Rgb(15, 30, 60),
            panel: Color::Rgb(21, 36, 67),
            accent: Color::Rgb(252, 182, 8),
            outgoing: Color::Rgb(11, 177, 252),
            text: Color::Rgb(221, 221, 221),
        },
    }
}
fn block(title: impl Into<String>, p: Palette, active: bool) -> Block<'static> {
    Block::bordered()
        .title(title.into())
        .title_style(Style::default().fg(p.accent))
        .border_style(if active {
            p.accent
        } else {
            Color::Rgb(53, 69, 98)
        })
        .style(Style::default().bg(p.panel).fg(p.text))
}
fn center(area: Rect, width: u16, height: u16) -> Rect {
    Rect::new(
        area.x + area.width.saturating_sub(width) / 2,
        area.y + area.height.saturating_sub(height) / 2,
        width.min(area.width),
        height.min(area.height),
    )
}
pub fn draw(frame: &mut Frame, app: &App) {
    let mut p = palette(app.config.theme);
    let colors = if app.input == Input::Palette {
        app.buffer
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    } else {
        app.config.custom_colors.clone()
    };
    if colors.len() == 4 {
        let parse = |text: &str| {
            u32::from_str_radix(text.trim_start_matches('#'), 16)
                .ok()
                .map(|v| Color::Rgb((v >> 16) as u8, (v >> 8) as u8, v as u8))
        };
        if let (Some(background), Some(accent), Some(outgoing), Some(text)) = (
            parse(&colors[0]),
            parse(&colors[1]),
            parse(&colors[2]),
            parse(&colors[3]),
        ) {
            p = Palette {
                background,
                panel: background,
                accent,
                outgoing,
                text,
            };
        }
    }
    let area = frame.area();
    frame.render_widget(
        Block::default().style(Style::default().bg(p.background).fg(p.text)),
        area,
    );
    if area.width < 55 || area.height < 16 {
        frame.render_widget(
            Paragraph::new(app.tr("resize_the_terminal_at_least_55_16")).wrap(Wrap { trim: true }),
            area,
        );
        return;
    }
    let sections = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(area);
    let title = if app.session.is_none() {
        "▱  LiminalNet · by rzu informatique".to_string()
    } else {
        format!(
            "← Cards    ▱  LiminalNet · by rzu informatique    [Space] {}",
            if app.session.is_some() {
                if app.opening {
                    app.tr("opening_capture")
                } else if app.finished {
                    app.tr("complete")
                } else if app.paused {
                    app.tr("paused")
                } else {
                    app.tr("capturing")
                }
            } else {
                ""
            }
        )
    };
    frame.render_widget(
        Paragraph::new(title)
            .alignment(Alignment::Center)
            .style(Style::default().bg(p.accent).fg(p.background))
            .block(Block::default().borders(Borders::TOP | Borders::BOTTOM)),
        sections[0],
    );
    match app.screen {
        Screen::Welcome => welcome(frame, app, sections[1], p),
        Screen::Sources => sources(frame, app, sections[1], p),
        Screen::Overview | Screen::Inspect | Screen::Alerts => {
            let split =
                Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).split(sections[1]);
            let tabs = Layout::horizontal([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(split[0]);
            for (index, label) in [
                app.tr("1_overview").to_string(),
                app.tr("2_inspect").to_string(),
                format!("[3] {} ({})", app.tr("notifications"), app.events.len()),
            ]
            .into_iter()
            .enumerate()
            {
                let active = matches!(
                    (index, app.screen),
                    (0, Screen::Overview) | (1, Screen::Inspect) | (2, Screen::Alerts)
                );
                frame.render_widget(
                    Paragraph::new(label)
                        .alignment(Alignment::Center)
                        .style(Style::default().fg(if active { p.accent } else { p.text }))
                        .block(block("", p, active)),
                    tabs[index],
                );
            }
            match app.screen {
                Screen::Overview => overview(frame, app, split[1], p),
                Screen::Inspect => inspect(frame, app, split[1], p),
                _ => {
                    frame.render_widget(
                        List::new(
                            app.events
                                .iter()
                                .skip(app.flow_index)
                                .map(|e| ListItem::new(clean(e)))
                                .collect::<Vec<_>>(),
                        )
                        .block(block(
                            app.tr("notifications_c_clear"),
                            p,
                            false,
                        )),
                        split[1],
                    );
                }
            }
        }
        Screen::Settings => settings(frame, app, sections[1], p),
        Screen::About => about(frame, app, sections[1], p),
        Screen::Firewall => {
            let rows = app.config.rules.iter().enumerate().map(|(index, rule)| {
                ListItem::new(format!(
                    "{} {}\n  {:?}\n  {}",
                    if index == app.flow_index { "▶" } else { " " },
                    rule.id,
                    rule.target,
                    rule.scope
                ))
            });
            frame.render_widget(
                List::new(rows).block(block(
                    "Firewall · recorded rules · d remove · Esc back",
                    p,
                    true,
                )),
                sections[1],
            );
        }
    }
    let footer = if app.screen == Screen::Welcome {
        app.tr("press_enter_to_continue")
    } else if app.screen == Screen::Sources {
        app.tr("select_enter_start_f_filter_i_file_e_export_r_refresh_s_settings_help_")
    } else {
        app.tr("esc_adapters_space_pause_u_units_tab_panel_f_favorites_t_sort_search_s")
    };
    frame.render_widget(
        Paragraph::new(if app.opening {
            let position = app
                .session
                .as_ref()
                .map(|s| (s.started.elapsed().as_millis() / 100) % 17)
                .unwrap_or(0) as usize;
            format!(
                "[{}▰▰▰{}] {}",
                " ".repeat(position),
                " ".repeat(17 - position),
                app.tr("opening_capture")
            )
        } else if app.status.is_empty() {
            footer.into()
        } else {
            clean(&app.status)
        })
        .style(Style::default().fg(if app.status.is_empty() {
            p.text
        } else {
            Color::LightRed
        })),
        sections[2],
    );
    frame.render_widget(
        Block::default().style(Style::default().bg(p.accent)),
        sections[3],
    );
    let footer_panels = Layout::horizontal([
        Constraint::Percentage(25),
        Constraint::Percentage(50),
        Constraint::Percentage(25),
    ])
    .split(sections[3]);
    for (index, text) in [
        format!(" LiminalNet {}", env!("CARGO_PKG_VERSION")),
        format!(
            "[F2] Git {}",
            if crate::links::REPOSITORY.is_some() {
                "↗"
            } else {
                "—"
            }
        ),
        "by RZU Informatique ".into(),
    ]
    .into_iter()
    .enumerate()
    {
        frame.render_widget(
            Paragraph::new(text)
                .alignment([Alignment::Left, Alignment::Center, Alignment::Right][index])
                .style(Style::default().bg(p.accent).fg(p.background)),
            footer_panels[index],
        );
    }
    if app.confirm {
        popup(
            frame,
            app.tr("quit_analysis"),
            app.tr("capture_will_stop_and_session_statistics_will_be_cleared_the_recorded_"),
            p,
        );
    }
    if let Some((remove, rule)) = &app.pending_rule {
        popup(
            frame,
            "Confirm firewall operation",
            &format!(
                "{}\n\nTarget: {:?}\n\nAdministrative permissions are required.\nLinux program rules affect only a dedicated current cgroup.\n\n[Enter / Esc] Cancel       [y] Apply",
                if *remove {
                    "Remove this LiminalNet rule?"
                } else {
                    "Install a real network blocking rule?"
                },
                rule.target
            ),
            p,
        );
    }
    if app.help {
        popup(frame, "LiminalNet", app.tr("help_body"), p);
    }
    if app.details
        && let Some(flow) = app.details_flow()
    {
        popup(
            frame,
            app.tr("connection_details"),
            &format!(
                "{}:{} → {}:{}\n\nProtocol: {}   Service (inferred): {}\nBytes: {}   Packets: {}\nFirst timestamp (Unix): {:.6}\nLast timestamp (Unix): {:.6}\nMAC: {} → {}\n{}\nProcess: {}\nExecutable: {}\nHost: {}\n\n[p] Measure RTT (sends one ping)\n{}\nEsc closes details.",
                flow.key.source,
                flow.key.source_port,
                flow.key.destination,
                flow.key.destination_port,
                flow.key.protocol,
                flow.service,
                flow.bytes,
                flow.packets,
                flow.first,
                flow.last,
                flow.source_mac.as_deref().unwrap_or("—"),
                flow.destination_mac.as_deref().unwrap_or("—"),
                clean(&flow.protocol_details),
                clean(&flow.program),
                clean(&flow.program_path),
                app.enrichment
                    .as_ref()
                    .map(|e| e.label(if flow.incoming {
                        flow.key.source
                    } else {
                        flow.key.destination
                    }))
                    .unwrap_or_default(),
                app.probe_message
            ),
            p,
        );
    }
    if app.input != Input::None {
        popup(
            frame,
            match app.input {
                Input::Filter => app.tr("edit_filter"),
                Input::File => app.tr("edit_file"),
                Input::Export => app.tr("edit_export"),
                Input::Search => app.tr("edit_search"),
                Input::Threshold => app.tr("edit_threshold"),
                Input::Country => app.tr("edit_country"),
                Input::Asn => app.tr("edit_asn"),
                Input::Blacklist => app.tr("edit_blacklist"),
                Input::Webhook => app.tr("edit_webhook"),
                Input::Palette => app.tr("edit_palette"),
                Input::None => "",
            },
            &format!("{}▏\n\nEnter: apply · Esc: cancel", clean(&app.buffer)),
            p,
        );
    }
}
fn popup(frame: &mut Frame, title: &str, text: &str, p: Palette) {
    let area = center(
        frame.area(),
        80,
        frame.area().height.saturating_sub(4).min(32),
    );
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(text)
            .wrap(Wrap { trim: false })
            .block(block(title, p, true)),
        area,
    );
}
