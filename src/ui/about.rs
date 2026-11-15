use super::*;

pub(super) fn about(frame: &mut Frame, app: &App, area: Rect, p: Palette) {
    let text = format!(
        "LiminalNet {}\n\nDéveloppé par RZU Informatique\nConçu, testé et débogué par RZU Informatique avec l’assistance d’OpenAI Codex (not vibe coded).\nHighly inspired by Sniffnet, created by Giuliano Bellini and contributors.\n\nGit: github.com/RZU-IT/LiminalNet\n\nDevelopment build: see docs/validation.md for actual verification status.\nIP Geolocation by DB-IP · CC BY 4.0\nServices: IANA port registry\nNo Sniffnet source code is bundled.\n\nEsc: {}",
        env!("CARGO_PKG_VERSION"),
        app.tr("back")
    );
    frame.render_widget(
        Paragraph::new(text)
            .wrap(Wrap { trim: true })
            .block(block("About LiminalNet", p, true)),
        area,
    );
}
