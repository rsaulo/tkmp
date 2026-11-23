mod app;
mod bench;
mod doc;
mod fonts;
mod kitty;
mod pele;
mod raster;
mod svgflat;
mod term;
mod text;
mod theme;

use std::path::PathBuf;

const USAGE: &str = "\
tkmp: a live markdown previewer with Mermaid diagrams and big-type headings

usage:
  tkmp [--max-width N] FILE          watch FILE and re-render on every save
  tkmp bench FILE [--cols N] [--cell WxH]
                                    time each pipeline stage, no terminal needed

options:
  --max-width N   wrap text at N columns at most (default 110); 0 fills the pane.
                  TKMP_MAX_WIDTH=N sets the same when the option is left out.

keys: j/k scroll, space/b page, d/u half page, g/G top/bottom (G follows),
      ]/[ next/previous heading, ctrl-r redraw, q quit

TKMP_NO_GRAPHICS=1 forces text-only rendering.";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        None | Some("-h" | "--help") => {
            println!("{USAGE}");
            return;
        }
        Some("-V" | "--version") => {
            println!("tkmp {}", env!("CARGO_PKG_VERSION"));
            return;
        }
        Some("bench") => bench_args(&args[1..]),
        Some(_) => watch_args(&args),
    };
    if let Err(e) = result {
        eprintln!("tkmp: {e}");
        std::process::exit(1);
    }
}

fn watch_args(args: &[String]) -> anyhow::Result<()> {
    let mut file = None;
    let mut max_width = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let value = match a.strip_prefix("--max-width") {
            Some("") => Some(it.next().ok_or_else(|| anyhow::anyhow!("--max-width needs a value"))?.as_str()),
            Some(rest) => rest.strip_prefix('='),
            None => None,
        };
        match value {
            Some(v) => max_width = Some(v.parse().map_err(|_| anyhow::anyhow!("--max-width needs a number"))?),
            None if file.is_none() => file = Some(PathBuf::from(a)),
            None => anyhow::bail!("unexpected argument: {a}"),
        }
    }
    let file = file.ok_or_else(|| anyhow::anyhow!("needs a FILE"))?;
    if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        eprintln!("tkmp: stdout is not a terminal");
        std::process::exit(2);
    }
    app::run(&file, max_width)
}

fn bench_args(args: &[String]) -> anyhow::Result<()> {
    let mut file = None;
    let mut cols = 120;
    let mut cell = raster::Cell { w: 16, h: 28 };
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--cols" => cols = it.next().ok_or_else(|| anyhow::anyhow!("--cols needs a value"))?.parse()?,
            "--cell" => {
                let v = it.next().ok_or_else(|| anyhow::anyhow!("--cell needs WxH"))?;
                let (w, h) = v.split_once('x').ok_or_else(|| anyhow::anyhow!("--cell needs WxH"))?;
                cell = raster::Cell { w: w.parse()?, h: h.parse()? };
            }
            f => file = Some(PathBuf::from(f)),
        }
    }
    bench::run(&file.ok_or_else(|| anyhow::anyhow!("bench needs a FILE"))?, cols, cell)
}
