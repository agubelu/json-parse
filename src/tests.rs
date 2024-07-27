#[cfg(test)]
mod scanner_tests {
    use crate::data::{JsonToken, TokenKind::*};
    use crate::scanner::Scanner;

    pub const fn token(kind: crate::data::TokenKind, line: usize, column: usize) -> JsonToken {
        let pos = crate::data::TokenPosition { line, column };
        JsonToken { pos, kind }
    }

    fn _assert_token_sequence(src: &str, tokens: &[JsonToken]) {
        let mut scanner = Scanner::new(src);
        for token in tokens {
            assert_eq!(scanner.next_token().as_ref(), Ok(token));
        }
        assert!(matches!(
            scanner.next_token(),
            Ok(JsonToken { kind: Eof, .. })
        ));
    }

    fn _assert_fails(src: &str, line: usize, column: usize, error: &str) {
        let mut scanner = Scanner::new(src);
        let mut scanned;

        loop {
            scanned = scanner.next_token();
            if let Ok(JsonToken { kind: Eof, .. }) = scanned {
                panic!("Did not fail as expected");
            }

            if let Err(parse_error) = scanned {
                assert_eq!(parse_error.column, column);
                assert_eq!(parse_error.line, line);
                assert!(parse_error.msg.contains(error));
                return;
            }
        }
    }

    #[test]
    fn test_basic_values() {
        let s = r#"  null
true false
false true
[ ] { } : ,
0 1 2 200 -100
" abcde "  "123456"
        "#;

        let expected = [
            token(Null, 1, 2),
            token(True, 2, 0),
            token(False, 2, 5),
            token(False, 3, 0),
            token(True, 3, 6),
            token(LeftBracket, 4, 0),
            token(RightBracket, 4, 2),
            token(LeftBrace, 4, 4),
            token(RightBrace, 4, 6),
            token(Colon, 4, 8),
            token(Comma, 4, 10),
            token(Number(0.0), 5, 0),
            token(Number(1.0), 5, 2),
            token(Number(2.0), 5, 4),
            token(Number(200.0), 5, 6),
            token(Number(-100.0), 5, 10),
            token(String(" abcde ".into()), 6, 0),
            token(String("123456".into()), 6, 11),
        ];

        _assert_token_sequence(s, &expected);
    }

    #[test]
    fn test_eofs() {
        // Check that the scanner provides constant EOFs after running out of tokens, without further advancing.
        let s = "null";
        let mut scanner = Scanner::new(s);
        assert_eq!(scanner.next_token(), Ok(token(Null, 1, 0)));
        let first_eof = scanner.next_token().unwrap();

        for _ in 0..1000 {
            assert_eq!(first_eof, scanner.next_token().unwrap());
        }
    }

    #[test]
    fn test_number_formats() {
        // Tests many different combinations of allowed number formats
        let s = r#"
0 1 20 300 0000001 -10 -800 -0000123
0.0 0.00001 123.456 -0.111 -000.9 -0000888.88
0e+0 0E10 1e1 0000123e+2 -20E5 -11e000012 10e-3 -123e-10
0.1e+1 000.001e+100 123.4E-2 -13.37e-8 -0.0E-0
        "#;

        let expected = [
            token(Number(0.0), 2, 0),
            token(Number(1.0), 2, 2),
            token(Number(20.0), 2, 4),
            token(Number(300.0), 2, 7),
            token(Number(1.0), 2, 11),
            token(Number(-10.0), 2, 19),
            token(Number(-800.0), 2, 23),
            token(Number(-123.0), 2, 28),
            token(Number(0.0), 3, 0),
            token(Number(0.00001), 3, 4),
            token(Number(123.456), 3, 12),
            token(Number(-0.111), 3, 20),
            token(Number(-0.9), 3, 27),
            token(Number(-888.88), 3, 34),
            token(Number(0.0), 4, 0),
            token(Number(0.0), 4, 5),
            token(Number(10.0), 4, 10),
            token(Number(12300.0), 4, 14),
            token(Number(-2000000.0), 4, 25),
            token(Number(-11e12), 4, 31),
            token(Number(0.01), 4, 42),
            token(Number(-123e-10), 4, 48),
            token(Number(1.0), 5, 0),
            token(Number(1e97), 5, 7),
            token(Number(1.234), 5, 20),
            token(Number(-13.37e-8), 5, 29),
            token(Number(0.0), 5, 39),
        ];

        _assert_token_sequence(s, &expected);
    }

    #[test]
    fn test_lone_minus() {
        _assert_fails("- 132", 1, 1, "At least a digit is expected after '-'");
    }

    #[test]
    fn test_fraction_without_integer() {
        _assert_fails(".123", 1, 0, "Unexpected character: '.'");
        _assert_fails("-.123", 1, 1, "At least a digit is expected after '-'");
    }

    #[test]
    fn test_fraction_without_decimal() {
        _assert_fails(
            "0. ",
            1,
            2,
            "At least a digit is expected after a fraction dot",
        );
        _assert_fails(
            "-123.",
            1,
            5,
            "At least a digit is expected after a fraction dot",
        );
        _assert_fails(
            "1.e8",
            1,
            2,
            "At least a digit is expected after a fraction dot",
        );
    }

    #[test]
    fn test_illegal_exponents() {
        _assert_fails(
            "123e",
            1,
            4,
            "At least a digit is expected after an exponent",
        );
        _assert_fails(
            "-90EA",
            1,
            4,
            "At least a digit is expected after an exponent",
        );
        _assert_fails(
            "-90E+A",
            1,
            5,
            "At least a digit is expected after an exponent",
        );
        _assert_fails("87.0e+1.2", 1, 7, "Unexpected character: '.'");
        _assert_fails(
            "87.0e.2",
            1,
            5,
            "At least a digit is expected after an exponent",
        );
    }

    #[test]
    fn test_strings_ok() {
        let s = r#" "one"  "two" "three"
"four" "five"  "#;

        let expected = [
            token(String("one".into()), 1, 1),
            token(String("two".into()), 1, 8),
            token(String("three".into()), 1, 14),
            token(String("four".into()), 2, 0),
            token(String("five".into()), 2, 7),
        ];

        _assert_token_sequence(s, &expected);
    }

    #[test]
    fn test_string_raw_newline_error() {
        // Checks that a string can't spawn multiple lines
        let s = r#" "this is a
misbehaving string!" "#;
        _assert_fails(s, 1, 11, "Line breaks are not allowed");
    }

    #[test]
    fn test_string_control_characters() {
        // Checks that raw characters under 0x20 are rejected
        for ch in 0x00u8..0x20u8 {
            let mut s = std::string::String::from("\"blah ");
            s.push(ch as char);
            s.push_str(" blah\"");

            _assert_fails(&s, 1, 6, "not allowed inside a string");
        }
    }

    #[test]
    fn test_successful_unicode_encoding_ascii() {
        let s = r#" "\u0075\u006e\u0069\u0063\u006f\u0064\u0065" "#;
        let tokens = [token(String("unicode".into()), 1, 1)];
        _assert_token_sequence(s, &tokens);
    }

    #[test]
    fn test_successful_unicode_encoding_others() {
        let s = r#"
"\u044E\u043D\u0438\u043A\u043E\u0434"
"\u0075\u006E\u0069\u006B\u014D\u0064\u0073"
"\u7EDF\u4E00\u7801"
"\u064A\u0648\u0646\u064A\u0643\u0648\u062F"
"\u12E9\u1292\u12AE\u12F5"
"\u30E6\u30CB\u30B3\u30FC\u30C9"
"\uD83D\uDCA9"
"\u0079\u0306"
        "#;

        let tokens = [
            token(String("юникод".into()), 2, 0),
            token(String("unikōds".into()), 3, 0),
            token(String("统一码".into()), 4, 0),
            token(String("يونيكود".into()), 5, 0),
            token(String("ዩኒኮድ".into()), 6, 0),
            token(String("ユニコード".into()), 7, 0),
            token(String("💩".into()), 8, 0),
            token(String("y̆".into()), 9, 0),
        ];
        _assert_token_sequence(s, &tokens);
    }

    #[test]
    fn test_successful_unicode_literals_others() {
        let s = r#"
"юникод"
"unikōds"
"统一码"
"يونيكود"
"ዩኒኮድ"
"ユニコード"
"💩"
"y̆"
        "#;

        let tokens = [
            token(String("юникод".into()), 2, 0),
            token(String("unikōds".into()), 3, 0),
            token(String("统一码".into()), 4, 0),
            token(String("يونيكود".into()), 5, 0),
            token(String("ዩኒኮድ".into()), 6, 0),
            token(String("ユニコード".into()), 7, 0),
            token(String("💩".into()), 8, 0),
            token(String("y̆".into()), 9, 0),
        ];
        _assert_token_sequence(s, &tokens);
    }

    #[test]
    fn test_unfinished_unicode_escape() {
        let s1 = r#" "Naughty: \uAB" "#; // Unfinished within closed string
        _assert_fails(s1, 1, 16, "Invalid Unicode escape sequence");
        let s2 = r#" "Naughty: \uAB"#; // Unexpected EOF
        _assert_fails(s2, 1, 16, "Unterminated string");
    }

    #[test]
    fn test_invalid_unicode_escape() {
        let s1 = r#" "Not a hex sequence: \u00PS" "#; // Invalid hex
        _assert_fails(s1, 1, 27, "Invalid Unicode escape sequence");
        let s2 = r#" "Error: \uD821 hehe" "#; // Lone high surrogate
        _assert_fails(s2, 1, 15, "unfinished character");
        let s3 = r#" "Error: \uD834\u0075" "#; // High surrogate + invalid follow-up
        _assert_fails(s3, 1, 20, "Invalid unicode character");
    }

    #[test]
    fn test_byte_offsets() {
        // Introduces a bunch of strings with multi-byte characters,
        // then tests that the internal byte counters remain consistent
        // by using them to parse a number and a keyword.
        let s = r#"
"юникод"
"unikōds"
"统一码"
"يونيكود"
"ዩኒኮድ"
"ユニコード"
"💩"
1.8e307
"y̆y̆y̆y̆y̆y̆y̆y̆y̆"
"i love 𝄞 music 𝄞"
true"#;
        let tokens = [
            token(String("юникод".into()), 2, 0),
            token(String("unikōds".into()), 3, 0),
            token(String("统一码".into()), 4, 0),
            token(String("يونيكود".into()), 5, 0),
            token(String("ዩኒኮድ".into()), 6, 0),
            token(String("ユニコード".into()), 7, 0),
            token(String("💩".into()), 8, 0),
            token(Number(1.8e307), 9, 0),
            token(String("y̆y̆y̆y̆y̆y̆y̆y̆y̆".into()), 10, 0),
            token(String("i love 𝄞 music 𝄞".into()), 11, 0),
            token(True, 12, 0),
        ];
        _assert_token_sequence(s, &tokens);
    }
}

#[cfg(test)]
mod parser_tests {
    use crate::{
        parse,
        JsonElement::{self, *},
    };

    fn _assert_parses(json: &str, expected: JsonElement) {
        assert_eq!(parse(json), Ok(expected));
    }

    fn _assert_fails(json: &str, line: usize, col: usize, msg: &str) {
        if let Err(parse_error) = parse(json) {
            assert_eq!(parse_error.line, line);
            assert_eq!(parse_error.column, col);
            assert!(parse_error.msg.contains(msg));
        } else {
            panic!("Did not fail as expected");
        }
    }

    #[test]
    fn test_basic_values() {
        _assert_parses("null", Null);
        _assert_parses("true", Boolean(true));
        _assert_parses("false", Boolean(false));
        _assert_parses("0", Number(0.0));
        _assert_parses(" -1.7e2 ", Number(-170.0));
        _assert_parses("\"hey there\"", String("hey there".into()));
        _assert_parses("[]", Array(vec![]));
        _assert_parses("{}", Object(vec![]));
    }

    #[test]
    fn test_arrays_simple() {
        _assert_parses(
            "[1, 2, \"\\u0075\", false, {}]",
            Array(vec![
                Number(1.0),
                Number(2.0),
                String("u".into()),
                Boolean(false),
                Object(vec![]),
            ]),
        );
    }

    #[test]
    fn test_arrays_trailing_comma() {
        _assert_fails("[1, 2, 3,]", 1, 9, "Unexpected ']'");
    }

    #[test]
    fn test_arrays_unclosed() {
        _assert_fails(
            "[1, 2, 3 false",
            1,
            9,
            "Expected ']', found boolean (false)",
        );
    }

    #[test]
    fn test_nested_arrays() {
        _assert_parses(
            "[ [ [ [ [ [ [ [ ] ] ] ] ] ] ] ]",
            Array(vec![Array(vec![Array(vec![Array(vec![Array(vec![
                Array(vec![Array(vec![Array(vec![])])]),
            ])])])])]),
        );
    }

    #[test]
    fn test_nested_arrays_mismatched() {
        _assert_fails("[[[[]]]", 1, 7, "Expected ']', found end-of-file");
        _assert_fails("[[[]]]]", 1, 6, "Expected end-of-file, found ']'");
    }

    #[test]
    fn test_objects_ok() {
        let json = r#"
        {
            "one": 1,
            "two" : [1, 2, 3],
            " other " : null ,
            "nested": {
                "one": 1
            }
        } "#;
        _assert_parses(
            json,
            Object(vec![
                ("one".into(), Number(1.0)),
                (
                    "two".into(),
                    Array(vec![Number(1.0), Number(2.0), Number(3.0)]),
                ),
                (" other ".into(), Null),
                ("nested".into(), Object(vec![("one".into(), Number(1.0))])),
            ]),
        );
    }

    #[test]
    fn test_objects_key_rules() {
        _assert_parses(
            // Keys are not case-sensitive
            r#"{"one": true, "ONE": false}"#,
            Object(vec![
                ("one".into(), Boolean(true)),
                ("ONE".into(), Boolean(false)),
            ]),
        );

        _assert_parses(
            // Non-ascii keys allowed
            r#"{"💩": true, "码": false}"#,
            Object(vec![
                ("💩".into(), Boolean(true)),
                ("码".into(), Boolean(false)),
            ]),
        );

        _assert_fails(
            // Duplicated keys at the same level not allowed
            r#"{"one": true, "one": false}"#,
            1,
            14,
            "Duplicated object key",
        );

        _assert_fails(
            // Test duplicated unicode-escaped keys
            r#"{"one": true, "\u006f\u006e\u0065": false}"#,
            1,
            14,
            "Duplicated object key",
        );

        _assert_fails(
            // Non-string keys not allowed
            r#"{1: "one", 2: "two"}"#,
            1,
            1,
            "Expected string, found number (1)",
        );
    }
}
