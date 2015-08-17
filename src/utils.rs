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
    c == '!' || c == '#' || c == '$' || c == '%' || c == '&' || c == '\''
        || c == '*' || c == '+' || c == '-' || c == '.' || c == '^' || c == '_'
        || c == '`' || c == '|' || c == '~' ||  digit(c) || alpha(c)
}

/// `bcharsnospace := DIGIT / ALPHA / "'" / "(" / ")" /
/// "+" / "_" / "," / "-" / "." /
/// "/" / ":" / "=" / "?"`
pub fn bcharsnospace(c: char) -> bool {
    digit(c) || alpha(c) || c == '\'' || c == '(' || c == ')'
        || c == '+' || c == '_' || c == ',' || c == '-' || c == '.'
        || c == '/' || c == ':' || c == '=' || c == '?'
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
    !s.is_empty()
        && s.len() <= 70
        && s.chars().all(bchars)
        && bcharsnospace(s.chars().last().unwrap())
}

/// Percent-decode the given bytes, and push the result to `output`.
// Code from https://github.com/servo/rust-url/blob/b32c7cc883c72fbc6e448a6ce881b74967c23ac3/
// src/percent_encoding.rs#L101-117
pub fn percent_decode_to(input: &[u8], output: &mut Vec<u8>) {
    let mut i = 0;
    while i < input.len() {
        let c = input[i];
        if c == b'%' && i + 2 < input.len() {
            if let (Some(h), Some(l)) = (from_hex(input[i + 1]), from_hex(input[i + 2])) {
                output.push(h * 0x10 + l);
                i += 3;
                continue
            }
        }

        output.push(c);
        i += 1;
    }
}

/// Percent-decode the given bytes
// Code from https://github.com/servo/rust-url/blob/b32c7cc883c72fbc6e448a6ce881b74967c23ac3/
// src/percent_encoding.rs#L120-126
#[inline]
pub fn percent_decode(input: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    percent_decode_to(input, &mut output);
    output
}

#[inline]
pub fn from_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0' ... b'9' => Some(byte - b'0'),  // 0..9
        b'A' ... b'F' => Some(byte + 10 - b'A'),  // A..F
        b'a' ... b'f' => Some(byte + 10 - b'a'),  // a..f
        _ => None
    }
}

pub fn unquote_string(x: &str) -> &str {
    let x = x.trim();
    if x.starts_with('"') && x.ends_with('"') {
        &x[1..x.len()-1]
    } else {
        x
    }
}
