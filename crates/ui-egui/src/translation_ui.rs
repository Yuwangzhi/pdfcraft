//! A document-local translation sidebar; network work never blocks the UI thread.

use crate::translation_flow::{RenderedBlock, typeset};
type Reply = Result<(String, Vec<RenderedBlock>), String>;

#[derive(Default)]
pub(crate) struct Translation {
    open: bool,
    trigger: SelectionTrigger<crate::canvas::TranslationKey>,
    image: Option<pdfcraft_engine::translation::SelectionImage>,
    source: String,
    source_expanded: bool,
    result: Option<Result<String, String>>,
    parts: Vec<RenderedBlock>,
    queued: Option<(String, Option<pdfcraft_engine::translation::SelectionImage>)>,
    pending: Option<std::sync::mpsc::Receiver<Reply>>,
}

impl Translation {
    pub(crate) fn start(&mut self, ctx: &egui::Context, text: String, image: Option<pdfcraft_engine::translation::SelectionImage>) {
        self.open = true;
        ctx.request_repaint();
        self.source = text.clone();
        self.source_expanded = false;
        self.image = image.clone();
        self.result = None;
        self.parts.clear();
        if self.pending.is_some() {
            self.queued = Some((text, image));
            return;
        }
        let text = match if self.source.trim().is_empty() && image.is_some() {
            Ok(String::new())
        } else {
            pdfcraft_engine::translation::prepare(&self.source)
        } {
            Ok(text) => text,
            Err(error) => {
                self.result = Some(Err(error));
                return;
            }
        };
        let (tx, rx) = std::sync::mpsc::channel();
        self.pending = Some(rx);
        let ctx = ctx.clone();
        if std::thread::Builder::new()
            .name("selection-translation".into())
            .spawn(move || {
                let result = pdfcraft_engine::translation::to_chinese_with_image(&text, image.as_ref()).map(|text| {
                    let parts = typeset(&text, &ctx);
                    (text, parts)
                });
                let _ = tx.send(result);
                ctx.request_repaint();
            })
            .is_err()
        {
            self.pending = None;
            self.result = Some(Err("无法启动翻译，请重试。".into()));
        }
    }

    pub(crate) fn update(&mut self, ctx: &egui::Context, selection: Option<crate::canvas::TranslationKey>) -> bool {
        if let Some(rx) = &self.pending {
            match rx.try_recv() {
                Ok(result) => {
                    // A newer selection owns the sidebar immediately. Never flash an
                    // obsolete response while its replacement waits for the serial worker.
                    if self.queued.is_none() {
                        self.result = Some(result.map(|(text, parts)| {
                            self.parts = parts;
                            text
                        }));
                    }
                    self.pending = None;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    if self.queued.is_none() {
                        self.result = Some(Err("翻译任务中断，请重试。".into()));
                    }
                    self.pending = None;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => ctx.request_repaint_after(std::time::Duration::from_millis(100)),
            }
        }
        if self.pending.is_none()
            && let Some((text, image)) = self.queued.take()
        {
            self.start(ctx, text, image);
        }
        let (now, pointer_down) = ctx.input(|i| (i.time, i.pointer.any_down()));
        let should_start = self.trigger.observe(selection, now, pointer_down).is_some();
        if self.trigger.due {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.open = false;
            self.queued = None;
            self.trigger.due = false;
            ctx.request_repaint();
        }
        should_start
    }

    pub(crate) fn is_open(&self) -> bool {
        self.open
    }

    pub(crate) fn show(&mut self, ui: &mut egui::Ui, doc: u64) {
        if !self.open {
            return;
        }
        let ctx = ui.ctx().clone();
        let mut open = self.open;
        let mut retry = false;
        let t = crate::theme::Tokens::get(&ctx);
        egui::Panel::right(egui::Id::new(("translation-panel", doc)))
            .default_size(420.0)
            .size_range(300.0..=620.0)
            .resizable(true)
            .frame(
                egui::Frame::NONE
                    .fill(t.panel)
                    .inner_margin(egui::Margin { left: 16, right: 16, top: 12, bottom: 12 })
                    .stroke(egui::Stroke::new(1.0, t.divider)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(if self.source.is_empty() { "截图翻译" } else { "划词翻译" });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if crate::icons::button(ui, "x", 26.0, false, "收起翻译栏").clicked() {
                            open = false;
                        }
                    });
                });
                ui.weak("DeepSeek Flash · 学术翻译 → 简体中文");
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_salt("translation-content")
                    .max_height((ui.available_height() - 76.0).max(80.0))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if !self.source.is_empty() {
                            let source = egui::CollapsingHeader::new("查看原文").open(Some(self.source_expanded)).show(ui, |ui| {
                                ui.add(egui::Label::new(&self.source).wrap().selectable(true));
                            });
                            if source.header_response.clicked() {
                                self.source_expanded = !self.source_expanded;
                            }
                            ui.add_space(8.0);
                        }
                        match &self.result {
                            Some(Ok(_)) => {
                                crate::translation_flow::show(ui, &self.parts);
                            }
                            Some(Err(error)) => {
                                ui.colored_label(ui.visuals().error_fg_color, error);
                            }
                            None => {
                                ui.horizontal(|ui| {
                                    ui.spinner();
                                    ui.label("正在翻译…");
                                });
                            }
                        }
                    });
                ui.separator();
                ui.horizontal(|ui| {
                    if let Some(Ok(text)) = &self.result
                        && ui.button("复制译文").clicked()
                    {
                        ctx.copy_text(text.clone());
                    }
                    if ui.add_enabled(self.pending.is_none(), egui::Button::new("重新翻译")).clicked() {
                        retry = true;
                    }
                });
                ui.weak(if self.source.is_empty() {
                    "仅发送框选截图"
                } else if self.image.is_some() {
                    "文字 + 选区截图核对公式"
                } else {
                    "仅发送所选文字"
                });
            });
        if !open {
            self.queued = None;
            self.trigger.due = false;
        }
        self.open = open;
        if retry {
            self.start(&ctx, self.source.clone(), self.image.clone());
        }
        if !self.open {
            ctx.request_repaint();
        }
    }
}

/// Wait for a completed selection; translate each settled selection once. Queue the latest
/// selection while a request runs instead of launching unbounded concurrent network work.
struct SelectionTrigger<K> {
    observed: Option<K>,
    deadline: f64,
    due: bool,
}

impl<K> Default for SelectionTrigger<K> {
    fn default() -> Self {
        Self { observed: None, deadline: 0.0, due: false }
    }
}

impl<K: PartialEq + Clone> SelectionTrigger<K> {
    fn observe(&mut self, text: Option<K>, now: f64, pointer_down: bool) -> Option<K> {
        if text != self.observed {
            self.observed = text;
            self.deadline = now + 0.3;
            self.due = self.observed.is_some();
        }
        if pointer_down {
            self.deadline = now + 0.3;
        }
        if self.due && !pointer_down && now >= self.deadline {
            self.due = false;
            return self.observed.clone();
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translation_temporarily_hides_docked_inspector_and_restores_layout() {
        use egui_kittest::kittest::Queryable;
        let pdf = pdfcraft_testkit::Pdf::new()
            .obj("<< /Type /Catalog /Pages 2 0 R >>")
            .obj("<< /Type /Pages /Kids [3 0 R] /Count 1 >>")
            .obj("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 600 800] >>")
            .build();
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1400.0, 900.0)).build_eframe(move |_| {
            let mut app = crate::PdfCraftApp::new();
            app.set_option("language", "en").unwrap();
            app.open_bytes("synthetic.pdf", None, pdf.clone()).unwrap();
            app.choose_right_panel(Some(crate::RightPanel::Bookmarks));
            app
        });
        h.run_steps(5);
        h.get_by_label("This document has no bookmarks.");
        let layout = serde_json::to_value(&h.state().docking).unwrap();
        h.state_mut().views[0].translation =
            Translation { open: true, source: "Synthetic selection".into(), result: Some(Ok("测试译文".into())), ..Default::default() };
        h.run_steps(5);
        assert!(h.query_by_label("This document has no bookmarks.").is_none());
        assert_eq!(h.state().right, Some(crate::RightPanel::Bookmarks));
        assert_eq!(serde_json::to_value(&h.state().docking).unwrap(), layout);
        h.state_mut().views[0].translation.open = false;
        h.run_steps(5);
        h.get_by_label("This document has no bookmarks.");
        assert_eq!(serde_json::to_value(&h.state().docking).unwrap(), layout);
    }

    #[test]
    fn original_starts_collapsed_opens_manually_and_resets_for_next_translation() {
        use egui_kittest::kittest::Queryable;
        let state =
            Translation {
                open: true, source: "Source stays hidden until requested.".into(), result: Some(Ok("译文".into())), ..Default::default()
            };
        let mut h = egui_kittest::Harness::builder().with_size(egui::vec2(1000.0, 900.0)).build_ui_state(
            |ui, state: &mut Translation| {
                state.update(ui.ctx(), None);
                state.show(ui, 1);
            },
            state,
        );
        h.run_steps(4);
        assert!(h.query_by_label("Source stays hidden until requested.").is_none());
        h.get_by_label("查看原文").click();
        h.run_steps(8);
        assert!(h.state().source_expanded);
        h.get_by_label("Source stays hidden until requested.");
        // Invalid input exercises request initialization without making any network request.
        h.state_mut().start(&egui::Context::default(), " ".into(), None);
        assert!(!h.state().source_expanded);
    }

    #[test]
    fn translation_sidebar_docks_right_and_stays_open_on_outside_click() {
        for source in ["selected text", ""] {
            let ctx = egui::Context::default();
            let mut translation = Translation { open: true, source: source.into(), result: Some(Ok("译文".into())), ..Default::default() };
            for _ in 0..3 {
                let _ = ctx.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1200.0, 900.0))),
                        ..Default::default()
                    },
                    |ui| {
                        translation.update(ui.ctx(), None);
                        translation.show(ui, 1);
                        let remaining = ui.available_rect_before_wrap();
                        assert!(remaining.right() < 900.0, "sidebar reserves space beside the page");
                        assert!(remaining.left() < 10.0, "sidebar docks to the right");
                    },
                );
            }
            assert!(translation.open);
            let _ = ctx.run_ui(
                egui::RawInput {
                    events: vec![egui::Event::PointerButton {
                        pos: egui::pos2(100.0, 800.0),
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: egui::Modifiers::NONE,
                    }],
                    ..Default::default()
                },
                |ui| {
                    translation.update(ui.ctx(), None);
                    translation.show(ui, 1);
                },
            );
            assert!(translation.open, "a press must not shift the page during selection");
            let _ = ctx.run_ui(
                egui::RawInput {
                    events: vec![egui::Event::PointerButton {
                        pos: egui::pos2(100.0, 800.0),
                        button: egui::PointerButton::Primary,
                        pressed: false,
                        modifiers: egui::Modifiers::NONE,
                    }],
                    ..Default::default()
                },
                |ui| {
                    translation.show(ui, 1);
                },
            );
            assert!(translation.open, "outside clicks keep the docked translation visible: {source}");
            let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
                translation.update(ui.ctx(), None);
                translation.show(ui, 1);
            });
            assert!(translation.open);
            let _ = ctx.run_ui(
                egui::RawInput {
                    events: vec![egui::Event::Key {
                        key: egui::Key::Escape,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: egui::Modifiers::NONE,
                    }],
                    ..Default::default()
                },
                |ui| {
                    translation.update(ui.ctx(), None);
                    translation.show(ui, 1);
                },
            );
            assert!(!translation.open);
        }
    }

    #[test]
    fn a_new_translation_replaces_content_and_discards_the_obsolete_reply() {
        let ctx = egui::Context::default();
        let (tx, rx) = std::sync::mpsc::channel();
        let mut translation = Translation {
            open: true,
            source: "previous".into(),
            source_expanded: true,
            result: Some(Ok("old translation".into())),
            parts: typeset("old translation", &ctx),
            pending: Some(rx),
            ..Default::default()
        };
        translation.start(&ctx, "new selection".into(), None);
        assert_eq!(translation.source, "new selection");
        assert!(translation.result.is_none() && translation.parts.is_empty());
        assert!(!translation.source_expanded);
        // The latest request wins; whitespace exercises initialization without network IO.
        translation.start(&ctx, " ".into(), None);
        assert_eq!(translation.queued.as_ref().unwrap().0, " ");
        tx.send(Ok(("obsolete response".into(), typeset("obsolete response", &ctx)))).unwrap();
        translation.update(&ctx, None);
        assert!(matches!(translation.result, Some(Err(_))));
        assert!(translation.parts.is_empty());
        assert!(translation.pending.is_none() && translation.queued.is_none());
        assert_eq!(translation.source, " ");
    }

    #[test]
    fn automatic_translation_waits_for_release_and_queues_latest_selection() {
        let mut t = SelectionTrigger::<String>::default();
        assert!(t.observe(Some("first".into()), 0.0, true).is_none());
        assert!(t.observe(Some("first sentence".into()), 0.5, true).is_none());
        assert!(t.observe(Some("first sentence".into()), 0.6, false).is_none());
        assert_eq!(t.observe(Some("first sentence".into()), 0.9, false).as_deref(), Some("first sentence"));
        assert!(t.observe(Some("first sentence".into()), 1.2, false).is_none());
        assert!(t.observe(Some("second".into()), 1.5, false).is_none());
        assert!(t.observe(Some("latest".into()), 2.0, false).is_none());
        assert_eq!(t.observe(Some("latest".into()), 2.5, false).as_deref(), Some("latest"));
        assert!(t.observe(Some("latest".into()), 3.0, false).is_none());
        assert!(t.observe(None, 4.0, false).is_none());
        assert!(t.observe(Some("latest".into()), 4.5, false).is_none());
        assert_eq!(t.observe(Some("latest".into()), 5.0, false).as_deref(), Some("latest"));
    }
}
