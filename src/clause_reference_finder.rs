// #[path = "source_parser.rs"]
// mod source_parser;

// pub struct ClauseReferenceInfo<'a> {
//     clause_no: &'a str,
//     ts_no: Option<&'a str>,
//     starting_pos: usize,
// }

// // pub fn insert_references(source: &str) -> String {
// //     let mut parser = source_parser::SourceParser::new(source);

// //     // let mut _clause_ref_info = { self.find_next() };

// //     // while !clause_ref_info.is_none() {
// //     //     result.push(clause_ref_info.unwrap());
// //     //     clause_ref_info = self.find_next();
// //     // }

// //     // self.result.clone()
// // }

// fn find_next<'a>(parser: &'a mut source_parser::SourceParser) -> Option<ClauseReferenceInfo<'a>> {
//     while true {
//         // parser.peek_word();
//         // if let Some(word) = parser.peek_word() {
//         //     if word != "TS" {
//         //         continue;
//         //     }
//         // } else {
//         //     break;
//         // }

//         if let Some(ref_info) = get_ref_starting_from_ts(parser) {
//             return Some(ref_info);
//         }
//     }

//     return None;
// }

// fn get_ref_starting_from_ts<'a>(
//     parser: &'a mut source_parser::SourceParser,
// ) -> Option<ClauseReferenceInfo<'a>> {
//     let starting_pos = parser.get_current_pos();

//     // Skip "TS"
//     parser.skip_word();

//     let ts_no = parse_ts_no(parser);
//     if ts_no.is_none() {
//         return None;
//     }

//     if parse_reference_ref(parser).is_none() {
//         return None;
//     }

//     let clause_no = parse_clause_no_with_clause_word(parser);
//     if clause_no.is_none() {
//         return None;
//     }

//     return Some(ClauseReferenceInfo {
//         clause_no: clause_no.unwrap(),
//         ts_no: ts_no,
//         starting_pos: starting_pos,
//     });
// }

// fn parse_clause_no_with_clause_word<'a>(
//     parser: &'a mut source_parser::SourceParser,
// ) -> Option<&'a str> {
//     parser.skip_whitespaces();

//     let clause_word = parser.skip_word();
//     if clause_word.is_none() || clause_word.unwrap() != "clause" {
//         return None;
//     }

//     parser.skip_whitespaces();

//     let clause_no_begin_pos = parser.get_current_pos();

//     loop {
//         if !parser.current_is_digit() {
//             return None;
//         }
//         parser.advance_for_count_and_get_omitted_source(1);

//         if parser.current_is('.') {
//             if !parser.next_is_digit() {
//                 break;
//             }
//             parser.advance_for_count_and_get_omitted_source(1);
//         }
//     }

//     let clause_no_end_pos = parser.get_current_pos();

//     return parser.get_source_slice(clause_no_begin_pos, clause_no_end_pos);
// }

// fn parse_ts_no<'a>(parser: &'a mut source_parser::SourceParser) -> Option<&'a str> {
//     parser.skip_whitespaces();

//     let ts_no_begin_pos = parser.get_current_pos();

//     // Skip first two digits
//     for _ in 0..2 {
//         if !parser.current_is_digit() {
//             return None;
//         }
//         parser.advance_for_count_and_get_omitted_source(1);
//     }

//     // Skip the dot
//     if !parser.current_is('.') {
//         return None;
//     }
//     parser.advance_for_count_and_get_omitted_source(1);

//     // Skip next three digits
//     for _ in 0..3 {
//         if !parser.current_is_digit() {
//             return None;
//         }
//         parser.advance_for_count_and_get_omitted_source(1);
//     }

//     let ts_no_end_pos = parser.get_current_pos();

//     return parser.get_source_slice(ts_no_begin_pos, ts_no_end_pos);
// }

// fn parse_reference_ref<'a>(parser: &'a mut source_parser::SourceParser) -> Option<&'a str> {
//     parser.skip_whitespaces();

//     let reference_ref_begin_pos = parser.get_current_pos();

//     if !parser.current_is('[') {
//         return None;
//     }

//     loop {
//         if parser.is_at_end() {
//             return None;
//         }

//         if parser.current_is(']') {
//             break;
//         }

//         if !parser.current_is_digit() {
//             return None;
//         }
//         parser.advance_for_count_and_get_omitted_source(1);
//     }

//     // Skip ]
//     parser.advance_for_count_and_get_omitted_source(1);

//     let reference_ref_end_pos = parser.get_current_pos();

//     return parser.get_source_slice(reference_ref_begin_pos, reference_ref_end_pos);
// }
// /*
// TS 23.501 [2] clause 5.15.7.2.
// TS 23.501 [2], clause 5.4.4.1
// TS 23.501 [2] clause 5.4.4b
// 23.501 [2] clause 5.6.3
// TS 29.502 [36]
// clause 5.15.5.3 of TS 23.501 [2]
// clause 4.4
// in 4.3.3.2
// Clause 4.3
// */
// #[test]
// fn test_find_references_no_reference_returns_empty() {
//     let source = "foo bar";
//     let mut parser = source_parser::SourceParser::new(source);
//     assert!(find_next(&mut parser).is_none())
// }

// #[test]
// fn test_find_references_ts_ts_no_reference_clause_clause_no_returns_one() {
//     let source = "foo TS 11.222 [2] clause 3.4.5 bar";
//     let mut parser = source_parser::SourceParser::new(source);
//     let found = find_next(&mut parser);

//     assert!(!found.is_none());

//     let unwrapped = found.unwrap();
//     assert!(!unwrapped.ts_no.is_none());
//     assert_eq!(unwrapped.ts_no.unwrap(), "11.222");
//     assert_eq!(unwrapped.clause_no, "3.4.5");
//     assert_eq!(unwrapped.starting_pos, 3);
// }

// // #[test]
// // fn test_find_next_ts_ts_no_reference_clause_clause_no() {
// //     //TS 23.501 [2] clause 5.4.4
// //     let source = "foo TS 23.501 [2] clause 5.4.4 bar";
// //     let finder = ClauseReferencesFinder::new(&source);
// //     let found = finder.find_next();
// //     assert!(found.is_none());
// // }
