//! Offline math typesetting for translated prose; no TeX interpreter, files or subprocesses.

pub enum Part {
    Text(String),
    Math { latex: String, display: bool, png: Result<Vec<u8>, String> },
}

/// Keep unsupported expressions visible as an explicit error, never silently drop symbols.
pub fn render(text: &str) -> Vec<Part> {
    let font = latex_rust::MathFont::stix_two_math().map_err(|_| "无法加载公式字体。".to_string());
    split(text)
        .into_iter()
        .map(|(source, math)| match math {
            None => Part::Text(source),
            Some(display) => {
                let png = font.as_ref().map_err(Clone::clone).and_then(|font| render_formula(&source, display, font));
                Part::Math { latex: source, display, png }
            }
        })
        .collect()
}

fn render_formula(source: &str, display: bool, font: &latex_rust::MathFont) -> Result<Vec<u8>, String> {
    use latex_rust::{Dim, MathStyle, PngOptions};
    if source.len() > 8192 {
        return Err("公式过长，请缩小选区。".into());
    }
    crate::guard(|| {
        let ast = latex_rust::parse(source).map_err(|_| "公式语法暂不支持，展开可查看原始表达。".to_string())?;
        let style = if display { MathStyle::Display } else { MathStyle::Text };
        let tree = latex_rust::layout(&ast, font, style).map_err(|_| "公式排版失败，展开可查看原始表达。".to_string())?;
        let w = tree.width.to_dec_string().parse::<f64>().unwrap_or(f64::INFINITY) * 36.0;
        let h = (&tree.height + &tree.depth).to_dec_string().parse::<f64>().unwrap_or(f64::INFINITY) * 36.0;
        if !w.is_finite() || !h.is_finite() || w > 6000.0 || h > 2000.0 || w * h > 2_000_000.0 {
            return Err("公式尺寸过大，请缩小选区。".into());
        }
        latex_rust::render_png(&tree, font, &PngOptions { font_size_pt: Dim::from_i64(18), display, ..Default::default() })
            .map_err(|_| "公式渲染失败，展开可查看原始表达。".to_string())
    })
    .map_err(|_| "公式渲染失败，展开可查看原始表达。".to_string())?
}

/// Supports the requested delimiters and the common $ / $$ alternatives. Text is kept in
/// order; an unclosed delimiter stays visible as text. Bound formula count from API input.
fn split(text: &str) -> Vec<(String, Option<bool>)> {
    let mut out = Vec::new();
    let mut remaining = text;
    while !remaining.is_empty() && out.len() < 128 {
        let next = remaining.char_indices().find_map(|(i, c)| {
            let tail = remaining.get(i..)?;
            if c == '\\' {
                if tail.starts_with("\\(") {
                    Some((i, "\\(", "\\)", false))
                } else if tail.starts_with("\\[") {
                    Some((i, "\\[", "\\]", true))
                } else {
                    None
                }
            } else if c == '$' && !remaining.get(..i).is_some_and(|s| s.ends_with('\\')) {
                if tail.starts_with("$$") { Some((i, "$$", "$$", true)) } else { Some((i, "$", "$", false)) }
            } else {
                None
            }
        });
        let Some((start, open, close, display)) = next else { break };
        let Some(after) = remaining.get(start + open.len()..) else { break };
        let Some(end) = after.find(close) else { break };
        if start > 0
            && let Some(prefix) = remaining.get(..start)
        {
            out.push((prefix.to_string(), None));
        }
        if let Some(math) = after.get(..end) {
            out.push((math.to_string(), Some(display)));
        }
        remaining = after.get(end + close.len()..).unwrap_or("");
    }
    if !remaining.is_empty() {
        out.push((remaining.to_string(), None));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn translation_math_preserves_prose_and_typesets_formulas() {
        let parts = render(r"递推关系为 \[x_{t+1}=\alpha x_t+\frac{1}{n}\sum_{i=1}^{n}\varepsilon_i.\qquad(1)\] 其中 \(n>0\)。");
        assert_eq!(parts.len(), 5);
        for part in &parts {
            if let Part::Math { png, .. } = part {
                let image = image::load_from_memory(png.as_ref().unwrap()).unwrap();
                assert!(image.width() > 10 && image.height() > 10);
            }
        }
        assert_eq!(split(r"中文 $x^2$ 与 $$\frac{1}{2}$$").len(), 4);
        assert_eq!(split(r"未完成 \(x")[0].0, r"未完成 \(x");
        assert!(matches!(&render(r"\[\input{/etc/passwd}\]")[0], Part::Math { png: Err(_), .. }));
    }
}
