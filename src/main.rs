//! `gosavetool`: subtitle and text tools on the command line. https://www.gosavetool.com/

use gosavetool::subtitles::{self, Format};
use gosavetool::text;
use std::io::Read;
use std::path::Path;
use std::process::ExitCode;
use std::{env, fs, io};

const HELP: &str = "GoSaveTool: subtitle and text tools

Subtitles:
  gosavetool convert <file> --to srt|vtt   Write name.srt or name.vtt next to the file
  gosavetool shift <file> <seconds>        Move every cue (1.5 = later, -2 = earlier), in place
  gosavetool repair <file>                 Sort, fix overlaps and bad times, renumber, in place
  gosavetool info <file>                   Format, number of cues and the end time

Text (reads the file, or standard input without one):
  gosavetool count [file]                  Words, characters, sentences, lines, reading time
  gosavetool limits [file]                 Social media and SEO character limits
  gosavetool case <title|sentence|upper|lower|slug|camel|snake|kebab|clean> [file]

Options:
  -o, --out <file>   Write to this file instead
  --stdout           Print the result instead of writing a file

Online tools: https://www.gosavetool.com/";

fn read_text(file: Option<&String>) -> io::Result<String> {
    match file {
        Some(f) if f != "-" => fs::read_to_string(f),
        _ => {
            let mut s = String::new();
            io::stdin().read_to_string(&mut s)?;
            Ok(s)
        }
    }
}

fn run(mut args: Vec<String>) -> Result<(), String> {
    let mut take = |names: &[&str]| -> Option<String> {
        let i = args.iter().position(|a| names.contains(&a.as_str()))?;
        args.remove(i);
        if i < args.len() { Some(args.remove(i)) } else { None }
    };
    let to = take(&["--to"]);
    let out = take(&["-o", "--out"]);
    let stdout = match args.iter().position(|a| a == "--stdout") {
        Some(i) => {
            args.remove(i);
            true
        }
        None => false,
    };
    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{}", HELP);
        return Ok(());
    }
    if args.iter().any(|a| a == "-V" || a == "--version") {
        println!("gosavetool {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let save = |content: String, target: &str| -> Result<(), String> {
        if stdout {
            print!("{}", content);
            return Ok(());
        }
        let target = out.as_deref().unwrap_or(target);
        fs::write(target, content).map_err(|e| format!("cannot write {}: {}", target, e))?;
        println!("Wrote {}", target);
        Ok(())
    };

    let cmd = args[0].as_str();
    match cmd {
        "count" | "limits" | "case" => {
            let op = if cmd == "case" { Some(args.get(1).cloned().ok_or("case needs an operation, like title")?) } else { None };
            let file = args.get(if cmd == "case" { 2 } else { 1 });
            let s = read_text(file).map_err(|e| e.to_string())?;
            // A file or a pipe ends with a newline that is not part of the text being counted
            let counted = s.strip_suffix('\n').map(|t| t.strip_suffix('\r').unwrap_or(t)).unwrap_or(s.as_str());
            match cmd {
                "count" => {
                    let st = text::stats(counted);
                    println!(
                        "Words: {}\nCharacters: {}\nWithout spaces: {}\nSentences: {}\nLines: {}\nReading time: {} s",
                        st.words, st.chars, st.chars_no_spaces, st.sentences, st.lines, st.read_seconds
                    );
                }
                "limits" => {
                    for l in text::limits(counted) {
                        let state = if l.fits { format!("{} left", l.left) } else { format!("{} over", -l.left) };
                        println!("{:<22}{:>5}  {}", l.field, l.limit, state);
                    }
                }
                _ => {
                    let f: fn(&str) -> String = match op.as_deref().unwrap_or("") {
                        "title" => text::title,
                        "sentence" => text::sentence,
                        "upper" => text::upper,
                        "lower" => text::lower,
                        "slug" => text::slug,
                        "camel" => text::camel,
                        "snake" => text::snake,
                        "kebab" => text::kebab,
                        "clean" => text::clean,
                        other => return Err(format!("unknown case {:?}", other)),
                    };
                    let result = f(&s);
                    match &out {
                        Some(path) => save(result, path)?,
                        None => print!("{}", result),
                    }
                }
            }
            Ok(())
        }
        "convert" | "shift" | "repair" | "info" => {
            let file = args.get(1).ok_or("missing file")?;
            let src = fs::read_to_string(file).map_err(|e| format!("cannot read {}: {}", file, e))?;
            let cues = subtitles::parse(&src);
            if cues.is_empty() {
                return Err(format!("no subtitle cues found in {}", file));
            }
            let own = if subtitles::is_vtt(&src) { Format::Vtt } else { Format::Srt };
            match cmd {
                "convert" => {
                    let to: Format = to.ok_or("convert needs --to srt or --to vtt")?.parse()?;
                    let target = Path::new(file).with_extension(to.to_string());
                    save(subtitles::write(&cues, to), &target.to_string_lossy())
                }
                "shift" => {
                    let secs: f64 = args.get(2).map(|v| v.replace(',', ".")).and_then(|v| v.parse().ok()).ok_or("shift needs seconds, like 1.5 or -2")?;
                    save(subtitles::write(&subtitles::shift(&cues, (secs * 1000.0).round() as i64), own), file)
                }
                "repair" => {
                    let (fixed, n) = subtitles::repair(&cues, 2000);
                    save(subtitles::write(&fixed, own), file)?;
                    if !stdout {
                        println!("{}", if n > 0 { format!("{} problems fixed", n) } else { "No problems found".to_string() });
                    }
                    Ok(())
                }
                _ => {
                    let end = cues.iter().map(|c| c.end).max().unwrap_or(0);
                    let name = if own == Format::Vtt { "WebVTT" } else { "SRT" };
                    println!("{}, {} cues, ends at {}", name, cues.len(), subtitles::from_ms(end, if own == Format::Vtt { '.' } else { ',' }));
                    Ok(())
                }
            }
        }
        other => Err(format!("unknown command {:?}", other)),
    }
}

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("gosavetool: {}\n\nRun gosavetool --help for usage.", e);
            ExitCode::FAILURE
        }
    }
}
