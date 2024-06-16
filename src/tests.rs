#[cfg(test)]
mod scanner_tests {
    use crate::scanner::Scanner;
    use crate::data::{JsonToken, TokenKind::*};

    fn _assert_token_sequence(src: &str, tokens: &[JsonToken]) {
        let mut scanner = Scanner::new(src);
        for token in tokens {
            assert_eq!(scanner.next_token().as_ref(), Ok(token));
        }
        assert!(matches!(scanner.next_token(), Ok(JsonToken { kind: Eof, .. })));
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
            JsonToken::new(Null, 1, 2),
            JsonToken::new(True, 2, 0),
            JsonToken::new(False, 2, 5),
            JsonToken::new(False, 3, 0),
            JsonToken::new(True, 3, 6),
            JsonToken::new(LeftBracket, 4, 0),
            JsonToken::new(RightBracket, 4, 2),
            JsonToken::new(LeftBrace, 4, 4),
            JsonToken::new(RightBrace, 4, 6),
            JsonToken::new(Colon, 4, 8),
            JsonToken::new(Comma, 4, 10),
            JsonToken::new(Number(0.0), 5, 0),
            JsonToken::new(Number(1.0), 5, 2),
            JsonToken::new(Number(2.0), 5, 4),
            JsonToken::new(Number(200.0), 5, 6),
            JsonToken::new(Number(-100.0), 5, 10),
            JsonToken::new(String(" abcde ".into()), 6, 0),
            JsonToken::new(String("123456".into()), 6, 11),
        ];

        _assert_token_sequence(s, &expected);
    }

    #[test]
    fn test_eofs() {
        // Check that the scanner provides constant EOFs after running out of tokens, without further advancing.
        let s = "null";
        let mut scanner = Scanner::new(s);
        assert_eq!(scanner.next_token(), Ok(JsonToken::new(Null, 1, 0)));
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
            JsonToken::new(Number(0.0), 2, 0),
            JsonToken::new(Number(1.0), 2, 2),
            JsonToken::new(Number(20.0), 2, 4),
            JsonToken::new(Number(300.0), 2, 7),
            JsonToken::new(Number(1.0), 2, 11),
            JsonToken::new(Number(-10.0), 2, 19),
            JsonToken::new(Number(-800.0), 2, 23),
            JsonToken::new(Number(-123.0), 2, 28),

            JsonToken::new(Number(0.0), 3, 0),
            JsonToken::new(Number(0.00001), 3, 4),
            JsonToken::new(Number(123.456), 3, 12),
            JsonToken::new(Number(-0.111), 3, 20),
            JsonToken::new(Number(-0.9), 3, 27),
            JsonToken::new(Number(-888.88), 3, 34),

            JsonToken::new(Number(0.0), 4, 0),
            JsonToken::new(Number(0.0), 4, 5),
            JsonToken::new(Number(10.0), 4, 10),
            JsonToken::new(Number(12300.0), 4, 14),
            JsonToken::new(Number(-2000000.0), 4, 25),
            JsonToken::new(Number(-11e12), 4, 31),
            JsonToken::new(Number(0.01), 4, 42),
            JsonToken::new(Number(-123e-10), 4, 48),

            JsonToken::new(Number(1.0), 5, 0),
            JsonToken::new(Number(1e97), 5, 7),
            JsonToken::new(Number(1.234), 5, 20),
            JsonToken::new(Number(-13.37e-8), 5, 29),
            JsonToken::new(Number(0.0), 5, 39),
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
        _assert_fails("0. ", 1, 2, "At least a digit is expected after a fraction dot");
        _assert_fails("-123.", 1, 5, "At least a digit is expected after a fraction dot");
        _assert_fails("1.e8", 1, 2, "At least a digit is expected after a fraction dot");
    }

    #[test]
    fn test_illegal_exponents() {
        _assert_fails("123e", 1, 4, "At least a digit is expected after an exponent");
        _assert_fails("-90EA", 1, 4, "At least a digit is expected after an exponent");
        _assert_fails("-90E+A", 1, 5, "At least a digit is expected after an exponent");
        _assert_fails("87.0e+1.2", 1, 7, "Unexpected character: '.'");
        _assert_fails("87.0e.2", 1, 5, "At least a digit is expected after an exponent");
    }
}
