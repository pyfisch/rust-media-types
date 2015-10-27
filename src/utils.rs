use std::ascii::AsciiExt;
use std::collections::HashMap;

use error::{Error, Result};

/// `ALPHA =  %x41-5A / %x61-7A ; A-Z / a-z`
pub fn alpha(c: char) -> bool {
    c >= 'A' && c <= 'Z' || c >= 'a' && c <= 'z'
}

/// `DIGIT = %x30-39 ; 0-9`
pub fn digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

/// `tchar = "!" / "#" / "$" / "%" / "&" / "'" / "*"
/// / "+" / "-" / "." / "^" / "_" / "`" / "|" / "~"
/// / DIGIT / ALPHA ; any VCHAR, except delimiters`
pub fn tchar(c: char) -> bool {
    c == '!' || c == '#' || c == '$' || c == '%' || c == '&' || c == '\'' || c == '*' ||
    c == '+' || c == '-' || c == '.' || c == '^' ||
    c == '_' || c == '`' || c == '|' || c == '~' || digit(c) || alpha(c)
}

/// `bcharsnospace := DIGIT / ALPHA / "'" / "(" / ")" /
/// "+" / "_" / "," / "-" / "." /
/// "/" / ":" / "=" / "?"`
pub fn bcharsnospace(c: char) -> bool {
    digit(c) || alpha(c) || c == '\'' || c == '(' || c == ')' || c == '+' ||
    c == '_' || c == ',' || c == '-' || c == '.' || c == '/' || c == ':' || c == '=' || c == '?'
}

/// `bchars := bcharsnospace / " "`
pub fn bchars(c: char) -> bool {
    bcharsnospace(c) || c == ' '
}

/// `token = 1*tchar`
pub fn token(s: &str) -> bool {
    !s.is_empty() && s.chars().all(tchar)
}

/// boundary := 0*69<bchars> bcharsnospace
pub fn boundary(s: &str) -> bool {
    !s.is_empty() && s.len() <= 70 && s.chars().all(bchars) &&
    bcharsnospace(s.chars().last().unwrap())
}

fn is_whitespace(c: u8) -> bool {
    c == b' ' || c == b'\n' || c == b'\r' || c == b'\t'
}

fn is_undefined(sequence: &[u8], s: usize) -> bool {
    sequence.len() <= s
}

pub type Bytes = Vec<u8>;

pub fn parse_media_type(sequence: &[u8]) -> Result<(Bytes, Bytes, HashMap<Bytes, Bytes>)> {
    // https://mimesniff.spec.whatwg.org/#parsing-a-mime-type
    if sequence.is_empty() {
        return Err(Error::Invalid);
    }
    let mut s: usize = 0;
    let mut type_ = Vec::new();
    let mut subtype = Vec::new();
    let mut parameters = HashMap::new();
    while is_whitespace(sequence[s]) {
        s += 1;
    }
    let mut t: u8 = 0;
    loop {
        if t > 127 {
            return Err(Error::Invalid);
        }
        if is_undefined(sequence, s) {
            return Err(Error::Invalid);
        }
        if sequence[s] == b'/' {
            break;
        }
        type_.push(sequence[s].to_ascii_lowercase());
        s += 1;
        t += 1;
    }
    s += 1;
    let mut u: u8 = 0;
    loop {
        if u > 127 {
            return Err(Error::Invalid);
        }
        if is_undefined(sequence, s) {
            return Ok((type_, subtype, parameters));
        }
        if is_whitespace(sequence[s]) || sequence[s] == b';' {
            break;
        }
        subtype.push(sequence[s].to_ascii_lowercase());
        s += 1;
        u += 1;
    }
    'L: loop {
        'M: loop {
            if is_undefined(sequence, s) || sequence[s] == b';' {
                break 'M;
            }
            if is_whitespace(sequence[s]) {
                s += 1;
                continue;
            }
            if sequence[s] == b'"' {
                s += 1;
                'N: loop {
                    if (is_undefined(sequence, s) || sequence[s] == b'"') && sequence[s] == b'"' {
                        s += 1;
                        break 'N;
                    }
                    if sequence[s] == b'\\' && !is_undefined(sequence, s + 1) {
                        s += 1
                    }
                    s += 1;
                }
            } else {
                'N2: loop {
                    if is_undefined(sequence, s) || is_whitespace(sequence[s]) ||
                       sequence[s] == b';' {
                        break 'N2;
                    }
                    s += 1;
                }
            }
        }
        if is_undefined(sequence, s) {
            return Ok((type_, subtype, parameters));
        }
        s += 1;
        while is_whitespace(sequence[s]) {
            s += 1;
        }
        let mut name = Vec::new();
        let mut extra = Vec::new();
        let mut p = 0;
        'M2: loop {
            name.extend(extra.iter());
            loop {
                if !(is_whitespace(sequence[s]) || sequence[s] == b'=') {
                    if p > 127 {
                        return Err(Error::Invalid);
                    }
                    if is_undefined(sequence, s) {
                        if name != b"" && parameters.get(&name).is_none() {
                            parameters.insert(name, Vec::new());
                        }
                        return Ok((type_, subtype, parameters));
                    }
                    name.push(sequence[s].to_ascii_lowercase());
                    s += 1;
                    p += 1;
                } else {
                    break;
                }
            }
            loop {
                if is_whitespace(sequence[s]) {
                    extra.push(sequence[s]);
                    s += 1;
                    p += 1;
                } else {
                    break;
                }
            }
            if sequence[s] == b'=' {
                break 'M2;
            }
        }
        s += 1;
        loop {
            if !is_whitespace(sequence[s]) {
                break;
            }
            s += 1;
        }
        let mut value = Vec::new();
        if is_undefined(sequence, s) {
            parameters.insert(name, value);
            return Ok((type_, subtype, parameters));
        }
        if sequence[s] == b'"' {
            s += 1;
            'M3: loop {
                if is_undefined(sequence, s) || sequence[s] == b'"' {
                    parameters.insert(name, value);
                    if sequence[s] == b'"' {
                        s += 1
                    }
                    break 'M3;
                }
                if sequence[s] == b'\\' && !is_undefined(sequence, s + 1) {
                    s += 1;
                }
                value.push(sequence[s]);
                s += 1;
            }
        } else {
            'M4: loop {
                if is_undefined(sequence, s) || is_whitespace(sequence[s]) || sequence[s] == b';' {
                    parameters.insert(name, value);
                    break 'M4;
                }
                value.push(sequence[s]);
                s += 1;
            }
        }
    }
}
