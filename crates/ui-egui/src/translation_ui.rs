//! A document-local translation window; network work never blocks the UI thread.

enum RenderedPart {
    Text(String),
    Math { latex: String, texture: Result<egui::TextureHandle, String> },
}
type Reply = Result<(String, Vec<RenderedPart>), String>;

fn typeset(text: &str, ctx: &egui::Context) -> Vec<RenderedPart> {
    pdfcraft_engine::translation_math::render(text)
        .into_iter()
        .enumerate()
        .map(|(i, part)| match part {
            pdfcraft_engine::translation_math::Part::Text(text) => RenderedPart::Text(text),
            pdfcraft_engine::translation_math::Part::Math { latex, png, .. } => {
                let texture = png.and_then(|bytes| {
                    let mut image = image::load_from_memory(&bytes).map_err(|_| "公式图像无法读取。".to_string())?.to_rgba8();
                    for pixel in image.pixels_mut() {
                        pixel.0[0] = 255;
                        pixel.0[1] = 255;
                        pixel.0[2] = 255;
                    }
                    let color = egui::ColorImage::from_rgba_unmultiplied([image.width() as usize, image.height() as usize], image.as_raw());
                    Ok(ctx.load_texture(format!("translation-math-{i}"), color, egui::TextureOptions::LINEAR))
                });
                RenderedPart::Math { latex, texture }
            }
        })
        .collect()
}

#[derive(Default)]
pub(crate) struct Translation {
    open: bool,
    trigger: SelectionTrigger<(usize, usize, usize)>,
    image: Option<pdfcraft_engine::translation::SelectionImage>,
    source: String,
    result: Option<Result<String, String>>,
    parts: Vec<RenderedPart>,
    queued: Option<(String, Option<pdfcraft_engine::translation::SelectionImage>)>,
    pending: Option<std::sync::mpsc::Receiver<Reply>>,
}

impl Translation {
    pub(crate) fn start(&mut self, ctx: &egui::Context, text: String, image: Option<pdfcraft_engine::translation::SelectionImage>) {
        self.open = true;
        if self.pending.is_some() {
            self.queued = Some((text, image));
            return;
        }
        self.source = text;
        self.image = image.clone();
        self.result = None;
        self.parts.clear();
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

    pub(crate) fn show(&mut self, ctx: &egui::Context, doc: u64, selection: Option<(usize, usize, usize)>, anchor: Option<egui::Pos2>) -> bool {
        if let Some(rx) = &self.pending {
            match rx.try_recv() {
                Ok(result) => {
                    self.result = Some(result.map(|(text, parts)| {
                        self.parts = parts;
                        text
                    }));
                    self.pending = None;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.result = Some(Err("翻译任务中断，请重试。".into()));
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
        let should_start = self.trigger.observe(selection, now, pointer_down, self.pending.is_some()).is_some();
        if self.trigger.due {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.open = false;
            self.queued = None;
        }
        if !self.open {
            return should_start;
        }
        let mut open = self.open;
        let mut retry = false;
        let window = egui::Window::new(if self.source.is_empty() { "截图翻译" } else { "划词翻译" })
            .id(egui::Id::new(("translation-window", doc)))
            .open(&mut open)
            .default_width(460.0)
            .default_pos(anchor.map(|p| p + egui::vec2(18.0, 22.0)).unwrap_or(egui::pos2(120.0, 120.0)))
            .resizable(true)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.weak("DeepSeek Flash · 学术翻译 → 简体中文");
                ui.separator();
                egui::ScrollArea::vertical().id_salt("translation-content").max_height(420.0).show(ui, |ui| {
                    ui.strong("原文");
                    ui.add(egui::Label::new(if self.source.is_empty() { "已框选的页面截图" } else { &self.source }).wrap().selectable(true));
                    ui.add_space(12.0);
                    ui.strong("译文");
                    match &self.result {
                        Some(Ok(_)) => {
                            for (index, part) in self.parts.iter().enumerate() {
                                match part {
                                    RenderedPart::Text(text) if !text.trim().is_empty() => {
                                        ui.add(egui::Label::new(egui::RichText::new(text.trim()).size(17.0)).wrap().selectable(true));
                                    }
                                    RenderedPart::Math { latex, texture: Ok(texture) } => {
                                        egui::ScrollArea::horizontal().id_salt(("formula", index)).show(ui, |ui| {
                                            ui.add(
                                                egui::Image::new(texture)
                                                    .fit_to_exact_size(texture.size_vec2() / 2.0)
                                                    .tint(ui.visuals().text_color()),
                                            )
                                            .on_hover_text(latex);
                                        });
                                        ui.add_space(4.0);
                                    }
                                    RenderedPart::Math { latex, texture: Err(error) } => {
                                        ui.colored_label(ui.visuals().error_fg_color, error);
                                        ui.collapsing("查看公式原文", |ui| {
                                            ui.label(latex);
                                        });
                                    }
                                    _ => {}
                                }
                            }
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
                    ui.weak(if self.source.is_empty() {
                        "仅发送框选截图"
                    } else if self.image.is_some() {
                        "文字 + 选区截图核对公式"
                    } else {
                        "仅发送所选文字"
                    });
                });
            });
        if let Some(window) = window
            && ctx.input(|i| i.pointer.any_pressed() && i.pointer.interact_pos().is_some_and(|p| !window.response.rect.contains(p)))
        {
            open = false;
            self.trigger.due = false;
        }
        if !open {
            self.queued = None;
        }
        self.open = open;
        if retry {
            self.start(ctx, self.source.clone(), self.image.clone());
        }
        should_start
    }
}

/// Wait for a completed selection; translate each settled selection once. Queue the latest
/// selection while a request runs instead of launching unbounded concurrent network work.
#[derive(Default)]
struct SelectionTrigger<K> {
    observed: Option<K>,
    deadline: f64,
    due: bool,
}

impl<K: PartialEq + Clone> SelectionTrigger<K> {
    fn observe(&mut self, text: Option<K>, now: f64, pointer_down: bool, busy: bool) -> Option<K> {
        if text != self.observed {
            self.observed = text;
            self.deadline = now + 0.3;
            self.due = self.observed.is_some();
        }
        if pointer_down {
            self.deadline = now + 0.3;
        }
        if self.due && !pointer_down && !busy && now >= self.deadline {
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
    fn translation_popups_close_on_outside_press_and_stay_closed() {
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
                        translation.show(ui.ctx(), 1, None, None);
                    },
                );
            }
            assert!(translation.open);
            let _ = ctx.run_ui(
                egui::RawInput {
                    events: vec![egui::Event::PointerButton {
                        pos: egui::pos2(1100.0, 800.0),
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: egui::Modifiers::NONE,
                    }],
                    ..Default::default()
                },
                |ui| {
                    translation.show(ui.ctx(), 1, None, None);
                },
            );
            assert!(!translation.open, "{source}");
            let _ = ctx.run_ui(egui::RawInput::default(), |ui| {
                translation.show(ui.ctx(), 1, None, None);
            });
            assert!(!translation.open);
        }
    }

    #[test]
    fn automatic_translation_waits_for_release_and_queues_latest_selection() {
        let mut t = SelectionTrigger::<String>::default();
        assert!(t.observe(Some("first".into()), 0.0, true, false).is_none());
        assert!(t.observe(Some("first sentence".into()), 0.5, true, false).is_none());
        assert!(t.observe(Some("first sentence".into()), 0.6, false, false).is_none());
        assert_eq!(t.observe(Some("first sentence".into()), 0.9, false, false).as_deref(), Some("first sentence"));
        assert!(t.observe(Some("first sentence".into()), 1.2, false, false).is_none());
        assert!(t.observe(Some("second".into()), 1.5, false, true).is_none());
        assert!(t.observe(Some("latest".into()), 2.0, false, true).is_none());
        assert!(t.observe(Some("latest".into()), 2.5, false, true).is_none());
        assert_eq!(t.observe(Some("latest".into()), 3.0, false, false).as_deref(), Some("latest"));
        assert!(t.observe(None, 4.0, false, false).is_none());
        assert!(t.observe(Some("latest".into()), 4.5, false, false).is_none());
        assert_eq!(t.observe(Some("latest".into()), 5.0, false, false).as_deref(), Some("latest"));
    }
}
