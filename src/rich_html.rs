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

fn add_clause_references(content: &str) -> String {
    let res = vec![
        // TS 23.501 [2], clause 5.4.4.1b // the comma is optional
        r#"(TS\s+)?(?P<ts_no_1>(\d{2}\.\d{3}))\s+\[\d+\],?\s+[cC]lause\s+(?P<clause_no_0>(\d[\.\da-z]*[\da-z]))"#,
        // clause 5.3.3.1 (Some text) in TS 23.401 [13] // "(Some text)" is optional, "in" can be "of"
        r#"[cC]lause\s+(?P<clause_no_1>(\d[\.\da-z]*[\da-z]))\s+(\([^<^>.]+\)\s+)?((of)|(in))\s+TS\s+(?P<ts_no_2>(\d{2}\.\d{3}))\s+\[\d+\]"#,
        // in clause 4.4 // "in" can be "see" and is optional
        r#"(((in)|(see))\s+)?[cC]lause\s+(?P<clause_no_2>(\d[\.\da-z]*[\da-z]))"#,
        // in 4.3.3.2 // "in" can be "see" and is mandatory
        r#"((in)|(see))\s+(?P<clause_no_3>(\d[\.\da-z]*[\da-z]))"#,
    ];

    let joined = res.join(")|(");
    let complete_regex = format!("(?s:(?P<whole_content>(({}))))", joined);

    /*
    TS 23.501 [2] clause 5.4.4b
    TS 23.501 [2] clause 5.15.7.2
    TS 23.501 [2], clause 5.4.4.1
    23.501 [2] clause 5.6.3
    TS 29.502 [36]
    clause 5.3.3.1 (Some text) in TS 23.401 [13]
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
                for name_no in 0..res.len() {
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

/*
TS 23.501 [2] clause 5.4.4b
TS 23.501 [2] clause 5.15.7.2
TS 23.501 [2], clause 5.4.4.1
23.501 [2] clause 5.6.3
TS 29.502 [36]
clause 5.3.3.1 (Some text) in TS 23.401 [13]
clause 5.15.5.3 of TS 23.501 [2]
clause 4.4
in 4.3.3.2
Clause 4.3
*/

#[test]
fn test_add_clause_links_ts_clause() {
    let source = "Foo TS 11.222 [3] clause 4.55.6 bar";
    let expected =
        r#"Foo <a href="../11.222/11.222.html#4.55.6">TS 11.222 [3] clause 4.55.6</a> bar"#;
    assert_eq!(add_clause_references(&source), expected)
}

#[test]
fn test_add_clause_links_ts_with_comma_clause() {
    let source = "Foo TS 11.222 [3], clause 4.55.6 bar";
    let expected =
        r#"Foo <a href="../11.222/11.222.html#4.55.6">TS 11.222 [3], clause 4.55.6</a> bar"#;
    assert_eq!(add_clause_references(&source), expected)
}

#[test]
fn test_add_clause_links_ts_clause_with_letter_at_end() {
    let source = "Foo TS 11.222 [3] clause 4.55.6b bar";
    let expected =
        r#"Foo <a href="../11.222/11.222.html#4.55.6b">TS 11.222 [3] clause 4.55.6b</a> bar"#;
    assert_eq!(add_clause_references(&source), expected)
}

#[test]
fn test_add_clause_links_ts_clause_with_dot_at_end() {
    let source = "Foo TS 11.222 [3] clause 4.55.6. Bar";
    let expected =
        r#"Foo <a href="../11.222/11.222.html#4.55.6">TS 11.222 [3] clause 4.55.6</a>. Bar"#;
    assert_eq!(add_clause_references(&source), expected)
}

#[test]
fn test_add_clause_links_ts_without_ts_word_clause() {
    let source = "Foo 11.222 [3] clause 4.55.6 bar";
    let expected = r#"Foo <a href="../11.222/11.222.html#4.55.6">11.222 [3] clause 4.55.6</a> bar"#;
    assert_eq!(add_clause_references(&source), expected)
}

#[test]
fn test_add_clause_links_clause_of_ts() {
    let source = "Foo clause 11.2.33 of TS 44.555 [6] bar";
    let expected =
        r#"Foo <a href="../44.555/44.555.html#11.2.33">clause 11.2.33 of TS 44.555 [6]</a> bar"#;
    assert_eq!(add_clause_references(&source), expected)
}

#[test]
fn test_add_clause_links_clause_in_ts() {
    let source = "Foo clause 11.2.33 in TS 44.555 [6] bar";
    let expected =
        r#"Foo <a href="../44.555/44.555.html#11.2.33">clause 11.2.33 in TS 44.555 [6]</a> bar"#;
    assert_eq!(add_clause_references(&source), expected)
}

#[test]
fn test_add_clause_links_clause_some_text_in_ts() {
    let source = "Foo clause 11.2.33 (Some text) in TS 44.555 [6] bar";
    let expected = r#"Foo <a href="../44.555/44.555.html#11.2.33">clause 11.2.33 (Some text) in TS 44.555 [6]</a> bar"#;
    assert_eq!(add_clause_references(&source), expected)
}

#[test]
fn test_add_clause_links_clause() {
    let source = "Foo clause 11.2.33 bar";
    let expected = r##"Foo <a href="#11.2.33">clause 11.2.33</a> bar"##;
    assert_eq!(add_clause_references(&source), expected)
}

#[test]
fn test_add_clause_links_clause_capital() {
    let source = "Foo Clause 11.2.33, bar";
    let expected = r##"Foo <a href="#11.2.33">Clause 11.2.33</a>, bar"##;
    assert_eq!(add_clause_references(&source), expected)
}

#[test]
fn test_add_clause_links_in_clause_no() {
    let source = "Foo in 11.2.33 bar";
    let expected = r##"Foo <a href="#11.2.33">in 11.2.33</a> bar"##;
    assert_eq!(add_clause_references(&source), expected)
}

#[test]
fn test_add_clause_links_see_clause_no() {
    let source = "Foo see 11.2.33 bar";
    let expected = r##"Foo <a href="#11.2.33">see 11.2.33</a> bar"##;
    assert_eq!(add_clause_references(&source), expected)
}
