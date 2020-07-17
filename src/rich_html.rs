#[path = "source_modifier.rs"]
mod source_modifier;
#[path = "source_parser.rs"]
mod source_parser;

use regex::Regex;

pub fn html_to_better_html(content: &String) -> String {
    let mut result = remove_hard_spaces(content);
    result = remove_span_language_en(&result);
    result = better_toc(&result);
    result = add_clauses_ids(&result);
    result = add_clause_links(&result);
    return result;
}

fn add_clause_links(content: &str) -> String {
    println!("\tClause links...");

    let res = vec![
        // TS 23.501 [2], clause 5.4.4.1b // the comma is optional, the last letter is optional
        r#"(TS\s+)?(?P<ts_no_0>(\d{2}\.\d{3}))\s+\[\d+\],?\s+[cC]lause\s+(?P<clause_no_0>(\d[\.\da-z]*[\da-z]))"#,
        // clause 5.3.3.1 (Some text) in TS 23.401 [13] // "(Some text)" is optional, "in" can be "of"
        r#"(((in)|(see))\s+)?[cC]lause\s+(?P<clause_no_1>(\d[\.\da-z]*[\da-z]))\s+(\([^<^>.]+\)\s+)?((of)|(in))\s+TS\s+(?P<ts_no_1>(\d{2}\.\d{3}))\s+\[\d+\]"#,
        // in clause 4.4 // "in" can be "see" and it is optional
        r#"(((in)|(see))\s+)?[cC]lause\s+(?P<clause_no_2>(\d[\.\da-z]*[\da-z]))"#,
        // in 4.3.3.2 // "in" can be "see" and it is mandatory
        r#"((in)|(see))\s+(?P<clause_no_3>(\d[\.\da-z]*[\da-z]))"#,
        // TS 23.501 [2]
        r#"(TS\s+)?(?P<ts_no_2>(\d{2}\.\d{3}))\s+\[\d+\]"#,
    ];

    let joined = res.join(")|(");
    let complete_regex = format!("(?s:(?P<whole_content>(({}))))", joined);
    let re = Regex::new(complete_regex.as_str()).unwrap();

    let mut result = String::new();
    let mut last_end = 0;

    for cap in re.captures_iter(content) {
        if let Some(whole_match) = cap.get(0) {
            let whole_content = &content[whole_match.start()..whole_match.end()];
            result.push_str(&content[last_end..whole_match.start()]);
            last_end = whole_match.end();

            let mut link = String::new();

            let ts_getter = || {
                let number_of_ts = 3;
                for name_no in 0..number_of_ts {
                    let group_name = format!("ts_no_{}", name_no);
                    if let Some(clause_no) = cap.name(&group_name) {
                        return Some(clause_no);
                    }
                }

                return None;
            };
            if let Some(ts_no) = ts_getter() {
                link.push_str(&format!("../{0}/{0}.html", ts_no.as_str()));
            }

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
                link.push_str(&format!("#{}", clause_no.as_str()));
            }

            let to_insert = if !link.is_empty() {
                format!("<a href=\"{}\">{}</a>", link, whole_content)
            } else {
                String::from(whole_content)
            };

            result.push_str(&to_insert);
        }
    }
    result.push_str(&content[last_end..]);
    result
}

fn better_toc(content: &str) -> String {
    println!("\tTOC...");
    let mut modifier = source_modifier::SourceModifier::new(&content);

    modifier.copy_til_end_of(r#"<div id="Table of Contents1" dir="ltr">"#);

    while modifier.is_a_before_b("<p", "</div>") {
        modifier.copy_til_begin_of("<p");

        let p_content = modifier.get_content_til_end_of("</p>").unwrap();
        if let Some(clause_no) = extract_clause_no_from_toc_entry(&p_content) {
            modifier.push_str(&format!("<a href=\"#{}\">", clause_no));
            modifier.copy_til_end_of("</p>");
            modifier.push_str("</a>");
        } else {
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

fn add_clauses_ids(content: &str) -> String {
    println!("\tClause ids...");

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
    println!("\tRemoving hard spaces...");
    return content.replace("&nbsp;", " ");
}

fn remove_span_language_en(content: &String) -> String {
    println!("\tRemoving span_language...");
    let re = Regex::new(r#"(?s:<span lang="en-[A-Z]{2}">(?P<span_content>(.*?))</span>)"#).unwrap();
    return String::from(re.replace_all(content, "$span_content"));
}

fn extract_clause_no_from_toc_entry(toc_entry: &str) -> Option<String> {
    let re = Regex::new(r#"(?s:<[.[^<>]]+?>)"#).unwrap();
    let content = re.replace_all(toc_entry, "");
    let split_content = content.trim().split_whitespace().collect::<Vec<&str>>();
    if split_content.is_empty() {
        None
    } else {
        if split_content[0].chars().next().unwrap().is_digit(10) {
            Some(String::from(split_content[0]))
        } else {
            None
        }
    }
}

#[test]
fn test_add_clause_links_ts_clause() {
    let source = "Foo TS 11.222 [3] clause 4.55.6 bar";
    let expected =
        r#"Foo <a href="../11.222/11.222.html#4.55.6">TS 11.222 [3] clause 4.55.6</a> bar"#;
    assert_eq!(add_clause_links(&source), expected)
}

#[test]
fn test_add_clause_links_ts_with_comma_clause() {
    let source = "Foo TS 11.222 [3], clause 4.55.6 bar";
    let expected =
        r#"Foo <a href="../11.222/11.222.html#4.55.6">TS 11.222 [3], clause 4.55.6</a> bar"#;
    assert_eq!(add_clause_links(&source), expected)
}

#[test]
fn test_add_clause_links_ts_clause_with_letter_at_end() {
    let source = "Foo TS 11.222 [3] clause 4.55.6b bar";
    let expected =
        r#"Foo <a href="../11.222/11.222.html#4.55.6b">TS 11.222 [3] clause 4.55.6b</a> bar"#;
    assert_eq!(add_clause_links(&source), expected)
}

#[test]
fn test_add_clause_links_ts_clause_with_dot_at_end() {
    let source = "Foo TS 11.222 [3] clause 4.55.6. Bar";
    let expected =
        r#"Foo <a href="../11.222/11.222.html#4.55.6">TS 11.222 [3] clause 4.55.6</a>. Bar"#;
    assert_eq!(add_clause_links(&source), expected)
}

#[test]
fn test_add_clause_links_ts_without_ts_word_clause() {
    let source = "Foo 11.222 [3] clause 4.55.6 bar";
    let expected = r#"Foo <a href="../11.222/11.222.html#4.55.6">11.222 [3] clause 4.55.6</a> bar"#;
    assert_eq!(add_clause_links(&source), expected)
}

#[test]
fn test_add_clause_links_clause_of_ts() {
    let source = "Foo in clause 11.2.33 of TS 44.555 [6] bar";
    let expected =
        r#"Foo <a href="../44.555/44.555.html#11.2.33">in clause 11.2.33 of TS 44.555 [6]</a> bar"#;
    assert_eq!(add_clause_links(&source), expected)
}

#[test]
fn test_add_clause_links_clause_in_ts() {
    let source = "Foo TS 11.222 [33] bar";
    let expected = r#"Foo <a href="../11.222/11.222.html">TS 11.222 [33]</a> bar"#;
    assert_eq!(add_clause_links(&source), expected)
}

#[test]
fn test_add_clause_links_clause_some_text_in_ts() {
    let source = "Foo clause 11.2.33 (Some text) in TS 44.555 [6] bar";
    let expected = r#"Foo <a href="../44.555/44.555.html#11.2.33">clause 11.2.33 (Some text) in TS 44.555 [6]</a> bar"#;
    assert_eq!(add_clause_links(&source), expected)
}

#[test]
fn test_add_clause_links_clause() {
    let source = "Foo clause 11.2.33 bar";
    let expected = r##"Foo <a href="#11.2.33">clause 11.2.33</a> bar"##;
    assert_eq!(add_clause_links(&source), expected)
}

#[test]
fn test_add_clause_links_clause_capital() {
    let source = "Foo Clause 11.2.33, bar";
    let expected = r##"Foo <a href="#11.2.33">Clause 11.2.33</a>, bar"##;
    assert_eq!(add_clause_links(&source), expected)
}

#[test]
fn test_add_clause_links_in_clause_no() {
    let source = "Foo in 11.2.33 bar";
    let expected = r##"Foo <a href="#11.2.33">in 11.2.33</a> bar"##;
    assert_eq!(add_clause_links(&source), expected)
}

#[test]
fn test_add_clause_links_see_clause_no() {
    let source = "Foo see 11.2.33 bar";
    let expected = r##"Foo <a href="#11.2.33">see 11.2.33</a> bar"##;
    assert_eq!(add_clause_links(&source), expected)
}

#[test]
fn test_add_clause_links_ts() {
    let source = "Foo see 11.2.33 bar";
    let expected = r##"Foo <a href="#11.2.33">see 11.2.33</a> bar"##;
    assert_eq!(add_clause_links(&source), expected)
}

#[test]
fn test_add_clause_links_doesnt_replace_regular_sentence_with_in_see() {
    let source = "Foo in bar, see baz. Qux";
    let expected = r#"Foo in bar, see baz. Qux"#;
    assert_eq!(add_clause_links(&source), expected)
}

#[test]
fn test_add_clause_links_doesnt_replace_standalone_number() {
    let source = "Foo 4.5 bar";
    let expected = r#"Foo 4.5 bar"#;
    assert_eq!(add_clause_links(&source), expected)
}

#[test]
fn test_remove_span_language_en() {
    let source =
        r#"FOO <span lang="en-GB"> BAR </span> BAZ <span lang="en-US"> QUX </span> TOP KEK"#;
    let expected = r#"FOO  BAR  BAZ  QUX  TOP KEK"#;
    assert_eq!(remove_span_language_en(&String::from(source)), expected)
}

#[test]
fn test_extract_clause_no_from_toc_entry() {
    let source = r#"<p lang="en-GB" style="margin-left: 0.79in;">
	5.17.2<font face="Calibri, sans-serif"><font size="2" style="font-size: 11pt"><span lang="en-US">	</span></font></font>Interworking
	with EPC	<a href="\#__RefHeading___Toc19177586">164</a></p>"#;

    let expected = "5.17.2";

    let result = extract_clause_no_from_toc_entry(&source);
    assert!(!result.is_none());
    assert_eq!(result.unwrap(), expected)
}

#[test]
fn test_extract_clause_no_from_h_entry() {
    let source = r##"<h4 lang="en-US" class="western"><a name="__RefHeading___Toc19183553"></a>
4.2.3.3	Lorem ipsum dolor sit amet</h4>
<p lang="en-GB" class="western" style="margin-bottom: 0.13in; line-height: 100%">
Lorem ipsum dolor sit amet, consectetur adipiscing elit, .</p>"##;

    let expected = "4.2.3.3";

    let result = extract_clause_no_from_h_entry(&source);
    assert!(!result.is_none());
    assert_eq!(result.unwrap(), expected)
}

#[test]
fn test_add_clauses_ids() {
    let source = r##"<h1 lang="en-US" class="western"><a name="__RefHeading___Toc19183553"></a>
1.2.3	Lorem ipsum dolor sit amet</h1>
<p lang="en-GB" class="western" style="margin-bottom: 0.13in; line-height: 100%">
Lorem ipsum dolor sit amet, consectetur adipiscing elit, .</p>

<h2 lang="en-US" class="western"><a name="__RefHeading___Toc19183553"></a>
4.5	Lorem ipsum dolor sit amet</h2>
<p lang="en-GB" class="western" style="margin-bottom: 0.13in; line-height: 100%">
Lorem ipsum dolor sit amet, consectetur adipiscing elit, .</p>"##;

    let expected = r##"<h1 id="1.2.3"  lang="en-US" class="western"><a name="__RefHeading___Toc19183553"></a>
1.2.3	Lorem ipsum dolor sit amet</h1>
<p lang="en-GB" class="western" style="margin-bottom: 0.13in; line-height: 100%">
Lorem ipsum dolor sit amet, consectetur adipiscing elit, .</p>

<h2 id="4.5"  lang="en-US" class="western"><a name="__RefHeading___Toc19183553"></a>
4.5	Lorem ipsum dolor sit amet</h2>
<p lang="en-GB" class="western" style="margin-bottom: 0.13in; line-height: 100%">
Lorem ipsum dolor sit amet, consectetur adipiscing elit, .</p>"##;

    assert_eq!(add_clauses_ids(&source), expected);
}
