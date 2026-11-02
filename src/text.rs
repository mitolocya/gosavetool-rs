//! Word, character and reading-time counts, social media character limits, case changes, URL slugs and
//! whitespace cleanup.
//!
//! The Rust version of `@gosavetool/text`. Online counter: <https://www.gosavetool.com/character-counter>
//!
//! ```
//! use gosavetool::text;
//!
//! assert_eq!(text::stats("Hello world.").words, 2);
//! assert_eq!(text::title("the lord of the rings"), "The Lord of the Rings");
//! assert_eq!(text::slug("Ölçü Birimi & Çeviri!"), "olcu-birimi-ceviri");
//! ```

/// Character limits of common fields; SEO values are the length search results show before cutting.
pub const LIMITS: [(&str, usize); 9] = [
    ("x-post", 280),
    ("instagram-caption", 2200),
    ("tiktok-caption", 4000),
    ("youtube-title", 100),
    ("youtube-description", 5000),
    ("linkedin-post", 3000),
    ("pinterest-description", 500),
    ("seo-title", 60),
    ("meta-description", 160),
];

const SMALL: [&str; 20] = [
    "a", "an", "and", "as", "at", "but", "by", "for", "in", "nor", "of", "on", "or", "so", "the", "to", "up", "yet", "via", "vs",
];

/// Counts of a text. Characters are Unicode code points, so an emoji counts as one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    pub words: usize,
    pub chars: usize,
    pub chars_no_spaces: usize,
    pub lines: usize,
    pub sentences: usize,
    /// At 238 words a minute, the average silent reading speed for English non-fiction.
    pub read_seconds: u64,
}

/// How a text meets one character limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limit {
    pub field: &'static str,
    pub limit: usize,
    pub used: usize,
    /// Negative when the text is too long.
    pub left: i64,
    pub fits: bool,
}

fn is_space(c: char) -> bool {
    c.is_whitespace() || c == '\u{feff}'
}

fn is_terminator(c: char) -> bool {
    matches!(c, '.' | '!' | '?' | '\u{2026}')
}

/// Words, characters, lines, sentences and reading time of a text.
pub fn stats(s: &str) -> Stats {
    let (mut words, mut chars, mut no_spaces, mut sentences, mut breaks) = (0, 0, 0, 0, 0);
    let (mut in_word, mut prev_body, mut prev_cr) = (false, false, false);
    for c in s.chars() {
        chars += 1;
        // A word starts with a letter or digit and goes on over letters, digits, apostrophes, _ and -
        if in_word {
            in_word = c.is_alphanumeric() || matches!(c, '\'' | '\u{2019}' | '_' | '-');
        } else if c.is_alphanumeric() {
            in_word = true;
            words += 1;
        }
        if !is_space(c) {
            no_spaces += 1;
        }
        // A sentence is text followed by . ! ? or …
        if is_terminator(c) {
            if prev_body {
                sentences += 1;
            }
            prev_body = false;
        } else {
            prev_body = true;
        }
        if c == '\r' || (c == '\n' && !prev_cr) {
            breaks += 1;
        }
        prev_cr = c == '\r';
    }
    Stats {
        words,
        chars,
        chars_no_spaces: no_spaces,
        lines: if s.is_empty() { 0 } else { breaks + 1 },
        sentences,
        read_seconds: (words as f64 / 238.0 * 60.0 + 0.5).floor() as u64,
    }
}

/// How the text meets every limit in [`LIMITS`], in that order.
pub fn limits(s: &str) -> Vec<Limit> {
    let used = s.chars().count();
    LIMITS
        .iter()
        .map(|&(field, limit)| Limit { field, limit, used, left: limit as i64 - used as i64, fits: used <= limit })
        .collect()
}

/// `"bestPostingTime 2"` to `["best", "Posting", "Time", "2"]`.
pub fn words(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut prev: Option<char> = None;
    for c in s.chars() {
        if c.is_alphanumeric() {
            if let Some(p) = prev {
                if c.is_uppercase() && (p.is_lowercase() || p.is_numeric()) && !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            cur.push(c);
            prev = Some(c);
        } else {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            prev = None;
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

pub fn upper(s: &str) -> String {
    s.to_uppercase()
}

pub fn lower(s: &str) -> String {
    s.to_lowercase()
}

fn capitalize(w: &str) -> String {
    let mut it = w.chars();
    match it.next() {
        Some(first) => first.to_uppercase().collect::<String>() + it.as_str(),
        None => String::new(),
    }
}

fn flush_title(word: &mut String, out: &mut String, first: &mut bool, after_colon: &mut bool) {
    if word.is_empty() {
        return;
    }
    if *first || *after_colon || !SMALL.contains(&word.as_str()) {
        out.push_str(&capitalize(word));
    } else {
        out.push_str(word);
    }
    word.clear();
    *first = false;
    *after_colon = false;
}

/// `"the lord of the rings: return of the king"` to `"The Lord of the Rings: Return of the King"`.
///
/// Short words (a, of, the, and…) stay in lower case unless they start the text or follow `: . ! ?`.
pub fn title(s: &str) -> String {
    let low = s.to_lowercase();
    let mut out = String::with_capacity(low.len());
    let mut word = String::new();
    let (mut first, mut after_colon) = (true, false);
    for c in low.chars() {
        if c.is_alphanumeric() || (!word.is_empty() && matches!(c, '\'' | '\u{2019}')) {
            word.push(c);
        } else {
            flush_title(&mut word, &mut out, &mut first, &mut after_colon);
            out.push(c);
            if matches!(c, ':' | '.' | '!' | '?') {
                after_colon = true;
            } else if !is_space(c) {
                after_colon = false;
            }
        }
    }
    flush_title(&mut word, &mut out, &mut first, &mut after_colon);
    out
}

/// `"HELLO WORLD. how ARE you?"` to `"Hello world. How are you?"`.
pub fn sentence(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    // At the start, or after . ! ? … and at least one space
    let (mut start, mut after_term) = (true, false);
    for c in s.to_lowercase().chars() {
        if start && c.is_alphabetic() {
            out.extend(c.to_uppercase());
            start = false;
            after_term = false;
            continue;
        }
        out.push(c);
        if is_terminator(c) {
            after_term = true;
            start = false;
        } else if is_space(c) {
            if after_term {
                start = true;
            }
        } else {
            start = false;
            after_term = false;
        }
    }
    out
}

// Latin letters with accents and their plain forms, for slugs
const FOLDS: [(&str, &str); 25] = [
    ("àáâãäåāăąǎÀÁÂÃÄÅĀĂĄǍ", "a"),
    ("çćĉċčÇĆĈĊČ", "c"),
    ("ďđðĎĐÐ", "d"),
    ("èéêëēĕėęěÈÉÊËĒĔĖĘĚ", "e"),
    ("ĝğġģĜĞĠĢ", "g"),
    ("ĥħĤĦ", "h"),
    ("ìíîïĩīĭįıÌÍÎÏĨĪĬĮİ", "i"),
    ("ĵĴ", "j"),
    ("ķĶ", "k"),
    ("ĺļľŀłĹĻĽĿŁ", "l"),
    ("ñńņňŉÑŃŅŇ", "n"),
    ("òóôõöøōŏőǒÒÓÔÕÖØŌŎŐǑ", "o"),
    ("ŕŗřŔŖŘ", "r"),
    ("śŝşšșŚŜŞŠȘ", "s"),
    ("ţťŧțŢŤŦȚ", "t"),
    ("ùúûüũūŭůűųǔÙÚÛÜŨŪŬŮŰŲǓ", "u"),
    ("ŵŴ", "w"),
    ("ýÿŷÝŸŶ", "y"),
    ("źżžŹŻŽ", "z"),
    ("ß", "ss"),
    ("æÆ", "ae"),
    ("œŒ", "oe"),
    ("þÞ", "th"),
    ("ﬁ", "fi"),
    ("ﬂ", "fl"),
];

/// `"Ölçü Birimi & Çeviri!"` to `"olcu-birimi-ceviri"`. Handles Turkish and the common accented Latin letters.
pub fn slug(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut dash = false;
    for c in s.chars() {
        let mut buf = [0u8; 4];
        let plain: &str = match FOLDS.iter().find(|(group, _)| group.contains(c)) {
            Some(&(_, p)) => p,
            None => c.encode_utf8(&mut buf),
        };
        for p in plain.chars() {
            let p = p.to_ascii_lowercase();
            if p.is_ascii_lowercase() || p.is_ascii_digit() {
                if dash && !out.is_empty() {
                    out.push('-');
                }
                dash = false;
                out.push(p);
            } else {
                dash = true;
            }
        }
    }
    out
}

pub fn camel(s: &str) -> String {
    words(s)
        .iter()
        .enumerate()
        .map(|(i, w)| if i == 0 { w.to_lowercase() } else { capitalize(&w.to_lowercase()) })
        .collect()
}

pub fn snake(s: &str) -> String {
    words(s).iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join("_")
}

pub fn kebab(s: &str) -> String {
    words(s).iter().map(|w| w.to_lowercase()).collect::<Vec<_>>().join("-")
}

fn clean_line(line: &str) -> String {
    let line = line.trim_end_matches(|c: char| c == ' ' || c == '\t');
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == ' ' || chars[i] == '\t' {
            let mut j = i;
            while j < chars.len() && (chars[j] == ' ' || chars[j] == '\t') {
                j += 1;
            }
            // A run of two or more after a visible character becomes one space
            if j - i >= 2 && i > 0 && !is_space(chars[i - 1]) {
                out.push(' ');
            } else {
                out.extend(&chars[i..j]);
            }
            i = j;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

/// Removes trailing spaces, runs of spaces and more than one blank line in a row.
pub fn clean(s: &str) -> String {
    let nl = if s.contains("\r\n") { "\r\n" } else { "\n" };
    let parts: Vec<&str> = s.split('\n').collect();
    let last = parts.len() - 1;
    let lines: Vec<String> = parts
        .iter()
        .enumerate()
        .map(|(i, &p)| clean_line(if i < last { p.strip_suffix('\r').unwrap_or(p) } else { p }))
        .collect();
    let mut out = lines.join(nl);
    let (three, two) = (nl.repeat(3), nl.repeat(2));
    while out.contains(&three) {
        out = out.replace(&three, &two);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts() {
        assert_eq!(
            stats("Hello world. It’s 2026!"),
            Stats { words: 4, chars: 23, chars_no_spaces: 20, lines: 1, sentences: 2, read_seconds: 1 }
        );
        let st = stats("Hello world. It’s 2026!\nSecond line.");
        assert_eq!((st.words, st.lines, st.sentences), (6, 2, 3));
        assert_eq!(stats("").lines, 0);
        assert_eq!(stats("\u{1F600}a").chars, 2);
        assert_eq!(stats("Ölçü birimi çeviri").words, 3);
        let x = limits(&"x".repeat(300))[0];
        assert_eq!((x.field, x.limit, x.used, x.left, x.fits), ("x-post", 280, 300, -20, false));
        assert_eq!(limits("a").len(), 9);
    }

    #[test]
    fn cases() {
        assert_eq!(title("the lord of the rings: return of the king"), "The Lord of the Rings: Return of the King");
        assert_eq!(title("it’s a dog’s life"), "It’s a Dog’s Life");
        assert_eq!(sentence("HELLO WORLD. how ARE you?"), "Hello world. How are you?");
        assert_eq!(slug("Ölçü Birimi & Çeviri!"), "olcu-birimi-ceviri");
        assert_eq!(slug("Straße café"), "strasse-cafe");
        assert_eq!(slug("--Hello, World--"), "hello-world");
        assert_eq!(camel("best posting time"), "bestPostingTime");
        assert_eq!(snake("bestPostingTime"), "best_posting_time");
        assert_eq!(kebab("Best Posting_Time"), "best-posting-time");
        assert_eq!(words("HTMLParser v2Beta"), vec!["HTMLParser", "v2", "Beta"]);
        assert_eq!(upper("straße"), "STRASSE");
    }

    #[test]
    fn whitespace() {
        assert_eq!(clean("a   b  \n\n\n\nc\t"), "a b\n\nc");
        assert_eq!(clean("a  \r\n\r\n\r\n\r\nb"), "a\r\n\r\nb");
        assert_eq!(clean("  indented  twice"), "  indented twice");
    }
}
