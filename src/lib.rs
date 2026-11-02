//! Subtitle and text tools from [GoSaveTool](https://www.gosavetool.com/), free online tools for creators.
//!
//! - [`subtitles`]: parse, convert, shift and repair SRT and WebVTT subtitles.
//! - [`text`]: word, character and reading-time counts, social media character limits, Title Case, slugs and other
//!   case changes, whitespace cleanup.
//!
//! No dependencies. The rules are the same as the `@gosavetool/subtitles` and `@gosavetool/text` JavaScript packages.
//!
//! ```
//! use gosavetool::{subtitles, text};
//!
//! let cues = subtitles::parse("1\n00:00:02,000 --> 00:00:01,000\nHi\n");
//! let (fixed, n) = subtitles::repair(&cues, 2000);
//! assert_eq!((fixed[0].end, n), (4000, 1));
//!
//! assert!(text::limits("Short and sweet.").iter().all(|l| l.fits));
//! ```

#![forbid(unsafe_code)]

pub mod subtitles;
pub mod text;
