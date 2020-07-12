use regex::Regex;
use structopt::StructOpt;

#[derive(StructOpt)]
struct CliArgs {
    #[structopt(parse(from_os_str))]
    input_dir: std::path::PathBuf,
    #[structopt(parse(from_os_str))]
    output_dir: std::path::PathBuf,
}

fn gather_paths_with_extension(
    dir: &std::path::PathBuf,
    ext: &str,
) -> Result<Vec<std::path::PathBuf>, Box<dyn std::error::Error>> {
    let files = std::fs::read_dir(&dir)?;

    let mut pdf_paths = Vec::new();

    for f in files {
        let path = &f.unwrap().path();

        if !path.is_file() {
            continue;
        }

        if let Some(path_ext) = path.extension() {
            if path_ext == ext {
                pdf_paths.push(path.clone());
            }
        };
    }

    return Ok(pdf_paths);
}

fn extract_ts_number_from_file_path(path: &std::path::PathBuf) -> Option<&str> {
    // Filename looks like: TSnumber-version.ext
    let filename = path.file_stem().unwrap().to_str().unwrap();

    let ts_number_and_version = filename.split("-").collect::<Vec<&str>>();
    if ts_number_and_version.len() < 2 {
        return None;
    }

    return Some(&ts_number_and_version[0]);
}

fn docx_to_html(path: &std::path::PathBuf) -> Option<String> {
    let output_file_name = format!("{}.html", path.file_stem().unwrap().to_string_lossy());

    let output = std::process::Command::new("lowriter")
        .args(&[
            "--convert-to",
            "html",
            path.to_str().unwrap(),
            &output_file_name,
        ])
        .output();

    match output {
        Err(_) => {
            return None;
        }
        _ => {}
    }

    let result = std::fs::read_to_string(&output_file_name);
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

fn better_toc(content: &String) -> String {
    let begin_pattern = r#"<div id="Table of Contents1" dir="ltr">"#;
    let toc_begin = content.find(&begin_pattern);
    if toc_begin.is_none() {
        return content.clone();
    }
    let toc_content_begin = toc_begin.unwrap() + begin_pattern.len();

    let end_pattern = r#"</div>"#;
    let toc_end = content.find(&end_pattern);
    if toc_end.is_none() {
        return content.clone();
    }
    let toc_content_end = toc_end.unwrap();

    let toc = &content[toc_content_begin..toc_content_end];

    let re = Regex::new(r"(?P<paragraph>(?s:<p.+>.*<font.+>.*?(?P<clause_no>[\d.a-z]+).*<font.+))")
        .unwrap();

    let paragraphs = toc.split("</p>").collect::<Vec<&str>>();
    let mut toc_with_links = String::new();
    for p in paragraphs {
        if re.is_match(p) {
            toc_with_links
                .push_str(&re.replace_all(p, "<a href=\"#$clause_no\">$paragraph</p></a>"));
        } else {
            toc_with_links.push_str(&format!("{}</p>", &p))
        }
    }

    return format!(
        "{}{}{}",
        &content[..toc_content_begin],
        toc_with_links,
        &content[toc_content_end..]
    );
}

fn add_clauses_ids(content: &String) -> String {
    // let re = Regex::new(r"(?s:<h(?P<h_value>\d)\s(?P<h_attributes>.*?)>(?P<h_content>.+</a>\s*(?P<clause_no>[\d.]+)\s.+)</h\d>)").unwrap();
    // let re =Regex::new(r"(?s:(?P<whole_h><h(?P<h_value>\d) (?P<h_attributes>[.^>]*)><a.*+?></a><a.*+?></a><a.*+?></a>.*?(?P<clause_no>[\d.]+)\s+[a-zA-Z\s]+.?</h\d>))").unwrap();
    let re = Regex::new(
        r#"(?s:(?P<whole_h><h(?P<h_value>\d) (?P<h_attributes>[(class="western")(lang="x\-none" class="western")]+)>(?P<h_content>(<a name="_Toc\d+"></a>){3}\s*(?P<clause_no>[\d.]+)\t[a-zA-z\s]+)</h\d>))"#,
    )
    .unwrap();

    // println!("Captures");
    // for cap in re.captures_iter(content) {
    //     println!(
    //         "__CLAUSE: {}, __ATTR: {}, __CAPTURE: {}\n",
    //         &cap["clause_no"], &cap["h_attributes"], &cap["whole_h"]
    //     );
    //     // println!(
    //     //     "__H VALUE: {}, __ATTR: {}, __CLAUSE: {}, __CAPTURE: {}\n",
    //     //     &cap["h_value"], &cap["h_attributes"], &cap["clause_no"], &cap["whole_h"]
    //     // );
    // }

    // return content.clone();

    return String::from(re.replace_all(
        content,
        "<h$h_value $h_attributes id=\"$clause_no\">$h_content</h$h_value>",
    ));
}

fn html_to_better_html(content: &String) -> Option<String> {
    let mut better_content = better_toc(&content);
    better_content = add_clauses_ids(&better_content);
    return Some(better_content);

    // let mut result_body_content = String::new();

    // for line in content.lines() {
    //     if is_toc_line(line) {
    //         result_body_content.push_str(&toc_line_to_html(line));
    //     } else if is_clause_start_line(line) {
    //         result_body_content.push_str(&clause_start_line_to_html(line));
    //     } else if is_table_start_line(line) {
    //         result_body_content.push_str(&table_start_line_to_html(line));
    //     } else {
    //         result_body_content.push_str(&regular_line_to_html(line));
    //     }
    // }

    // return Some(format!(
    //     "<html><head></head><body>{}</body>",
    //     result_body_content
    // ));
}

fn handle_file(path: &std::path::PathBuf) {
    let text_content = docx_to_html(path);
    if text_content.is_none() {
        return;
    }

    let ts_number = &extract_ts_number_from_file_path(path);
    if ts_number.is_none() {
        return;
    }

    let html_content = html_to_better_html(&text_content.unwrap());

    std::fs::write("output.html", &html_content.unwrap());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::from_args();
    println!(
        "Hello, world! in: {}, out: {}",
        args.input_dir.to_string_lossy(),
        args.output_dir.to_string_lossy()
    );

    let pdf_paths = gather_paths_with_extension(&args.input_dir, "docx")?;
    for p in pdf_paths {
        println!("{}", p.to_string_lossy());
        handle_file(&p);
    }

    Ok(())
}
