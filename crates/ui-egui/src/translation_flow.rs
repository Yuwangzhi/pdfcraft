//! Width-responsive prose with inline math, semantic paragraphs and separate display equations.
use pdfcraft_engine::translation_math::{BlockKind, Part};

pub(crate) enum Run {
    Text { text: String, strong: bool },
    Math { latex: String, display: bool, texture: Result<egui::TextureHandle, String> },
}
pub(crate) struct RenderedBlock {
    kind: BlockKind,
    runs: Vec<Run>,
}

pub(crate) fn typeset(text: &str, ctx: &egui::Context) -> Vec<RenderedBlock> {
    pdfcraft_engine::translation_math::document(text)
        .into_iter()
        .enumerate()
        .map(|(block_index, block)| {
            let mut runs = Vec::new();
            let mut strong = false;
            for (part_index, part) in block.parts.into_iter().enumerate() {
                match part {
                    Part::Text(text) => {
                        for (i, text) in text.split("**").enumerate() {
                            if i > 0 {
                                strong = !strong;
                            }
                            if !text.is_empty() {
                                runs.push(Run::Text { text: text.into(), strong });
                            }
                        }
                    }
                    Part::Math { latex, display, png } => {
                        let texture = png.and_then(|bytes| {
                            let mut image = image::load_from_memory(&bytes).map_err(|_| "公式图像无法读取。".to_string())?.to_rgba8();
                            for pixel in image.pixels_mut() {
                                pixel.0[0] = 255;
                                pixel.0[1] = 255;
                                pixel.0[2] = 255;
                            }
                            let color = egui::ColorImage::from_rgba_unmultiplied([image.width() as usize, image.height() as usize], image.as_raw());
                            Ok(ctx.load_texture(format!("translation-{block_index}-{part_index}"), color, egui::TextureOptions::LINEAR))
                        });
                        runs.push(Run::Math { latex, display, texture });
                    }
                }
            }
            RenderedBlock { kind: block.kind, runs }
        })
        .collect()
}

pub(crate) fn show(ui: &mut egui::Ui, blocks: &[RenderedBlock]) {
    for (index, block) in blocks.iter().enumerate() {
        ui.push_id(index, |ui| {
            let size = if block.kind == BlockKind::Heading { 19.0 } else { 17.0 };
            if let BlockKind::ListItem(marker) = &block.kind {
                ui.horizontal_top(|ui| {
                    ui.label(egui::RichText::new(marker).size(size));
                    ui.vertical(|ui| body(ui, &block.runs, size, false));
                });
            } else {
                body(ui, &block.runs, size, block.kind == BlockKind::Heading);
            }
            ui.add_space(if block.kind == BlockKind::Heading { 6.0 } else { 10.0 });
        });
    }
}

fn body(ui: &mut egui::Ui, runs: &[Run], size: f32, heading: bool) {
    let mut start = 0;
    for (i, run) in runs.iter().enumerate() {
        if let Run::Math { display: true, .. } = run {
            inline(ui, runs.get(start..i).unwrap_or_default(), size, heading);
            ui.add_space(5.0);
            egui::ScrollArea::horizontal().id_salt(("display-formula", i)).show(ui, |ui| {
                ui.horizontal(|ui| {
                    if let Run::Math { texture: Ok(texture), .. } = run {
                        // Center short equations; only wide equations need horizontal scrolling.
                        let width = texture.size_vec2().x * size / 36.0;
                        ui.add_space(((ui.available_width() - width) * 0.5).max(0.0));
                    }
                    math(ui, run, size);
                });
            });
            ui.add_space(5.0);
            start = i + 1;
        }
    }
    inline(ui, runs.get(start..).unwrap_or_default(), size, heading);
}

fn inline(ui: &mut egui::Ui, runs: &[Run], size: f32, heading: bool) {
    if runs.is_empty() {
        return;
    }
    // egui's wrapped Label continues from the preceding widget, including onto subsequent
    // rows. A formula is one indivisible inline item; prose flows around it at any width.
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(1.0, 5.0);
        for run in runs {
            match run {
                Run::Text { text, strong } => {
                    let mut rich = egui::RichText::new(text).size(size);
                    if heading || *strong {
                        rich = rich.strong();
                    }
                    ui.add(egui::Label::new(rich).wrap().selectable(true));
                }
                _ => math(ui, run, size),
            }
        }
    });
}

fn math(ui: &mut egui::Ui, run: &Run, size: f32) {
    if let Run::Math { latex, texture, .. } = run {
        match texture {
            Ok(texture) => {
                let dimensions = texture.size_vec2() * size / 36.0;
                ui.add(egui::Image::new(texture).fit_to_exact_size(dimensions).tint(ui.visuals().text_color())).on_hover_text(latex);
            }
            Err(error) => {
                ui.colored_label(ui.visuals().error_fg_color, error).on_hover_text(latex);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inline_math_flows_with_prose_and_rewraps_in_a_narrow_popup() {
        let ctx = egui::Context::default();
        let blocks = typeset(r"Given \(x_t\), predict \(y_t\) from previous observations and preserve the sentence.", &ctx);
        let mut heights = Vec::new();
        for width in [700.0, 220.0] {
            let mut height = 0.0;
            for _ in 0..2 {
                let _ = ctx.run_ui(
                    egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(width, 800.0))), ..Default::default() },
                    |ui| {
                        egui::CentralPanel::default().show(ui, |ui| {
                            let start = ui.cursor().top();
                            show(ui, &blocks);
                            height = ui.cursor().top() - start;
                        });
                    },
                );
            }
            heights.push(height);
        }
        assert!(heights[0] < 80.0, "inline formula must not become its own row: {heights:?}");
        assert!(heights[1] > heights[0], "narrow popup must reflow: {heights:?}");
    }
}
