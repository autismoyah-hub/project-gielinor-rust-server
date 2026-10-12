//! Bounded codec for the persisted JSON object of integer variable ids and values.
use std::collections::BTreeMap;

pub fn encode(values: &BTreeMap<u16, i32>) -> String {
    format!(
        "{{{}}}",
        values
            .iter()
            .map(|(key, value)| format!("\"{key}\":{value}"))
            .collect::<Vec<_>>()
            .join(",")
    )
}
pub fn decode(text: &str) -> Result<BTreeMap<u16, i32>, &'static str> {
    if text.len() > 1_048_576 {
        return Err("Variable JSON exceeds limit");
    }
    let mut p = Parser {
        bytes: text.as_bytes(),
        pos: 0,
    };
    p.expect(b'{')?;
    let mut values = BTreeMap::new();
    p.ws();
    if p.peek() == Some(b'}') {
        p.pos += 1;
    } else {
        loop {
            p.expect(b'"')?;
            let mut key = String::new();
            while p.peek() != Some(b'"') {
                let mut b = p.byte()?;
                if b == b'\\' {
                    if p.byte()? != b'u' {
                        return Err("Invalid variable id escape");
                    }
                    let mut code = 0u32;
                    for _ in 0..4 {
                        code = code * 16
                            + (p.byte()? as char)
                                .to_digit(16)
                                .ok_or("Invalid JSON unicode escape")?;
                    }
                    b = u8::try_from(code).map_err(|_| "Variable id must be decimal")?;
                }
                if !b.is_ascii_digit() || key.len() >= 5 {
                    return Err("Invalid variable id");
                }
                key.push(b as char);
            }
            p.pos += 1;
            let id: u16 = key.parse().map_err(|_| "Invalid variable id")?;
            p.expect(b':')?;
            p.ws();
            let start = p.pos;
            if p.peek() == Some(b'-') {
                p.pos += 1;
            }
            let digits = p.pos;
            while p.peek().is_some_and(|b| b.is_ascii_digit()) {
                p.pos += 1;
            }
            if digits == p.pos || (p.pos - digits > 1 && p.bytes[digits] == b'0') {
                return Err("Invalid JSON integer");
            }
            let value: i32 = std::str::from_utf8(&p.bytes[start..p.pos])
                .map_err(|_| "Invalid integer")?
                .parse()
                .map_err(|_| "Variable value outside i32")?;
            values.insert(id, value);
            p.ws();
            match p.byte()? {
                b'}' => break,
                b',' => (),
                _ => return Err("Expected comma or object end"),
            }
        }
    }
    p.ws();
    if p.pos != p.bytes.len() {
        return Err("Trailing variable JSON");
    }
    Ok(values)
}
struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}
impl Parser<'_> {
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }
    fn byte(&mut self) -> Result<u8, &'static str> {
        let b = self.peek().ok_or("Truncated variable JSON")?;
        self.pos += 1;
        Ok(b)
    }
    fn ws(&mut self) {
        while self
            .peek()
            .is_some_and(|b| matches!(b, b' ' | b'\t' | b'\r' | b'\n'))
        {
            self.pos += 1;
        }
    }
    fn expect(&mut self, b: u8) -> Result<(), &'static str> {
        self.ws();
        if self.byte()? == b {
            Ok(())
        } else {
            Err("Invalid variable JSON syntax")
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn integer_extremes_zeroes_duplicates_and_escaped_keys() {
        let values = decode(" { \"\\u0037\":0, \"7\":-2147483648,\"65535\":2147483647 } ").unwrap();
        assert_eq!(values[&7], i32::MIN);
        assert_eq!(decode(&encode(&values)).unwrap(), values);
        assert!(decode("{\"1\":01}").is_err());
        assert!(decode("{\"1\":2147483648}").is_err());
        assert!(decode("{\"1\":0,}").is_err());
        assert!(decode("{\"65536\":0}").is_err());
    }
}
