#[path = "source_parser.rs"]
mod source_parser;

pub struct SourceModifier<'a> {
    parser: source_parser::SourceParser<'a>,
    result: String,
}

impl SourceModifier<'_> {
    pub fn new<'a>(source: &'a str) -> SourceModifier<'a> {
        SourceModifier {
            parser: source_parser::SourceParser::new(source),
            result: String::new(),
        }
    }

    pub fn copy_til_end_of(&mut self, pattern: &str) {
        let omitted = self
            .parser
            .goto_end_of_and_get_omitted_content(&pattern)
            .unwrap();
        self.result.push_str(omitted)
    }

    pub fn copy_til_begin_of(&mut self, pattern: &str) {
        let omitted = self
            .parser
            .goto_begin_of_and_get_omitted_content(&pattern)
            .unwrap();
        self.result.push_str(omitted)
    }

    pub fn is_a_before_b(&self, a: &str, b: &str) -> bool {
        self.parser.is_a_before_b(a, b)
    }

    pub fn goto_begin_of_and_get_omitted_content(&mut self, pattern: &str) -> Option<&str> {
        self.parser.goto_begin_of_and_get_omitted_content(pattern)
    }

    pub fn get_content_til_begin_of(&self, pattern: &str) -> Option<&str> {
        self.parser.get_content_til_begin_of(pattern)
    }

    pub fn get_content_til_end_of(&self, pattern: &str) -> Option<&str> {
        self.parser.get_content_til_end_of(pattern)
    }

    pub fn push_str(&mut self, s: &str) {
        self.result.push_str(s)
    }

    pub fn get_result(&self) -> &String {
        &self.result
    }
}
