# gosavetool

Parse, convert, shift and repair **SRT and WebVTT** subtitles, and count **words, characters and reading time** with **social media character limits**, **Title Case**, **slugs** and other case changes. A library and a command-line tool, with no dependencies.

Made by [GoSaveTool](https://www.gosavetool.com/), free online tools for creators.

## Command line

```bash
cargo install gosavetool

gosavetool convert movie.srt --to vtt      # writes movie.vtt
gosavetool shift movie.srt -1.5            # 1.5 s earlier, in place
gosavetool repair movie.srt                # sort, fix overlaps and bad times
gosavetool info movie.vtt                  # WebVTT, 412 cues, ends at 01:42:10.500

echo "Your caption" | gosavetool count
gosavetool limits post.txt                 # characters left for X, Instagram, TikTok, YouTube, SEO...
echo "the lord of the rings" | gosavetool case title
```

## Library

```toml
[dependencies]
gosavetool = "1"
```

```rust
use gosavetool::subtitles::{self, Format};
use gosavetool::text;

let vtt = subtitles::convert(&srt_text, Format::Vtt);
let cues = subtitles::parse(&text);                         // Cue { start, end, settings, text }, times in ms
let later = subtitles::to_srt(&subtitles::shift(&cues, 1500));
let (fixed, n) = subtitles::repair(&cues, 2000);            // sorted, no overlaps, no empty cues

let st = text::stats(post);                                 // words, chars, chars_no_spaces, lines, sentences, read_seconds
let fits_x = text::limits(post)[0].fits;                    // x-post, instagram-caption, tiktok-caption, ...
assert_eq!(text::title("the lord of the rings"), "The Lord of the Rings");
assert_eq!(text::slug("Ölçü Birimi & Çeviri!"), "olcu-birimi-ceviri");
assert_eq!(text::camel("best posting time"), "bestPostingTime");
```

Characters are Unicode code points, so an emoji counts as one. Slugs handle Turkish and the common accented Latin letters. The rules follow the [`@gosavetool/subtitles`](https://www.npmjs.com/package/@gosavetool/subtitles) and [`@gosavetool/text`](https://www.npmjs.com/package/@gosavetool/text) JavaScript packages.

## Tests

```bash
cargo test
```

## Online tools

[VTT Subtitle Generator](https://www.gosavetool.com/vtt-subtitle-generator), [Auto Captions Generator](https://www.gosavetool.com/auto-captions-generator), [Character Counter](https://www.gosavetool.com/character-counter)

## License

MIT
