//! Restore last view settings when reopening documents (Acrobat's Preferences ▸ Documents
//! option; #1767): the page, zoom and page display (with the two-page cover setting) a file was
//! last shown with, remembered by its path when its tab is closed or settings are saved, and
//! applied when the file is opened again, or offered in an optional continuation prompt. Desktop only: a document without a file has nothing to be found by.

use serde::Serialize;

#[derive(Clone)]
pub(crate) struct ReadingPrompt {
    pub last: LastView,
    pub opened_page: usize,
}

use crate::PdfCraftApp;
use crate::canvas::{self, ViewDefaults};

/// How many files' views are remembered (the most recently closed or saved first).
const MAX_REMEMBERED: usize = 100;

/// How a file was last shown.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LastView {
    /// Canonical when present, otherwise an absolute path.
    pub path: String,
    /// 0-based.
    pub page: usize,
    /// As settings name it: `fit-width`, `fit-page` or a percentage, kept to its fraction
    /// (`62.5%`; read back by `ViewDefaults::with_zoom`).
    pub zoom: String,
    /// The page display token (`canvas::display_token`).
    pub display: String,
    /// Whether two-page view showed a cover page (the first page on its own).
    pub cover: bool,
}

/// Resolve aliases while files exist; retain an absolute path for missing files.
#[cfg(not(target_arch = "wasm32"))]
fn absolute(path: &str) -> String {
    std::fs::canonicalize(path).or_else(|_| std::path::absolute(path)).map_or_else(|_| path.to_string(), |p| p.to_string_lossy().into_owned())
}

#[cfg(target_arch = "wasm32")]
fn absolute(path: &str) -> String {
    path.to_string()
}

impl PdfCraftApp {
    /// How view `index` shows its document now; `None` for a document without a file. In full
    /// screen, the display it will go back to, at the page reached.
    fn view_to_remember(&self, index: usize) -> Option<LastView> {
        let view = self.views.get(index)?;
        let path = self.session.get(view.id)?.path.as_deref()?;
        // Saving idle settings must not replace the unread saved position with the opening page.
        if let Some(prompt) = self.reading_prompts.get(&view.id).filter(|p| p.opened_page == view.current) {
            return Some(prompt.last.clone());
        }
        let shown = view.normal_display();
        // A fixed zoom keeps its fraction (62.5%, not 63%): `zoom_name` rounds to whole percents.
        let zoom = match shown.fit {
            canvas::Fit::None => format!("{}%", shown.zoom * 100.0),
            fit => ViewDefaults { fit, zoom: shown.zoom, ..ViewDefaults::default() }.zoom_name(),
        };
        Some(LastView {
            path: absolute(path),
            page: view.current,
            zoom,
            display: canvas::display_token(shown.layout, shown.continuous).to_string(),
            cover: shown.cover,
        })
    }

    /// Remember how view `index` shows its document (when its tab closes).
    pub(crate) fn remember_view(&mut self, index: usize) {
        if let Some(v) = self.view_to_remember(index) {
            self.last_views.retain(|l| l.path != v.path);
            self.last_views.insert(0, v);
            self.last_views.truncate(MAX_REMEMBERED);
        }
        if let Some(view) = self.views.get(index) {
            self.reading_prompts.remove(&view.id);
        }
    }

    /// The remembered views as settings store them: the open views first, as they are now
    /// (so quitting remembers them without closing their tabs), then the rest.
    pub(crate) fn last_views_to_save(&self) -> Vec<LastView> {
        let mut out: Vec<LastView> = (0..self.views.len()).filter_map(|i| self.view_to_remember(i)).collect();
        for l in &self.last_views {
            if !out.iter().any(|o| o.path == l.path) {
                out.push(l.clone());
            }
        }
        out.truncate(MAX_REMEMBERED);
        out
    }

    /// Settings are untrusted: an entry without a usable path, zoom or display is dropped, and
    /// the list is capped.
    pub(crate) fn restore_last_views(&mut self, v: &serde_json::Value) {
        let Some(list) = v.as_array() else { return };
        self.last_views = list
            .iter()
            .take(MAX_REMEMBERED)
            .filter_map(|e| {
                let path = absolute(e["path"].as_str().filter(|p| !p.is_empty())?);
                let zoom = e["zoom"].as_str().filter(|z| ViewDefaults::default().with_zoom(z).is_some())?.to_string();
                let display = e["display"].as_str().filter(|d| canvas::parse_display_token(d).is_some())?.to_string();
                let page = usize::try_from(e["page"].as_u64().unwrap_or(0)).unwrap_or(0);
                let cover = e["cover"].as_bool().unwrap_or(false);
                Some(LastView { path, page, zoom, display, cover })
            })
            .collect();
    }

    /// Restore or offer the last view of document `index`, when the preference is on and
    /// the file is remembered. Applied after the document's own initial view, which it
    /// overrides: where the reader left off is what they asked to come back to.
    pub(crate) fn apply_last_view(&mut self, index: usize) {
        if !self.restore_last_view {
            return;
        }
        let Some(view) = self.views.get(index) else { return };
        let Some(doc) = self.session.get(view.id) else { return };
        let Some(path) = doc.path.as_deref().map(absolute) else { return };
        let pages = doc.info.pages.len();
        let Some(last) = self.last_views.iter().find(|l| l.path == path).cloned() else { return };
        if pages == 0 {
            return;
        }
        let mut last = last;
        last.page = last.page.min(pages.saturating_sub(1));
        if self.prompt_last_view && last.page != view.current {
            self.reading_prompts.insert(view.id, ReadingPrompt { last, opened_page: view.current });
            return;
        }
        self.apply_remembered_view(index, &last);
    }

    fn apply_remembered_view(&mut self, index: usize, last: &LastView) {
        let Some(view) = self.views.get_mut(index) else { return };
        if let Some((layout, continuous)) = canvas::parse_display_token(&last.display) {
            view.set_display(layout, continuous);
        }
        view.set_cover(last.cover);
        if let Some(defaults) = ViewDefaults::default().with_zoom(&last.zoom) {
            match defaults.fit {
                canvas::Fit::None => view.set_zoom(defaults.zoom),
                fit => view.set_fit(fit),
            }
        }
        view.go_to_reading_page(last.page);
    }

    /// Explicit continuation, shared by the prompt, command palette and UI control channel.
    pub fn resume_reading(&mut self) -> bool {
        let Some((index, id)) = self.active_ids() else { return false };
        let Some(prompt) = self.reading_prompts.remove(&id) else { return false };
        self.apply_remembered_view(index, &prompt.last);
        true
    }

    pub fn dismiss_reading_prompt(&mut self) -> bool {
        self.active_ids().is_some_and(|(_, id)| self.reading_prompts.remove(&id).is_some())
    }

    pub(crate) fn show_reading_prompt(&mut self, ui: &mut egui::Ui) {
        let Some((_, id)) = self.active_ids() else { return };
        let Some(prompt) = self.reading_prompts.get(&id) else { return };
        let page = prompt.last.page.saturating_add(1);
        let chinese = crate::i18n::current().code().starts_with("zh");
        let message = if chinese { format!("上次读到第 {page} 页") } else { format!("You last read page {page}") };
        let mut resume = false;
        let mut dismiss = false;
        let t = crate::theme::Tokens::get(ui.ctx());
        egui::Panel::top("reading-position-prompt").frame(egui::Frame::NONE.fill(t.panel).inner_margin(egui::Margin::symmetric(14, 8))).show(
            ui,
            |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(message);
                    resume = ui.button(if chinese { "继续阅读" } else { "Continue reading" }).clicked();
                    dismiss = ui.button(if chinese { "从当前页阅读" } else { "Read from here" }).clicked();
                });
            },
        );
        if resume {
            self.resume_reading();
        }
        if dismiss {
            self.dismiss_reading_prompt();
        }
    }
}
