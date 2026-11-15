use super::*;

impl App {
    fn edit(&mut self, input: Input, value: String) {
        self.input = input;
        self.buffer = value;
    }
    fn commit(&mut self) {
        let reload = matches!(self.input, Input::Country | Input::Asn | Input::Blacklist);
        match self.input {
            Input::Filter => self.filter = self.buffer.clone(),
            Input::File => self.file = self.buffer.clone(),
            Input::Export => self.export = self.buffer.clone(),
            Input::Search => {
                self.search = self.buffer.clone();
                self.flow_index = 0;
            }
            Input::Country => self.config.country_database = self.buffer.clone(),
            Input::Asn => self.config.asn_database = self.buffer.clone(),
            Input::Blacklist => self.config.blacklist = self.buffer.clone(),
            Input::Webhook => {
                if !self.buffer.is_empty()
                    && !self.buffer.starts_with("https://")
                    && !self.buffer.starts_with("http://127.0.0.1:")
                {
                    self.status = "Use HTTPS, or localhost HTTP for testing".into();
                    return;
                }
                self.config.webhook = self.buffer.clone();
            }
            Input::Palette => {
                let colors: Vec<_> = self
                    .buffer
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if !colors.is_empty()
                    && (colors.len() != 4
                        || colors.iter().any(|color| {
                            color.len() != 7
                                || !color.starts_with('#')
                                || u32::from_str_radix(&color[1..], 16).is_err()
                        }))
                {
                    self.status =
                        "Enter four #RRGGBB colors: background, incoming, outgoing, text".into();
                    return;
                }
                self.config.custom_colors = colors;
            }
            Input::Threshold => match self.buffer.parse() {
                Ok(value) => self.config.threshold = value,
                Err(_) => {
                    self.status = "Invalid threshold: enter bytes per second".into();
                    return;
                }
            },
            Input::None => {}
        }
        self.input = Input::None;
        self.save();
        if reload {
            self.reload_enrichment();
        }
    }
    pub fn handle(&mut self, event: Event) {
        if self.screen == Screen::Welcome
            && matches!(&event,Event::Key(key) if key.kind==KeyEventKind::Press && key.code==KeyCode::Esc)
        {
            self.config.welcomed = true;
            self.save();
            self.screen = Screen::Sources;
            return;
        }
        if self.rule_result.is_some() {
            return;
        }
        if let Event::Key(key) = event
            && self.pending_rule.is_some()
        {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('o') => {
                    if self.rule_result.is_some() {
                        return;
                    }
                    if let Some((remove, rule)) = self.pending_rule.take() {
                        let (tx, rx) = std::sync::mpsc::channel();
                        self.rule_result = Some(rx);
                        self.status = "Applying firewall operation…".into();
                        std::thread::spawn(move || {
                            let result = if remove {
                                crate::firewall::remove(&rule).map(|()| (true, rule))
                            } else {
                                crate::firewall::install(rule.target).map(|rule| (false, rule))
                            };
                            let _ = tx.send(result.map_err(|e| format!("{e:#}")));
                        });
                    }
                }
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('n') => self.pending_rule = None,
                _ => {}
            }
            return;
        }
        if let Event::Mouse(mouse) = event {
            if self.input != Input::None || self.confirm || self.help || self.details {
                return;
            }
            let key = match mouse.kind {
                MouseEventKind::ScrollUp => Some(KeyCode::Up),
                MouseEventKind::ScrollDown => Some(KeyCode::Down),
                MouseEventKind::Down(MouseButton::Left) => {
                    let (width, height) = crossterm::terminal::size().unwrap_or((120, 38));
                    if mouse.row < 3 && self.session.is_some() {
                        Some(if mouse.column < width / 3 {
                            KeyCode::Esc
                        } else {
                            KeyCode::Char(' ')
                        })
                    } else if self.screen == Screen::Settings && mouse.row >= 4 {
                        let visible = height.saturating_sub(7) as usize;
                        let offset = self
                            .settings_choice
                            .saturating_sub(visible.saturating_sub(1));
                        self.settings_choice = (offset + mouse.row.saturating_sub(4) as usize)
                            .min(SETTINGS_KEYS.len() - 1);
                        Some(KeyCode::Enter)
                    } else if self.screen == Screen::Sources {
                        if mouse.column < width * 55 / 100
                            && mouse.row >= 4
                            && mouse.row < height.saturating_sub(2)
                        {
                            let start = self.selected.saturating_sub(
                                ((height.saturating_sub(7)) as usize / 3).saturating_sub(1),
                            );
                            self.selected = (start + (mouse.row.saturating_sub(4) / 3) as usize)
                                .min(self.devices.len().saturating_sub(1));
                            None
                        } else if mouse.column >= width * 55 / 100 && mouse.row >= 4 {
                            match mouse.row {
                                6 => Some(KeyCode::Char('f')),
                                8 => Some(KeyCode::Char('i')),
                                10 => Some(KeyCode::Char('e')),
                                12 => Some(KeyCode::Enter),
                                _ => None,
                            }
                        } else {
                            None
                        }
                    } else if matches!(
                        self.screen,
                        Screen::Overview | Screen::Inspect | Screen::Alerts
                    ) && mouse.row < 6
                    {
                        Some(KeyCode::Char(if mouse.column < width / 3 {
                            '1'
                        } else if mouse.column < width * 2 / 3 {
                            '2'
                        } else {
                            '3'
                        }))
                    } else if self.screen == Screen::Overview && !self.compact {
                        use ratatui::layout::{Constraint, Layout, Rect};
                        let body = Rect::new(0, 6, width, height.saturating_sub(8));
                        let vertical = Layout::vertical([
                            Constraint::Percentage(46),
                            Constraint::Percentage(54),
                        ])
                        .split(body);
                        let panels = Layout::horizontal([
                            Constraint::Percentage(50),
                            Constraint::Percentage(25),
                            Constraint::Percentage(25),
                        ])
                        .split(vertical[1]);
                        if let Some((index, panel)) = panels
                            .iter()
                            .enumerate()
                            .find(|(_, rect)| rect.contains((mouse.column, mouse.row).into()))
                        {
                            self.focus = index;
                            if mouse.row == panel.y {
                                if mouse.column <= panel.x + 2 {
                                    Some(KeyCode::Char('f'))
                                } else if mouse.column >= panel.right().saturating_sub(3) {
                                    Some(KeyCode::Char('t'))
                                } else {
                                    None
                                }
                            } else {
                                self.flow_index =
                                    ((mouse.row.saturating_sub(panel.y + 1)) / 2) as usize;
                                if mouse.column <= panel.x + 2 {
                                    Some(KeyCode::Char('*'))
                                } else {
                                    Some(KeyCode::Enter)
                                }
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            };
            if let Some(code) = key {
                self.handle(Event::Key(KeyEvent::new(code, KeyModifiers::NONE)));
            }
            return;
        }
        let Event::Key(key) = event else {
            return;
        };
        if key.kind != KeyEventKind::Press {
            return;
        }
        if self.input != Input::None {
            match key.code {
                KeyCode::Esc => self.input = Input::None,
                KeyCode::Enter => self.commit(),
                KeyCode::Backspace => {
                    self.buffer.pop();
                }
                KeyCode::Char(c) if !c.is_control() => self.buffer.push(c),
                _ => {}
            }
            return;
        }
        if self.help {
            if matches!(key.code, KeyCode::Esc | KeyCode::Char('?')) {
                self.help = false;
            }
            return;
        }
        if self.details {
            if key.code == KeyCode::Char('p')
                && self.probe_result.is_none()
                && let Some(flow) = self.details_flow()
            {
                let ip = if flow.incoming {
                    flow.key.source
                } else {
                    flow.key.destination
                }
                .to_string();
                let (tx, rx) = std::sync::mpsc::channel();
                self.probe_result = Some(rx);
                self.probe_message = "Measuring RTT…".into();
                std::thread::spawn(move || {
                    let mut command = std::process::Command::new("ping");
                    if cfg!(windows) {
                        command.args(["-n", "1", "-w", "2000"]);
                    } else {
                        command.args(["-n", "-c", "1", "-W", "2"]);
                    }
                    command.arg(ip);
                    #[cfg(windows)]
                    {
                        use std::os::windows::process::CommandExt;
                        command.creation_flags(0x08000000);
                    }
                    let result = command
                        .output()
                        .map(|output| {
                            String::from_utf8_lossy(&output.stdout)
                                .lines()
                                .rev()
                                .take(3)
                                .collect::<Vec<_>>()
                                .into_iter()
                                .rev()
                                .collect::<Vec<_>>()
                                .join("\n")
                        })
                        .unwrap_or_else(|error| error.to_string());
                    let _ = tx.send(result);
                });
            }
            if matches!(key.code, KeyCode::Esc | KeyCode::Enter) {
                self.details = false;
            }
            return;
        }
        if self.confirm {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('o') => {
                    self.worker = None;
                    self.session = None;
                    self.screen = Screen::Sources;
                    self.confirm = false;
                    self.status.clear();
                }
                KeyCode::Esc | KeyCode::Char('n') | KeyCode::Enter => self.confirm = false,
                _ => {}
            }
            return;
        }
        match key.code {
            KeyCode::F(2) => {
                if let Err(error) = crate::links::open(crate::links::REPOSITORY) {
                    self.status = error.to_string();
                }
                return;
            }
            KeyCode::Char('?') => {
                self.help = true;
                return;
            }
            KeyCode::Char('q') => {
                if self.session.is_some() {
                    self.confirm = true;
                } else {
                    self.quit = true;
                }
                return;
            }
            KeyCode::Char('s') => {
                self.screen = Screen::Settings;
                return;
            }
            KeyCode::Char('a') => {
                self.screen = Screen::About;
                return;
            }
            KeyCode::Char('4') => {
                self.screen = Screen::Firewall;
                self.flow_index = 0;
                return;
            }
            KeyCode::Esc if self.session.is_some() => {
                if matches!(
                    self.screen,
                    Screen::Settings | Screen::About | Screen::Firewall
                ) {
                    self.screen = Screen::Overview;
                } else {
                    self.confirm = true;
                }
                return;
            }
            KeyCode::Esc => {
                self.screen = Screen::Sources;
                return;
            }
            _ => {}
        }
        match self.screen {
            Screen::Welcome => {
                if key.code == KeyCode::Enter {
                    self.config.welcomed = true;
                    self.save();
                    self.screen = Screen::Sources;
                    if self.config.auto_capture && !self.config.adapter.is_empty() {
                        self.start();
                    }
                }
            }
            Screen::Sources => match key.code {
                KeyCode::Up => self.selected = self.selected.saturating_sub(1),
                KeyCode::Down => {
                    self.selected = (self.selected + 1).min(self.devices.len().saturating_sub(1))
                }
                KeyCode::Enter => self.start(),
                KeyCode::Char('r') => self.refresh(),
                KeyCode::Char('f') => self.edit(Input::Filter, self.filter.clone()),
                KeyCode::Char('i') => self.edit(Input::File, self.file.clone()),
                KeyCode::Char('e') => self.edit(Input::Export, self.export.clone()),
                _ => {}
            },
            Screen::Settings => match key.code {
                KeyCode::Char('u') => {
                    self.config.threshold_unit = match self.config.threshold_unit {
                        Unit::Bits => Unit::Packets,
                        Unit::Packets => Unit::Bytes,
                        Unit::Bytes => Unit::Bits,
                    };
                    self.save();
                }
                KeyCode::Up => self.settings_choice = self.settings_choice.saturating_sub(1),
                KeyCode::Down => {
                    self.settings_choice = (self.settings_choice + 1).min(SETTINGS_KEYS.len() - 1)
                }
                KeyCode::Enter => {
                    self.handle(Event::Key(KeyEvent::new(
                        KeyCode::Char(SETTINGS_KEYS[self.settings_choice]),
                        KeyModifiers::NONE,
                    )));
                }
                KeyCode::Char('l') => {
                    self.config.french = !self.config.french;
                    self.save();
                }
                KeyCode::Char('t') => {
                    self.config.theme = (self.config.theme + 1) % 4;
                    self.save();
                }
                KeyCode::Char('w') => {
                    self.config.show_welcome = !self.config.show_welcome;
                    self.save();
                }
                KeyCode::Char('v') => self.screen = Screen::Welcome,
                KeyCode::Char('d') => {
                    self.edit(Input::Threshold, self.config.threshold.to_string())
                }
                KeyCode::Char('g') => {
                    self.edit(Input::Country, self.config.country_database.clone())
                }
                KeyCode::Char('j') => self.edit(Input::Asn, self.config.asn_database.clone()),
                KeyCode::Char('b') => self.edit(Input::Blacklist, self.config.blacklist.clone()),
                KeyCode::Char('h') => self.edit(Input::Webhook, self.config.webhook.clone()),
                KeyCode::Char('k') => {
                    self.edit(Input::Palette, self.config.custom_colors.join(","))
                }
                KeyCode::Char('r') => {
                    self.config.custom_colors.clear();
                    self.config.theme = 0;
                    self.save();
                }
                KeyCode::Char('c') => {
                    self.config.gradients = !self.config.gradients;
                    self.save();
                }
                KeyCode::Char('n') => {
                    self.config.alert_blacklist = !self.config.alert_blacklist;
                    self.save();
                }
                KeyCode::Char('p') => {
                    self.config.alert_favorites = !self.config.alert_favorites;
                    self.save();
                }
                KeyCode::Char('m') => {
                    self.config.alert_sound = !self.config.alert_sound;
                    self.save();
                }
                KeyCode::Char('x') => self.notify("LiminalNet test notification".into()),
                KeyCode::Char('o') => {
                    let enabled = !self.config.autostart;
                    match crate::autostart::set(enabled) {
                        Ok(()) => {
                            self.config.autostart = enabled;
                            self.save();
                        }
                        Err(error) => self.status = format!("{error:#}"),
                    }
                }
                KeyCode::Char('z') => {
                    self.config.auto_capture = !self.config.auto_capture;
                    self.save();
                }
                _ => {}
            },
            Screen::Overview | Screen::Inspect | Screen::Alerts => match key.code {
                KeyCode::Char('g') if self.screen == Screen::Overview => {
                    self.host_group = (self.host_group + 1) % 4;
                    self.flow_index = 0;
                }
                KeyCode::Char('b') | KeyCode::Char('B') if self.screen == Screen::Inspect => {
                    if let Some(flow) = self.flows().get(self.flow_index) {
                        let target = if key.code == KeyCode::Char('B') {
                            if flow.program_path.is_empty() {
                                self.status = "No executable identified for this connection".into();
                                return;
                            }
                            crate::firewall::Target::Program(flow.program_path.clone())
                        } else {
                            crate::firewall::Target::Ip(
                                if flow.incoming {
                                    flow.key.source
                                } else {
                                    flow.key.destination
                                }
                                .to_string(),
                            )
                        };
                        self.pending_rule = Some((
                            false,
                            crate::firewall::Rule {
                                id: String::new(),
                                target,
                                scope: String::new(),
                            },
                        ));
                    }
                }
                KeyCode::Char('m') => self.compact = !self.compact,
                KeyCode::Char('1') => {
                    self.screen = Screen::Overview;
                    self.flow_index = 0;
                }
                KeyCode::Char('2') => {
                    self.screen = Screen::Inspect;
                    self.flow_index = 0;
                }
                KeyCode::Char('3') => {
                    self.screen = Screen::Alerts;
                    self.flow_index = 0;
                }
                KeyCode::Char('l') if self.screen == Screen::Inspect => {
                    self.inspect_blacklist = !self.inspect_blacklist;
                    self.flow_index = 0;
                }
                KeyCode::Char('f') if self.screen == Screen::Inspect => {
                    self.inspect_favorites = !self.inspect_favorites;
                    self.flow_index = 0;
                }
                KeyCode::Char(' ') if !self.finished => {
                    if let Some(worker) = &self.worker {
                        let value = !worker.paused.load(Ordering::Relaxed);
                        worker.paused.store(value, Ordering::Relaxed);
                    }
                }
                KeyCode::Char('u') => {
                    self.config.unit = match self.config.unit {
                        Unit::Bytes => Unit::Bits,
                        Unit::Bits => Unit::Packets,
                        Unit::Packets => Unit::Bytes,
                    };
                    self.save();
                }
                KeyCode::Tab => {
                    self.focus = (self.focus + 1) % 3;
                    self.flow_index = 0;
                }
                KeyCode::Char('f') => {
                    self.only_favorites[self.focus] = !self.only_favorites[self.focus];
                    self.flow_index = 0;
                }
                KeyCode::Char('t') => {
                    self.sorting[self.focus] = (self.sorting[self.focus] + 1) % 3;
                    self.flow_index = 0;
                }
                KeyCode::Char('*') if self.screen == Screen::Overview => {
                    if let Some((name, _, _, _)) = self.panel_rows(self.focus).get(self.flow_index)
                    {
                        let key = format!("{}:{name}", self.focus);
                        if !self.config.favorites.remove(&key) {
                            self.config.favorites.insert(key);
                        }
                        self.save();
                    }
                }
                KeyCode::Char('/') => self.edit(Input::Search, self.search.clone()),
                KeyCode::Up => self.flow_index = self.flow_index.saturating_sub(1),
                KeyCode::Down => {
                    let count = if self.screen == Screen::Alerts {
                        self.events.len()
                    } else if self.screen == Screen::Overview {
                        self.panel_rows(self.focus).len()
                    } else {
                        self.flows().len()
                    };
                    self.flow_index = (self.flow_index + 1).min(count.saturating_sub(1))
                }
                KeyCode::Enter if self.screen == Screen::Overview => {
                    if let Some((name, _, _, _)) = self.panel_rows(self.focus).get(self.flow_index)
                    {
                        self.search = format!("={name}");
                        self.flow_index = 0;
                        self.screen = Screen::Inspect;
                    }
                }
                KeyCode::Enter if self.screen == Screen::Inspect => {
                    self.probe_message.clear();
                    self.probe_result = None;
                    self.details_key = self
                        .flows()
                        .get(self.flow_index)
                        .map(|flow| flow.key.clone());
                    self.details = !self.flows().is_empty()
                }
                KeyCode::Char('c') if self.screen == Screen::Alerts => {
                    self.events.clear();
                    self.flow_index = 0;
                }
                _ => {}
            },
            Screen::Firewall => match key.code {
                KeyCode::Up => self.flow_index = self.flow_index.saturating_sub(1),
                KeyCode::Down => {
                    self.flow_index =
                        (self.flow_index + 1).min(self.config.rules.len().saturating_sub(1))
                }
                KeyCode::Delete | KeyCode::Char('d') => {
                    if let Some(rule) = self.config.rules.get(self.flow_index) {
                        self.pending_rule = Some((true, rule.clone()));
                    }
                }
                _ => {}
            },
            Screen::About => {}
        }
    }
}
