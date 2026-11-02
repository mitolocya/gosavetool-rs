//! SRT and WebVTT: read cues, write them in either format, shift and repair timings.
//!
//! The Rust version of `@gosavetool/subtitles`. Online editor: <https://www.gosavetool.com/vtt-subtitle-generator>
//!
//! ```
//! use gosavetool::subtitles::{self, Format};
//!
//! let srt = "1\n00:00:01,000 --> 00:00:02,500\nHello\n";
//! assert_eq!(subtitles::convert(srt, Format::Vtt), "WEBVTT\n\n00:00:01.000 --> 00:00:02.500\nHello\n");
//! ```

use std::fmt;
use std::str::FromStr;

/// One subtitle cue; times in milliseconds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cue {
    pub start: u64,
    pub end: u64,
    /// WebVTT cue settings, such as `align:start`; empty for SRT.
    pub settings: String,
    pub text: String,
}

/// A subtitle format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Srt,
    Vtt,
}

impl FromStr for Format {
    type Err = String;

    /// `"srt"` or `"vtt"` (also `"webvtt"`), in any case.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "srt" => Ok(Format::Srt),
            "vtt" | "webvtt" => Ok(Format::Vtt),
            _ => Err(format!("unknown subtitle format {:?}: use srt or vtt", s)),
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Format::Srt => "srt",
            Format::Vtt => "vtt",
        })
    }
}

fn all_digits(s: &str, min: usize, max: usize) -> bool {
    (min..=max).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_digit())
}

/// `"00:01:02,500"` or `"01:02.5"` to milliseconds; `None` when it is not a timestamp.
pub fn to_ms(stamp: &str) -> Option<u64> {
    let stamp = stamp.trim();
    let sep = stamp.rfind(|c: char| c == ',' || c == '.')?;
    let (clock, frac) = (&stamp[..sep], &stamp[sep + 1..]);
    if !all_digits(frac, 1, 3) {
        return None;
    }
    let parts: Vec<&str> = clock.split(':').collect();
    if !(parts.len() == 2 || parts.len() == 3) || !parts.iter().all(|p| all_digits(p, 1, 2)) {
        return None;
    }
    let nums: Vec<u64> = parts.iter().map(|p| p.parse().unwrap_or(0)).collect();
    let (h, m, s) = if nums.len() == 3 { (nums[0], nums[1], nums[2]) } else { (0, nums[0], nums[1]) };
    let ms: u64 = format!("{:0<3}", frac).parse().ok()?;
    Some(((h * 60 + m) * 60 + s) * 1000 + ms)
}

/// Milliseconds to `"00:01:02,500"` (separator `,`, SRT) or `"00:01:02.500"` (separator `.`, WebVTT).
pub fn from_ms(ms: u64, sep: char) -> String {
    format!("{:02}:{:02}:{:02}{}{:03}", ms / 3_600_000, ms / 60_000 % 60, ms / 1000 % 60, sep, ms % 1000)
}

/// `"\r\r\n"`, `"\r\n"` and a lone `"\r"` to `"\n"`, as `/\r*\n|\r/g` does.
fn normalize_newlines(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\r' {
            let mut j = i;
            while j < chars.len() && chars[j] == '\r' {
                j += 1;
            }
            if j < chars.len() && chars[j] == '\n' {
                out.push('\n');
                i = j + 1;
            } else {
                out.push('\n');
                i += 1;
            }
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    out
}

/// The start, end and settings of a `start --> end settings` line.
fn arrow(line: &str) -> Option<(u64, u64, String)> {
    let at = line.find("-->")?;
    let start = to_ms(&line[..at])?;
    let rest = line[at + 3..].trim_start();
    let stamp_len = rest.find(|c: char| !(c.is_ascii_digit() || c == ':' || c == ',' || c == '.')).unwrap_or(rest.len());
    let end = to_ms(&rest[..stamp_len])?;
    Some((start, end, rest[stamp_len..].trim().to_string()))
}

/// Cues of an SRT or WebVTT text. Headers, NOTE and STYLE blocks are skipped.
pub fn parse(src: &str) -> Vec<Cue> {
    let text = normalize_newlines(src.strip_prefix('\u{feff}').unwrap_or(src));
    let mut cues = Vec::new();
    // Blocks are separated by one or more empty lines
    let lines: Vec<&str> = text.split('\n').collect();
    for block in lines.split(|l| l.is_empty()) {
        for (i, line) in block.iter().enumerate() {
            if let Some((start, end, settings)) = arrow(line) {
                cues.push(Cue { start, end, settings, text: block[i + 1..].join("\n").trim().to_string() });
                break;
            }
        }
    }
    cues
}

/// True when the text starts with the WEBVTT signature.
pub fn is_vtt(src: &str) -> bool {
    src.strip_prefix('\u{feff}').unwrap_or(src).starts_with("WEBVTT")
}

/// SRT text, numbered from 1.
pub fn to_srt(cues: &[Cue]) -> String {
    let parts: Vec<String> = cues
        .iter()
        .enumerate()
        .map(|(i, c)| format!("{}\n{} --> {}\n{}", i + 1, from_ms(c.start, ','), from_ms(c.end, ','), c.text))
        .collect();
    parts.join("\n\n") + "\n"
}

/// WebVTT text; cue settings are kept.
pub fn to_vtt(cues: &[Cue]) -> String {
    let parts: Vec<String> = cues
        .iter()
        .map(|c| {
            let settings = if c.settings.is_empty() { String::new() } else { format!(" {}", c.settings) };
            format!("{} --> {}{}\n{}", from_ms(c.start, '.'), from_ms(c.end, '.'), settings, c.text)
        })
        .collect();
    format!("WEBVTT\n\n{}\n", parts.join("\n\n"))
}

/// Writes cues in a format.
pub fn write(cues: &[Cue], format: Format) -> String {
    match format {
        Format::Srt => to_srt(cues),
        Format::Vtt => to_vtt(cues),
    }
}

/// Converts a subtitle text to SRT or WebVTT.
pub fn convert(src: &str, to: Format) -> String {
    write(&parse(src), to)
}

/// Moves every cue by `ms` (negative = earlier); cues pushed before zero are clipped or dropped.
pub fn shift(cues: &[Cue], ms: i64) -> Vec<Cue> {
    let move_by = |t: u64| (t as i64 + ms).max(0) as u64;
    cues.iter()
        .filter(|c| move_by(c.end) > 0)
        .map(|c| Cue { start: move_by(c.start), end: move_by(c.end), ..c.clone() })
        .collect()
}

/// Repairs common faults: cues out of order, an end before its start, overlapping cues and empty cues.
/// Returns the repaired cues and the number of changes. Cues that end before they start get `min_duration`.
pub fn repair(cues: &[Cue], min_duration: u64) -> (Vec<Cue>, usize) {
    let mut fixed = cues.iter().filter(|c| c.text.is_empty()).count();
    let mut out: Vec<Cue> = cues.iter().filter(|c| !c.text.is_empty()).cloned().collect();
    let before: Vec<u64> = out.iter().map(|c| c.start).collect();
    // sort_by_key is stable: cues with the same start keep their order
    out.sort_by_key(|c| c.start);
    if out.iter().map(|c| c.start).ne(before.iter().copied()) {
        fixed += 1;
    }
    for i in 0..out.len() {
        if out[i].end <= out[i].start {
            out[i].end = out[i].start + min_duration;
            fixed += 1;
        }
        if i + 1 < out.len() && out[i].end > out[i + 1].start {
            out[i].end = (out[i].start + 1).max(out[i + 1].start.saturating_sub(1));
            fixed += 1;
        }
    }
    (out, fixed)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRT: &str = "1\r\n00:00:01,000 --> 00:00:03,500\r\nHello\r\nworld\r\n\r\n2\r\n00:00:03,000 --> 00:00:02,000\r\nSecond\r\n\r\n3\r\n00:00:00,500 --> 00:00:00,900\r\nFirst\r\n";

    #[test]
    fn parse_and_write() {
        let cues = parse(SRT);
        assert_eq!(cues.len(), 3);
        assert_eq!(cues[0].text, "Hello\nworld");
        assert!(to_vtt(&cues).starts_with("WEBVTT\n\n00:00:01.000 --> 00:00:03.500\nHello"));
        assert_eq!(convert("WEBVTT\n\nNOTE hi\n\n01:02.5 --> 01:04.000 align:start\nA\n", Format::Srt), "1\n00:01:02,500 --> 00:01:04,000\nA\n");
        assert_eq!(convert("WEBVTT\n\n01:02.5 --> 01:04.000 align:start\nA\n", Format::Vtt), "WEBVTT\n\n00:01:02.500 --> 00:01:04.000 align:start\nA\n");
        assert!("ass".parse::<Format>().is_err());
        assert_eq!("VTT".parse::<Format>(), Ok(Format::Vtt));
        assert!(parse("no cues here").is_empty());
    }

    #[test]
    fn shift_and_repair() {
        let cues = parse(SRT);
        let moved: Vec<(u64, u64)> = shift(&cues, -1000).iter().map(|c| (c.start, c.end)).collect();
        assert_eq!(moved, vec![(0, 2500), (2000, 1000)]);
        let (fixed, n) = repair(&cues, 2000);
        assert_eq!(fixed.iter().map(|c| c.text.as_str()).collect::<Vec<_>>(), vec!["First", "Hello\nworld", "Second"]);
        assert!(fixed.iter().all(|c| c.end > c.start));
        assert_eq!(n, 3);
        assert_eq!(cues[1].end, 2000);
    }

    #[test]
    fn times() {
        assert_eq!(from_ms(3_723_004, ','), "01:02:03,004");
        assert_eq!(to_ms("1:02:03.4"), Some(3_723_400));
        assert_eq!(to_ms("nope"), None);
        assert!(is_vtt("\u{feff}WEBVTT\n"));
        assert_eq!(normalize_newlines("a\r\r\nb\rc"), "a\nb\nc");
    }
}
