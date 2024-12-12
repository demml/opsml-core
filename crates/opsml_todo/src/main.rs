use clap::Parser;
use opsml_utils::color::LogColors;
use rayon::prelude::*;
use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use tabled::settings::object::{Columns, Object, Rows};
use tabled::settings::panel::Header;
use tabled::settings::themes::ColumnNames;
use tabled::settings::Alignment;
use tabled::settings::{format::Format, Color, Style, Width};
use tabled::{Table, Tabled};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "todo_scanner")]
#[command(about = "Scans a directory for TODO comments", long_about = None)]
struct Cli {
    /// The path to the directory to scan
    #[arg(short, long)]
    path: Option<String>,
}

#[derive(Tabled)]
struct Todo {
    #[tabled(rename = "Line")]
    line: String,

    #[tabled(rename = "File")]
    file: String,

    #[tabled(rename = "Comment")]
    comment: String,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let parent_path = cli
        .path
        .unwrap_or_else(|| env::current_dir().unwrap().display().to_string());

    let entries: Vec<_> = WalkDir::new(&parent_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .collect();

    let todos: Vec<Todo> = entries
        .par_iter()
        .map(|entry| {
            let mut file_todos = Vec::new();
            let file_path = entry.path().display().to_string();

            // get relative path to file_path relative to parent_path
            let file_path = Path::new(&file_path)
                .strip_prefix(&parent_path)
                .unwrap_or(Path::new(&file_path))
                .display()
                .to_string();

            let file = File::open(entry.path()).unwrap();
            let reader = io::BufReader::new(file);

            for (index, line) in reader.lines().enumerate() {
                let line = line.unwrap();
                if let Some(pos) = line.find("TODO:") {
                    // Check if TODO is surrounded by quotation marks
                    let before = &line[..pos];
                    let after = &line[pos + 5..];
                    if !(before.trim_end().ends_with('"') && after.trim_start().starts_with('"')) {
                        let comment = after.trim().to_string();
                        let line_num = format!("{}", index + 1).to_string();
                        file_todos.push(Todo {
                            line: LogColors::purple(&line_num),
                            file: file_path.clone() + ":" + &(index + 1).to_string(),
                            comment,
                        });
                    }
                }
            }
            file_todos
        })
        .flatten()
        .collect();

    if todos.is_empty() {
        println!("{}", LogColors::green("No TODOs found"));
        return Ok(());
    }
    // TODO: Add table formatting

    let mut table = Table::new(todos);

    table.with(Style::sharp());
    table.modify(Columns::single(0), Width::wrap(10).keep_words(true));
    table.modify(Columns::single(1), Width::wrap(50));
    table.modify(Columns::single(2), Width::wrap(100).keep_words(true));

    table.modify(
        Rows::new(0..1),
        (
            Format::content(|s| format!("{}", LogColors::green(s))),
            Alignment::center(),
            Color::BOLD,
        ),
    );

    // TODO: Center and color
    // Color the column names

    println!("{}", &table);
    Ok(())
}
