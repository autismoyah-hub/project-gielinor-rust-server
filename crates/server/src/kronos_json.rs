//! JSON syntax used by the read-only Kronos combat catalog.
use std::collections::BTreeMap;
#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}
pub fn parse(text: &str) -> Result<Value, &'static str> {
    if text.len() > 16 * 1024 * 1024 {
        return Err("JSON file exceeds limit");
    }
    let mut parser = Parser {
        bytes: text.as_bytes(),
        pos: 0,
    };
    let value = parser.value(0)?;
    parser.ws();
    if parser.pos != parser.bytes.len() {
        return Err("Trailing JSON data");
    }
    Ok(value)
}
struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}
impl Parser<'_> {
    fn ws(&mut self) {
        while self
            .bytes
            .get(self.pos)
            .is_some_and(|b| matches!(b, b' ' | b'\t' | b'\r' | b'\n'))
        {
            self.pos += 1;
        }
    }
    fn take(&mut self) -> Result<u8, &'static str> {
        let b = *self.bytes.get(self.pos).ok_or("Unexpected JSON end")?;
        self.pos += 1;
        Ok(b)
    }
    fn expect(&mut self, byte: u8) -> Result<(), &'static str> {
        self.ws();
        if self.take()? == byte {
            Ok(())
        } else {
            Err("Unexpected JSON token")
        }
    }
    fn literal(&mut self, text: &[u8]) -> Result<(), &'static str> {
        for b in text {
            if self.take()? != *b {
                return Err("Invalid JSON literal");
            }
        }
        Ok(())
    }
    fn hex(&mut self) -> Result<u32, &'static str> {
        let mut code = 0;
        for _ in 0..4 {
            code = code * 16
                + (self.take()? as char)
                    .to_digit(16)
                    .ok_or("Invalid JSON unicode escape")?;
        }
        Ok(code)
    }
    fn string(&mut self) -> Result<String, &'static str> {
        self.expect(b'"')?;
        let mut bytes = vec![];
        loop {
            match self.take()? {
                b'"' => return String::from_utf8(bytes).map_err(|_| "Invalid JSON UTF-8"),
                b'\\' => match self.take()? {
                    b'"' => bytes.push(b'"'),
                    b'\\' => bytes.push(b'\\'),
                    b'/' => bytes.push(b'/'),
                    b'b' => bytes.push(8),
                    b'f' => bytes.push(12),
                    b'n' => bytes.push(10),
                    b'r' => bytes.push(13),
                    b't' => bytes.push(9),
                    b'u' => {
                        let mut code = self.hex()?;
                        if (0xd800..=0xdbff).contains(&code) {
                            self.literal(b"\\u")?;
                            let low = self.hex()?;
                            if !(0xdc00..=0xdfff).contains(&low) {
                                return Err("Invalid JSON surrogate pair");
                            }
                            code = 0x10000 + ((code - 0xd800) << 10) + (low - 0xdc00);
                        }
                        let ch = char::from_u32(code).ok_or("Invalid JSON unicode scalar")?;
                        bytes.extend(ch.encode_utf8(&mut [0; 4]).as_bytes());
                    }
                    _ => return Err("Invalid JSON escape"),
                },
                b if b < 32 => return Err("JSON string control character"),
                b => bytes.push(b),
            }
        }
    }
    fn value(&mut self, depth: usize) -> Result<Value, &'static str> {
        if depth > 64 {
            return Err("JSON nesting exceeds limit");
        }
        self.ws();
        match self
            .bytes
            .get(self.pos)
            .copied()
            .ok_or("Missing JSON value")?
        {
            b'"' => self.string().map(Value::String),
            b'n' => {
                self.literal(b"null")?;
                Ok(Value::Null)
            }
            b't' => {
                self.literal(b"true")?;
                Ok(Value::Bool(true))
            }
            b'f' => {
                self.literal(b"false")?;
                Ok(Value::Bool(false))
            }
            b'[' => {
                self.pos += 1;
                self.ws();
                let mut values = vec![];
                if self.bytes.get(self.pos) == Some(&b']') {
                    self.pos += 1;
                    return Ok(Value::Array(values));
                }
                loop {
                    values.push(self.value(depth + 1)?);
                    self.ws();
                    match self.take()? {
                        b']' => break,
                        b',' => (),
                        _ => return Err("Invalid JSON array separator"),
                    }
                }
                Ok(Value::Array(values))
            }
            b'{' => {
                self.pos += 1;
                self.ws();
                let mut values = BTreeMap::new();
                if self.bytes.get(self.pos) == Some(&b'}') {
                    self.pos += 1;
                    return Ok(Value::Object(values));
                }
                loop {
                    let key = self.string()?;
                    self.expect(b':')?;
                    values.insert(key, self.value(depth + 1)?);
                    self.ws();
                    match self.take()? {
                        b'}' => break,
                        b',' => (),
                        _ => return Err("Invalid JSON object separator"),
                    }
                }
                Ok(Value::Object(values))
            }
            b'-' | b'0'..=b'9' => {
                let start = self.pos;
                if self.bytes[self.pos] == b'-' {
                    self.pos += 1;
                }
                match self.take()? {
                    b'0' => (),
                    b'1'..=b'9' => {
                        while self.bytes.get(self.pos).is_some_and(u8::is_ascii_digit) {
                            self.pos += 1;
                        }
                    }
                    _ => return Err("Invalid JSON number"),
                }
                if self.bytes.get(self.pos) == Some(&b'.') {
                    self.pos += 1;
                    let first = self.pos;
                    while self.bytes.get(self.pos).is_some_and(u8::is_ascii_digit) {
                        self.pos += 1;
                    }
                    if first == self.pos {
                        return Err("Invalid JSON fraction");
                    }
                }
                if self
                    .bytes
                    .get(self.pos)
                    .is_some_and(|b| matches!(b, b'e' | b'E'))
                {
                    self.pos += 1;
                    if self
                        .bytes
                        .get(self.pos)
                        .is_some_and(|b| matches!(b, b'+' | b'-'))
                    {
                        self.pos += 1;
                    }
                    let first = self.pos;
                    while self.bytes.get(self.pos).is_some_and(u8::is_ascii_digit) {
                        self.pos += 1;
                    }
                    if first == self.pos {
                        return Err("Invalid JSON exponent");
                    }
                }
                Ok(Value::Number(
                    std::str::from_utf8(&self.bytes[start..self.pos])
                        .unwrap()
                        .into(),
                ))
            }
            _ => Err("Invalid JSON value"),
        }
    }
}
