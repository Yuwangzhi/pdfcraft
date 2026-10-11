//! Offline math typesetting for translated prose; no TeX interpreter, files or subprocesses.

pub enum Part {
    Text(String),
    Math { latex: String, display: bool, png: Result<Vec<u8>, String> },
}

/// Semantic blocks are independent of the popup width. PDF soft line breaks are reflowed;
/// paragraph boundaries, headings, lists and display/inline math keep their meaning.
#[derive(Debug, PartialEq, Eq)]
pub enum BlockKind {
    Paragraph,
    Heading,
    ListItem(String),
}
pub struct Block {
    pub kind: BlockKind,
    pub parts: Vec<Part>,
}

pub fn document(text: &str) -> Vec<Block> {
    let mut lines: Vec<Vec<Part>> = vec![Vec::new()];
    for part in render(text) {
        match part {
            Part::Text(text) => {
                for (i, line) in text.split('\n').enumerate() {
                    if i > 0 {
                        lines.push(Vec::new());
                    }
                    if !line.is_empty()
                        && let Some(last) = lines.last_mut()
                    {
                        last.push(Part::Text(line.to_string()));
                    }
                }
            }
            math => {
                if let Some(last) = lines.last_mut() {
                    last.push(math);
                }
            }
        }
    }
    let mut blocks: Vec<Block> = Vec::new();
    let mut current: Option<Block> = None;
    for mut line in lines {
        let blank = line.iter().all(|p| matches!(p, Part::Text(t) if t.trim().is_empty()));
        if blank {
            if let Some(block) = current.take() {
                blocks.push(block);
            }
            continue;
        }
        let kind = match line.first_mut() {
            Some(Part::Text(text)) => classify_line(text),
            _ => BlockKind::Paragraph,
        };
        let display = line.iter().any(|p| matches!(p, Part::Math { display: true, .. }));
        if (kind != BlockKind::Paragraph || display)
            && let Some(block) = current.take()
        {
            blocks.push(block);
        }
        if current.is_none() {
            current = Some(Block { kind, parts: Vec::new() });
        }
        if let Some(block) = current.as_mut() {
            // A soft line break joins prose, rather than imposing the PDF's column width.
            if let Some(Part::Text(first)) = line.first_mut() {
                *first = first.trim_start().to_string();
            }
            if let Some(Part::Text(last)) = block.parts.last_mut() {
                *last = last.trim_end().to_string();
                let next = line.first().and_then(|p| if let Part::Text(t) = p { t.chars().next() } else { None });
                if last.chars().last().is_some_and(|c| c.is_ascii_alphanumeric()) && next.is_some_and(|c| c.is_ascii_alphanumeric()) {
                    last.push(' ');
                }
            }
            block.parts.append(&mut line);
            if (block.kind == BlockKind::Heading || display)
                && let Some(block) = current.take()
            {
                blocks.push(block);
            }
        }
    }
    if let Some(block) = current {
        blocks.push(block);
    }
    blocks
}

fn classify_line(text: &mut String) -> BlockKind {
    let trimmed = text.trim_start();
    let hashes = trimmed.chars().take_while(|c| *c == '#').count();
    if (1..=6).contains(&hashes) && trimmed.get(hashes..).is_some_and(|s| s.starts_with(' ')) {
        *text = trimmed.get(hashes..).unwrap_or("").trim().trim_matches('*').to_string();
        return BlockKind::Heading;
    }
    // A standalone bold line is a heading; inline emphasis remains inline.
    if trimmed.starts_with("**") && trimmed.ends_with("**") && trimmed.matches("**").count() == 2 && trimmed.len() > 4 {
        *text = trimmed.trim_matches('*').to_string();
        return BlockKind::Heading;
    }
    for prefix in ["- ", "* ", "• "] {
        if let Some(body) = trimmed.strip_prefix(prefix) {
            *text = body.to_string();
            return BlockKind::ListItem("•".into());
        }
    }
    if let Some((number, body)) = trimmed.split_once(". ")
        && !number.is_empty()
        && number.len() <= 3
        && number.bytes().all(|b| b.is_ascii_digit())
    {
        let marker = format!("{number}.");
        *text = body.to_string();
        return BlockKind::ListItem(marker);
    }
    BlockKind::Paragraph
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
pub(crate) fn split(text: &str) -> Vec<(String, Option<bool>)> {
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
    fn translated_document_reflows_prose_without_losing_math_or_structure() {
        let blocks = document(
            "### 方法\n\n给定 \\(x_t\\)，模型预测\n未来值 \\(y_t\\)。\n\n\\[y_t=\\alpha x_t\\quad(1)\\]\n\n- 保留 **变量**。\n- 保留编号。\n\nEnglish soft\nline break.",
        );
        assert_eq!(blocks.len(), 6);
        assert_eq!(blocks[0].kind, BlockKind::Heading);
        assert_eq!(blocks[1].parts.iter().filter(|p| matches!(p, Part::Math { display: false, .. })).count(), 2);
        assert!(matches!(&blocks[2].parts[0], Part::Math { display: true, png: Ok(_), .. }));
        assert!(matches!(&blocks[3].kind, BlockKind::ListItem(_)));
        let tail: String = blocks[5].parts.iter().filter_map(|p| if let Part::Text(t) = p { Some(t.as_str()) } else { None }).collect();
        assert_eq!(tail, "English soft line break.");
        let multi = document("前文\n\\[\\begin{aligned}a&=b\\\\c&=d\\end{aligned}\\]\n后文");
        assert_eq!(multi.len(), 3);
    }

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
