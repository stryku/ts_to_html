use regex::Regex;
use structopt::StructOpt;

#[derive(StructOpt)]
struct CliArgs {
    #[structopt(parse(from_os_str))]
    input_dir: std::path::PathBuf,
    #[structopt(parse(from_os_str))]
    output_dir: std::path::PathBuf,
}

fn gather_pdf_paths(
    dir: &std::path::PathBuf,
) -> Result<Vec<std::path::PathBuf>, Box<dyn std::error::Error>> {
    let files = std::fs::read_dir(&dir)?;

    let mut pdf_paths = Vec::new();

    for f in files {
        let path = &f.unwrap().path();

        if !path.is_file() {
            continue;
        }

        if let Some(ext) = path.extension() {
            if ext == "pdf" {
                pdf_paths.push(path.clone());
            }
        };
    }

    return Ok(pdf_paths);
}

fn extract_ts_number_from_pdf_path(path: &std::path::PathBuf) -> Option<&str> {
    let filename = path.file_stem().unwrap().to_str().unwrap();

    let split = filename.split("_").collect::<Vec<&str>>();
    if split.len() < 2 {
        return None;
    }

    let ts_number_and_version = split[1];
    let ts_number_and_version_split = ts_number_and_version.split("v").collect::<Vec<&str>>();

    if ts_number_and_version_split.len() < 2 {
        return None;
    }

    return Some(&ts_number_and_version_split[0][1..]);
}

fn pdf_to_text(path: &std::path::PathBuf) -> Option<String> {
    let output = std::process::Command::new("pdftotext")
        .args(&["-layout", path.to_str().unwrap(), "tmp.txt"])
        .output();

    match output {
        Err(_) => {
            return None;
        }
        _ => {}
    }

    let result = std::fs::read_to_string("tmp.txt");
    match result {
        Ok(content) => Some(content),
        Err(_) => None,
    }
}

fn is_toc_line(line: &str) -> bool {
    return line.contains(".....");
}

fn toc_line_to_html(line: &str) -> String {
    let clause_no = line.split(" ").nth(0).unwrap();
    return format!("<a href=\"#{}\"><pre>{}</pre></a>\n", clause_no, line);
}

fn handle_table_references(line: &str) -> String {
    let re = Regex::new(r"in [tT]able (?P<table_no>[\d\.-]+) ").unwrap();
    return String::from(
        re.replace_all(line, "in <a href=\"#table_$table_no\">Table $table_no</a> "),
    );
}

fn regular_line_to_html(line: &str) -> String {
    let references_replaced = handle_table_references(line);
    return format!("<pre>{}</pre>\n", references_replaced);
}

fn is_clause_start_line(line: &str) -> bool {
    if line.is_empty() || line.starts_with("3GPP") {
        return false;
    }

    return line.chars().nth(0).unwrap().is_digit(10);
}

fn clause_start_line_to_html(line: &str) -> String {
    let clause_no = line.split(" ").nth(0).unwrap();

    let number_of_dots = clause_no.matches('.').count();
    let header_value = std::cmp::min(number_of_dots, 6);

    return format!(
        "<h{}><pre id=\"{}\">{}</pre></h{}>",
        header_value, clause_no, line, header_value
    );
}

fn is_table_start_line(line: &str) -> bool {
    if line.is_empty() {
        return false;
    }
    return line.trim().starts_with("Table ");
}

fn table_start_line_to_html(line: &str) -> String {
    let table_no_with_colon = line.trim().split(' ').nth(1).unwrap();
    let table_no = &table_no_with_colon[..table_no_with_colon.len() - 1];
    let table_id = format!("table_{}", table_no);
    return format!("<b><pre id=\"{}\">{}</pre></b>", table_id, line);
}

fn text_to_html(content: &String) -> Option<String> {
    let mut result_body_content = String::new();

    for line in content.lines() {
        if is_toc_line(line) {
            result_body_content.push_str(&toc_line_to_html(line));
        } else if is_clause_start_line(line) {
            result_body_content.push_str(&clause_start_line_to_html(line));
        } else if is_table_start_line(line) {
            result_body_content.push_str(&table_start_line_to_html(line));
        } else {
            result_body_content.push_str(&regular_line_to_html(line));
        }
    }

    return Some(format!(
        "<html><head></head><body>{}</body>",
        result_body_content
    ));
}

fn handle_pdf(path: &std::path::PathBuf) {
    let text_content = pdf_to_text(path);
    if text_content.is_none() {
        return;
    }

    let ts_number = &extract_ts_number_from_pdf_path(path);
    if ts_number.is_none() {
        return;
    }

    let html_content = text_to_html(&text_content.unwrap());

    std::fs::write("output.html", &html_content.unwrap());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::from_args();
    println!(
        "Hello, world! in: {}, out: {}",
        args.input_dir.to_string_lossy(),
        args.output_dir.to_string_lossy()
    );

    let pdf_paths = gather_pdf_paths(&args.input_dir)?;
    for p in pdf_paths {
        println!("{}", p.to_string_lossy());
        handle_pdf(&p);
    }

    Ok(())
}
