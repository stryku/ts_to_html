pub struct SourceParser<'a> {
    source: &'a str,
    current_pos: usize,
}

impl SourceParser<'_> {
    pub fn new<'a>(source: &'a str) -> SourceParser<'a> {
        SourceParser {
            source: &source,
            current_pos: 0,
        }
    }

    pub fn get_current_pos(&self) -> usize {
        return self.current_pos;
    }

    pub fn is_a_before_b(&self, a: &str, b: &str) -> bool {
        let a_pos = self.find_pattern_pos(&a);
        let b_pos = self.find_pattern_pos(&b);

        if a_pos.is_none() {
            return false;
        }

        if b_pos.is_none() {
            return true;
        }

        return a_pos.unwrap() < b_pos.unwrap();
    }

    pub fn goto_begin_of(&mut self, pattern: &str) {
        self.current_pos += if let Some(found_pos) = self.find_pattern_pos(&pattern) {
            found_pos
        } else {
            self.source.len()
        }
    }

    pub fn goto_begin_of_and_get_omitted_content(&mut self, pattern: &str) -> Option<&str> {
        let old_pos = self.current_pos;
        self.goto_begin_of(pattern);
        return Some(&self.source[old_pos..self.current_pos]);
    }

    pub fn goto_end_of_and_get_omitted_content(&mut self, pattern: &str) -> Option<&str> {
        let old_pos = self.current_pos;
        self.goto_end_of(pattern);
        return Some(&self.source[old_pos..self.current_pos]);
    }

    pub fn goto_end_of(&mut self, pattern: &str) {
        self.current_pos += if let Some(found_pos) = self.find_pattern_pos(&pattern) {
            found_pos + pattern.len()
        } else {
            self.source.len()
        }
    }

    pub fn get_content_til_begin_of(&self, pattern: &str) -> Option<&str> {
        if let Some(found_pos) = self.find_pattern_pos(pattern) {
            let end_pos = self.current_pos + found_pos;
            Some(&self.source[self.current_pos..end_pos])
        } else {
            None
        }
    }

    pub fn get_content_til_end_of(&self, pattern: &str) -> Option<&str> {
        if let Some(found_pos) = self.find_pattern_pos(pattern) {
            let end_pos = self.current_pos + found_pos + pattern.len();
            Some(&self.source[self.current_pos..end_pos])
        } else {
            None
        }
    }

    fn find_pattern_pos(&self, pattern: &str) -> Option<usize> {
        return self.source[self.current_pos..].find(pattern);
    }

    pub fn is_at_end(&self) -> bool {
        return self.get_current_pos() == self.source.len();
    }
}

#[test]
fn test_source_parser_find_pattern_pos_finds_and_doesnt_move_pos() {
    let parser = SourceParser::new(&"foobarbaz");
    assert_eq!(parser.find_pattern_pos(&"bar").unwrap(), 3);
    assert_eq!(parser.get_current_pos(), 0);
}

#[test]
fn test_source_parser_goto_begin_of_goes_to_found_pos() {
    let mut parser = SourceParser::new(&"foobarbaz");
    parser.goto_begin_of(&"bar");
    assert_eq!(parser.get_current_pos(), 3);
}

#[test]
fn test_source_parser_goto_begin_of_goes_to_end_if_not_found() {
    let mut parser = SourceParser::new(&"foobarbaz");
    parser.goto_begin_of(&"nonexisting");
    assert!(parser.is_at_end());
    assert_eq!(parser.get_current_pos(), 9);
}

#[test]
fn test_source_parser_goto_begin_of_and_get_omitted_content_goes_to_found_and_returns_omitted_content(
) {
    let mut parser = SourceParser::new(&"foobarbaz");
    let omitted_content = parser
        .goto_begin_of_and_get_omitted_content(&"bar")
        .unwrap();
    assert_eq!(omitted_content, "foo");
    assert_eq!(parser.get_current_pos(), 3);
}

#[test]
fn test_source_parser_goto_end_of_finds_and_moves_pos() {
    let mut parser = SourceParser::new(&"foobarbaz");
    parser.goto_end_of(&"bar");
    assert_eq!(parser.get_current_pos(), 6);
}

#[test]
fn test_source_parser_goto_end_of_goes_to_end_if_not_found() {
    let mut parser = SourceParser::new(&"foobarbaz");
    parser.goto_end_of(&"nonexisting");
    assert!(parser.is_at_end());
    assert_eq!(parser.get_current_pos(), 9);
}
