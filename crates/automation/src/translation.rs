//! Fork translation tools, kept separate from the general dispatcher.
use super::dispatch::Content;
use super::error::{Result, ToolError, failed};
use super::json::Args;
use super::{Automation, Value, json};

impl Automation {
    pub(crate) fn render_translation_math(&mut self, a: &Args) -> Result<Vec<Content>> {
        let text = a.str("text")?;
        if text.len() > 32_768 {
            return Err(Args::wrong("text", "at most 32768 bytes"));
        }
        let mut output = Vec::new();
        for (index, block) in pdfcraft_engine::translation_math::document(text).into_iter().enumerate() {
            let kind = match block.kind {
                pdfcraft_engine::translation_math::BlockKind::Paragraph => "paragraph",
                pdfcraft_engine::translation_math::BlockKind::Heading => "heading",
                pdfcraft_engine::translation_math::BlockKind::ListItem(_) => "list_item",
            };
            for part in block.parts {
                match part {
                    pdfcraft_engine::translation_math::Part::Text(text) => {
                        output.push(Content::Json(json!({"text": text, "block":index, "kind":kind})))
                    }
                    pdfcraft_engine::translation_math::Part::Math { latex, display, png } => {
                        let data = png.map_err(failed)?;
                        let reader = png::Decoder::new(std::io::Cursor::new(&data)).read_info().map_err(failed)?;
                        let (width, height) = (reader.info().width, reader.info().height);
                        output.push(Content::Json(json!({"latex":latex,"display":display,"block":index,"kind":kind})));
                        output.push(Content::Png { data, width, height });
                    }
                }
            }
        }
        Ok(output)
    }
    pub(crate) fn translate_selection(&mut self, name: &str, a: &Args) -> Result<Value> {
        if !a.opt_bool("allow_network")?.unwrap_or(false) {
            return Err(ToolError::InvalidArgs("Set allow_network=true to send the supplied text and optional selection crop to DeepSeek.".into()));
        }
        let image = if let Some(rect) = a.get("rect") {
            let rect: [f32; 4] = serde_json::from_value(rect.clone()).map_err(|_| Args::wrong("rect", "[x0,y0,x1,y1] in page points"))?;
            Some(pdfcraft_engine::translation::SelectionImage { source: self.doc(a)?.export_source(), page: self.page(a)?, rect })
        } else {
            None
        };
        let translated = if name == "region_translate" {
            pdfcraft_engine::translation::image_to_chinese(image.as_ref().ok_or_else(|| Args::wrong("rect", "a screenshot rectangle"))?)
        } else {
            pdfcraft_engine::translation::to_chinese_with_image(a.str("text")?, image.as_ref())
        }
        .map_err(failed)?;
        Ok(
            json!({ "translation": translated, "target": "zh-CN", "provider": "DeepSeek", "model": pdfcraft_engine::translation::MODEL, "selection_image": image.is_some() }),
        )
    }
}
