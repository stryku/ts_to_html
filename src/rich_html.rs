#[path = "source_modifier.rs"]
mod source_modifier;
#[path = "source_parser.rs"]
mod source_parser;

use regex::Regex;

pub fn html_to_better_html(content: &String) -> String {
    let mut result = remove_hard_spaces(content);
    result = remove_span_language_en_gb(&result);
    result = better_toc2(&result);
    result = add_clauses_ids2(&result);
    result = add_clause_references(&result);
    return result;
}

fn add_clause_references(content: &String) -> String {
    let res = vec![
        r#"(TS\s+)?(?P<ts_no_1>(\d{2}\.\d{3}))\s+\[\d+\],?\s+[cC]lause\s+(?P<clause_no_1>(\d[\.\da-z]*[\da-z]))"#,
        r#"[cC]lause\s+(?P<clause_no_2>(\d[\.\da-z]*[\da-z]))\s+\([^<^>.]+\)\s+((of)|(in))\s+TS\s+(?P<ts_no_2>(\d{2}\.\d{3}))\s+\[\d+\]"#,
        r#"(((in)|(see))\s+)?[cC]lause\s+(?P<clause_no_3>(\d[\.\da-z]*[\da-z]))"#,
        r#"((in)|(see))\s+(?P<clause_no_4>(\d[\.\da-z]*[\da-z]))"#,
    ];

    let joined = res.join(")|(");
    let complete_regex = format!("(?s:(?P<whole_content>(({}))))", joined);

    /*
    clause 5.3.3.1 (Tracking Area Update procedure with Serving GW change) in TS 23.401 [13]
    TS 23.501 [2] clause 5.4.4b
    TS 23.501 [2] clause 5.15.7.2
    TS 23.501 [2], clause 5.4.4.1
    23.501 [2] clause 5.6.3
    TS 29.502 [36]
    clause 5.15.5.3 of TS 23.501 [2]
    clause 4.4
    in 4.3.3.2
    Clause 4.3
    */

    let re = Regex::new(complete_regex.as_str()).unwrap();

    let mut result = String::new();
    let mut last_end = 0;

    for cap in re.captures_iter(content) {
        if let Some(whole_match) = cap.get(0) {
            let whole_content = &content[whole_match.start()..whole_match.end()];
            result.push_str(&content[last_end..whole_match.start()]);
            last_end = whole_match.end();

            let ts_no = if let Some(ts_1) = cap.name("ts_no_1") {
                Some(ts_1)
            } else if let Some(ts_2) = cap.name("ts_no_2") {
                Some(ts_2)
            } else {
                None
            };

            let clause_getter = || {
                for name_no in 1..res.len() {
                    let group_name = format!("clause_no_{}", name_no);
                    if let Some(clause_no) = cap.name(&group_name) {
                        return Some(clause_no);
                    }
                }

                return None;
            };

            if let Some(clause_no) = clause_getter() {
                let to_insert = if !ts_no.is_none() {
                    format!(
                        "<a href=\"../{ts_no}/{ts_no}.html#{clause_no}\">{whole_content}</a>",
                        ts_no = ts_no.unwrap().as_str(),
                        clause_no = clause_no.as_str(),
                        whole_content = whole_content
                    )
                } else {
                    format!(
                        "<a href=\"#{clause_no}\">{whole_content}</a>",
                        clause_no = clause_no.as_str(),
                        whole_content = whole_content
                    )
                };

                result.push_str(&to_insert);
            }
        }
    }
    result.push_str(&content[last_end..]);
    result
}

fn better_toc2(content: &String) -> String {
    let mut modifier = source_modifier::SourceModifier::new(&content);

    modifier.copy_til_end_of(r#"<div id="Table of Contents1" dir="ltr">"#);

    while modifier.is_a_before_b("<p", "</div>") {
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

fn extract_clause_no_from_h_entry(h_entry: &str) -> Option<String> {
    let mut parser = source_parser::SourceParser::new(&h_entry);

    parser.goto_end_of(">");

    while parser.is_a_before_b("<a", "</h") {
        parser.goto_end_of("</a>");
    }

    let content_til_end = parser.get_content_til_begin_of("</h");
    if content_til_end.is_none() {
        return None;
    }

    let trimmed_content = content_til_end.unwrap().trim();

    let is_clause_h = trimmed_content.chars().nth(0).unwrap().is_digit(10);
    if !is_clause_h {
        return None;
    }

    let split = trimmed_content.split_whitespace().collect::<Vec<&str>>();
    if split.len() < 2 {
        return None;
    }

    return Some(String::from(split[0]));
}

fn add_clauses_ids2(content: &String) -> String {
    let mut modifier = source_modifier::SourceModifier::new(&content);

    while modifier.is_before_end("<h") {
        modifier.copy_til_begin_of("<h");

        let h_content = modifier.get_content_til_end_of("</h").unwrap();
        if let Some(clause_no) = extract_clause_no_from_h_entry(&h_content) {
            modifier.copy_til_end_of("<h");
            modifier.copy_chars_count(1);
            modifier.push_str(&format!(" id=\"{}\" ", clause_no));
            modifier.copy_til_end_of("</h");
        } else {
            modifier.copy_til_end_of("</h");
        }
    }

    modifier.copy_til_end_of_source();
    return modifier.get_result().clone();
}

fn remove_hard_spaces(content: &String) -> String {
    return content.replace("&nbsp;", " ");
}

fn remove_span_language_en_gb(content: &String) -> String {
    let re = Regex::new(r#"(?s:<span lang="en-[A-Z]{2}">(?P<span_content>(.*?))</span>)"#).unwrap();
    return String::from(re.replace_all(content, "$span_content"));
}

fn extract_clause_no_from_toc_entry(toc_entry: &str) -> Option<String> {
    let mut parser = source_parser::SourceParser::new(&toc_entry);

    parser.goto_end_of("<p");
    parser.goto_end_of(">");
    if !parser.is_a_before_b("<font", "</p>") {
        return None;
    }

    parser.goto_end_of("<font");

    let is_clause_reference =
        parser.is_a_before_b("<font", "</p>") && parser.is_a_before_b("<font", "</font");
    if !is_clause_reference {
        return None;
    }

    parser.goto_end_of(">");
    return Some(String::from(
        parser.get_content_til_begin_of("<font").unwrap().trim(),
    ));
}
