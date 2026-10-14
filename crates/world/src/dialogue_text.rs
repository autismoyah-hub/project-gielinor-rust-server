//! Kotlin TextAlignment page wrapping, including its explicit <br> separators.
#[derive(Clone, Debug)]
pub struct Font {
    pub advances: [i32; 256],
    pub kerning: Vec<i8>,
}
impl Font {
    pub fn pages(&self, text: &str) -> Result<Vec<(String, usize)>, &'static str> {
        self.pages_with_width(text, 380)
    }
    pub fn message_pages(&self, text: &str) -> Result<Vec<(String, usize)>, &'static str> {
        self.pages_with_width(text, 470)
    }
    fn pages_with_width(
        &self,
        text: &str,
        line_width: i32,
    ) -> Result<Vec<(String, usize)>, &'static str> {
        let chars: Vec<u16> = text.encode_utf16().collect();
        let mut built = Vec::<u16>::new();
        let mut lines = Vec::new();
        let (mut width, mut start, mut word, mut word_width, mut spacing, mut tag, mut prev) =
            (0i32, 0usize, None, 0i32, 0usize, None, 0u16);
        let advance = |c: u16| self.advances[(if c == 160 { 32 } else { c } & 255) as usize];
        let kern = |a: u16, b: u16| -> Result<i32, &'static str> {
            if self.kerning.is_empty() || a == 0 {
                return Ok(0);
            }
            self.kerning
                .get(((a as usize) << 8) | b as usize)
                .map(|v| *v as i32)
                .ok_or("font kerning character outside reference range")
        };
        for (index, &original) in chars.iter().enumerate() {
            let mut c = original;
            if c == 60 {
                tag = Some(index);
                continue;
            }
            if c == 62 && tag.is_some() {
                let from = tag.take().unwrap();
                let name = String::from_utf16_lossy(&chars[from + 1..index]);
                built.extend_from_slice(&chars[from..=index]);
                match name.as_str() {
                    "br" => {
                        lines.push(String::from_utf16_lossy(&built[start..]));
                        start = built.len();
                        width = 0;
                        word = None;
                        prev = 0;
                    }
                    "lt" | "gt" => {
                        let glyph = if name == "lt" { 60 } else { 62 };
                        width += advance(glyph) + kern(prev, glyph)?;
                        prev = glyph;
                    }
                    _ if name.starts_with("img=") => {
                        width += 13;
                        prev = 0;
                    }
                    _ => {}
                }
                c = 0;
            }
            if tag.is_none() {
                if c != 0 {
                    built.push(c);
                    width += advance(c) + kern(prev, c)?;
                    prev = c;
                }
                if c == 32 {
                    word = Some(built.len());
                    word_width = width;
                    spacing = 1;
                }
                if width > line_width {
                    if let Some(end) = word {
                        lines.push(String::from_utf16_lossy(&built[start..end - spacing]));
                        start = end;
                        word = None;
                        width -= word_width;
                        prev = 0;
                    }
                }
                if c == 45 {
                    word = Some(built.len());
                    word_width = width;
                    spacing = 0;
                }
            }
            if lines.len() > 8 {
                return Err("reference dialogue exceeds two pages");
            }
        }
        if built.len() > start {
            lines.push(String::from_utf16_lossy(&built[start..]));
        }
        if lines.is_empty() || lines.len() > 8 {
            return Err("reference dialogue requires one or two pages");
        }
        Ok(lines
            .chunks(4)
            .map(|page| (page.join("<br>"), page.len()))
            .collect())
    }
}
pub fn line_height(lines: usize) -> i32 {
    match lines {
        2 => 28,
        3 => 20,
        _ => 16,
    }
}
