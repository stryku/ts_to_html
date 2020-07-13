mod source_modifier;
mod source_parser;

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
    let output_file_name = format!("tmp/{}.html", path.file_stem().unwrap().to_string_lossy());

    let output = std::process::Command::new("lowriter")
        .args(&[
            "--convert-to",
            "html",
            path.to_str().unwrap(),
            &output_file_name,
            "--outdir",
            "tmp"
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
    // let re = Regex::new(
    //     r#"(?s:(?P<whole_h><h(?P<h_value>\d) (?P<h_attributes>[(class="western")(lang="x\-none" class="western")]+)>(?P<h_content>(<a name="_Toc\d+"></a>){3}\s*(?P<clause_no>[\d\.]+)\t.+)</h\d>))"#,
    // )
    // .unwrap();

    let re = Regex::new(
        r#"(?s:(?P<whole_h><h(?P<h_value>\d) (?P<h_attributes>[(class="western")(lang="x\-none" class="western")]+)>(?P<h_content>(<a name="_Toc\d+"></a>){3}\s*(?P<clause_no>[\d\.]+))))"#,
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
        "<h$h_value $h_attributes id=\"$clause_no\">$h_content",
    ));
}

fn add_clauses_references(content: &String) -> String {
    let re =
        Regex::new(r#"(?P<clause_content>clause&nbsp;(?P<clause_no>[\d\.\-a-z]*\d))"#).unwrap();

    return String::from(re.replace_all(content, "<a href=\"#$clause_no\">$clause_content</a>"));

    // println!("Captures");
    // for cap in re.captures_iter(content) {
    //     println!("__CLAUSE: {}\n", &cap["clause_no"]);
    //     // println!(
    //     //     "__H VALUE: {}, __ATTR: {}, __CLAUSE: {}, __CAPTURE: {}\n",
    //     //     &cap["h_value"], &cap["h_attributes"], &cap["clause_no"], &cap["whole_h"]
    //     // );
    // }

    // return content.clone();
}

fn extract_clause_no_from_toc_entry(toc_entry: &str) -> Option<String> {
    let mut parser = source_parser::SourceParser::new(&toc_entry);

    parser.goto_end_of("<p");
    parser.goto_end_of(">");
    if !parser.is_a_before_b("<font", "</p>") {
        return None;
    }

    parser.goto_end_of("<font");

    let is_clause_reference = parser.is_a_before_b( "<font", "</p>") && parser.is_a_before_b("<font", "</font");
    if !is_clause_reference {
        return None;
    }

    parser.goto_end_of(">");
    return Some(String::from(
        parser.get_content_til_begin_of("<font").unwrap().trim(),
    ));
}

fn better_toc2(content: &String) -> String {
    let mut modifier = source_modifier::SourceModifier::new(&content);

    println!("----------------1");
    modifier.copy_til_end_of(r#"<div id="Table of Contents1" dir="ltr">"#);

    while modifier.is_a_before_b("<p", "</div>") {
        println!("----------------2");
        modifier.copy_til_begin_of("<p");

        let p_content = modifier.get_content_til_end_of("</p>").unwrap();
        if let Some(clause_no) = extract_clause_no_from_toc_entry(&p_content) {
            println!("Handling clause {}", &clause_no);
            modifier.push_str(&format!("<a href=\"#{}\">", clause_no));
            modifier.copy_til_end_of("</p>");
            modifier.push_str("</a>");
        } else {
            println!("Cupying p");
            modifier.copy_til_end_of("</p>");
        }
    }

    modifier.copy_til_end_of_source();
    return modifier.get_result().clone();
}

fn extract_clause_no_from_h_entry(h_entry:&str) ->Option<String> {
    let mut parser = source_parser::SourceParser::new(&h_entry);

    parser.goto_end_of(">");

    while parser.is_a_before_b("<a","</h") {
        parser.goto_end_of("</a>");
    }

    let content_til_end = parser.get_content_til_begin_of("</h");
    if content_til_end.is_none(){
        return None
    }

    let trimmed_content = content_til_end.unwrap().trim();

    let is_clause_h = trimmed_content.chars().nth(0).unwrap().is_digit(10);
    if !is_clause_h {
        return None
    }

    let split = trimmed_content.split_whitespace().collect::<Vec<&str>>();
    if split.len() < 2 {
        return None
    }

    return Some(String::from(split[0]));
}

fn add_clauses_ids2(content:&String) ->String {
    let mut modifier = source_modifier::SourceModifier::new(&content);

    while modifier.is_before_end("<h")  {
        modifier.copy_til_begin_of("<h");

        let h_content = modifier.get_content_til_end_of("</h").unwrap();
        if let Some(clause_no) = extract_clause_no_from_h_entry(&h_content) {
            modifier.copy_til_end_of("<h");
            modifier.copy_chars_count(1);
            modifier.push_str(&format!(" id=\"{}\" ", clause_no));
            modifier.copy_til_end_of("</h");
        }
        else {
            modifier.copy_til_end_of("</h");
        }
    }

    modifier.copy_til_end_of_source();
    return modifier.get_result().clone();
}

fn remove_hard_spaces(content:&String) -> String {
    return content.replace("&nbsp;", " ")
}

fn remove_span_language_en_gb(content:&String) -> String {
    let re = Regex::new(r#"(?s:<span lang="en-[A-Z]{2}">(?P<span_content>(.*?))</span>)"#).unwrap();
    return String::from(re.replace_all(content, "$span_content"));
}

fn html_to_better_html(content: &String) -> String {
    let mut result = remove_hard_spaces(content);
    result = remove_span_language_en_gb(&result);
    result = better_toc2(&result);
    result = add_clauses_ids2(&result);
    return result;

    // let mut better_content = better_toc(&content);
    // better_content = add_clauses_ids(&better_content);
    // better_content = add_clauses_references(&better_content);
    // return Some(better_content);

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

    std::fs::write("output.html", &html_content);
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
