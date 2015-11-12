

pub fn encode_uri<S: Into<String>>(c: S, full_url: bool) -> String {
  c.into().into_bytes().iter().fold(String::new(), |mut out, &b| {
    match b as char {
      'A' ... 'Z'
      | 'a' ... 'z'
      | '0' ... '9'
      | '-' | '.' | '_' | '~' => out.push(b as char),
      ':' | '/' | '?' | '#' | '[' | ']' | '@' |
      '!' | '$' | '&' | '"' | '(' | ')' | '*' |
      '+' | ',' | ';' | '='
          if full_url => out.push(b as char),

      ch => out.push_str(&format!("%{:02X}", ch as usize)),
    };

    out
  })
}

