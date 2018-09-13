use error::{ParseError, InvalidRule};

use regex::Regex;

#[derive(Debug, PartialEq)]
pub enum Command {
    Capitalize,
    Lowercase,
    Titlecase,
    IndefiniteArticle,
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Literal(String),
    NonTerminal(String),
    StickyNonTerminal(String),
    Binding(String),
    Expression(Command, Box<Token>),
    Range(usize, usize),
    VariableAssignment(String, String),
}


/// Make a Token::Literal from a string slice.
fn make_literal(expr: &str, beg: usize, end: usize) -> Token {
    let lit = expr.get(beg..end).unwrap();
    Token::Literal(lit.replace("\\<", "<").replace("\\>", ">").into())
}


///
fn parse_cmd_expr(expr: &str, beg: usize, end: usize) -> Result<Token, ParseError> {
    if expr.len() == 2 {
        return Err(ParseError::ZeroLengthSubst(beg + 1, end));
    }
    let expr = &expr[1..expr.len() - 1];
    let (cmd_str, tok_str) = match expr.find(" ") {
        Some(first_space) => expr.split_at(first_space),
        None => return Err(ParseError::InvalidExpression(beg, end))
    };
    let cmd = match cmd_str {
        "cap" | "capitalize" => Command::Capitalize,
        "low" | "lowercase" => Command::Lowercase,
        "title" | "titlecase" => Command::Titlecase,
        "a" | "an" => Command::IndefiniteArticle,
        _ => return Err(ParseError::UnknownCommand(beg, beg + cmd_str.len()))
    };
    let tok = validate_substitution_expr(tok_str.trim(), beg + cmd_str.len(), end)?;
    Ok(Token::Expression(cmd, Box::new(tok)))
}

/// Parse a range expression.
fn parse_range(expr: &str, beg: usize, end: usize) -> Result<Token, ParseError> {
    if let Some(index) = expr.find("-") {
        let (l_str, u_str) = expr.split_at(index);
        let lower = l_str.parse::<usize>();
        let upper = u_str[1..].parse::<usize>();
        if !lower.is_ok() || !upper.is_ok() {
            return Err(ParseError::InvalidRange(beg, end));
        }
        return Ok(Token::Range(lower.unwrap(), upper.unwrap()));
    }
    Err(ParseError::InvalidRange(beg, end))
}


fn parse_variable(expr: &str, _beg: usize, _end: usize) -> Result<Token, ParseError> {
    if let Some(index) = expr.find(":") {
        let (vname, ntname) = expr.split_at(index);
        Ok(Token::VariableAssignment(vname.to_string(), ntname.to_string()))
    } else {
        Err(ParseError::InternalError)
    }
}

/// Validate and create a Token from an expression string.
fn validate_substitution_expr(expr: &str, beg: usize, end: usize) -> Result<Token, ParseError> {
    lazy_static! {
        static ref VALIDATE_NAME: Regex = Regex::new(r##"^[@!$#]?[\w0-9_-]+$"##).unwrap();
    }
    let initial = &expr[0..1];
    match initial {
        "(" => {
            if expr.matches("(").count() != expr.matches(")").count() {
                return Err(ParseError::InvalidExpression(beg, end));
            }
            parse_cmd_expr(expr, beg, end)
        },
        _ => {
            if !VALIDATE_NAME.is_match(expr) {
                return Err(ParseError::InvalidName(beg, end));
            }
            match initial {
                "@" => Ok(Token::Binding(expr[1..].into())),
                "!" => Ok(Token::StickyNonTerminal(expr[1..].into())),
                "#" => parse_range(&expr[1..], beg, end),
                "$" => parse_variable(&expr[1..], beg, end),
                _ => Ok(Token::NonTerminal(expr.into())),
            }
        }
    }
}

/// Make a Token::NonTerminal from a string slice.
fn make_subst(expr: &str, beg: usize, end: usize) -> Result<Token, ParseError> {
    if beg == end {
        return Err(ParseError::ZeroLengthSubst(beg, end));
    }
    match expr.get(beg..end) {
        Some(snip) => validate_substitution_expr(snip, beg, end),
        None => Err(ParseError::InternalError)
    }
}

/// Transform a string into a Vector of Tokens.
pub fn make_expr(expr: &str) -> Result<Vec<Token>, ParseError> {
    let mut exprs : Vec<Token> = vec![];

    let mut in_subst = false;
    let mut cbeg = 0;

    let lbrackets = expr.matches("<").count() - expr.matches("\\<").count();
    let rbrackets = expr.matches(">").count() - expr.matches("\\>").count();

    if lbrackets != rbrackets {
        return Err(ParseError::UnbalancedBrackets);
    }

    let mut prev_glyph = ' ';

    for (index, glyph) in expr.chars().enumerate() {
        match glyph {
            '<' => {
                if in_subst || prev_glyph == '\\' {
                    continue;
                }
                if cbeg != index {
                    exprs.push(make_literal(&expr, cbeg, index));
                }
                cbeg = index + 1;
                in_subst = true;
            }
            '>' => {
                if prev_glyph == '\\' {
                    continue;
                }
                exprs.push(make_subst(&expr, cbeg, index)?);
                cbeg = index + 1;
                in_subst = false;
            }
            _ => ()
        }
        prev_glyph = glyph;
    }
    if cbeg < expr.len() {
        // check for unterminated subst here
        exprs.push(make_literal(&expr, cbeg, expr.len()));
    }
    Ok(exprs)
}

pub fn parse(expr: &str) -> Result<Vec<Token>, InvalidRule> {
    match make_expr(expr) {
        Ok(tokens) => Ok(tokens),
        Err(error) => Err(InvalidRule::from_parse_error(expr, error))
    }
}


#[cfg(test)]
mod test {
    use super::*;

    macro_rules! should_fail_with {
        ($input: expr, $err_type: pat) => {
            match make_expr($input) {
                Err($err_type) => (),
                Err(other) => assert!(false, "Unexpected error: {}", other),
                Ok(_) => assert!(false, "Got Ok(_) instead of Err(_)!")
            }
        }
    }

    macro_rules! evaluates_to {
        ($input: expr, $tokens: expr) => {
            for (lhs, rhs) in make_expr($input).unwrap().iter().zip($tokens.iter()) {
                assert_eq!(lhs, rhs);
            }
        }
    }

    #[test]
    fn test_expr() {
        evaluates_to!("This is an expression with one <substitution> symbol.",
                      [
                           Token::Literal("This is an expression with one ".into()),
                           Token::NonTerminal("substitution".into()),
                           Token::Literal(" symbol.".into())
                      ]);

        evaluates_to!("This is <(cap one)> a complex expression with <!sticky> and <@bind>!",
                      [
                            Token::Literal("This is ".into()),
                            Token::Expression(Command::Capitalize,
                                              Box::new(Token::NonTerminal("one".into()))),
                            Token::Literal(" a complex expression with ".into()),
                            Token::StickyNonTerminal("sticky".into()),
                            Token::Literal(" and ".into()),
                            Token::Binding("bind".into()),
                            Token::Literal("!".into()),
                      ]);
    }


    #[test]
    fn test_escaped_subst() {
        let exprs = make_expr("Expr with \\<escaped brackets\\>").unwrap();
        assert_eq!(exprs[0], Token::Literal("Expr with <escaped brackets>".into()));

        let exprs = make_expr("Expr with \\<\\< unbalanced escaped brackets\\>").unwrap();
        assert_eq!(exprs[0], Token::Literal("Expr with << unbalanced escaped brackets>".into()));
    }

    #[test]
    fn test_nonterminal() {
        let exprs = make_expr("<aa>").unwrap();
        assert_eq!(exprs[0], Token::NonTerminal("aa".to_string()));

        evaluates_to!("<aa><bb><cc>",
                      [
                           Token::NonTerminal("aa".to_string()),
                           Token::NonTerminal("bb".to_string()),
                           Token::NonTerminal("cc".to_string())
                      ]);

        evaluates_to!("<aa> <cc>",
                      [
                           Token::NonTerminal("aa".to_string()),
                           Token::Literal(" ".to_string()),
                           Token::NonTerminal("cc".to_string())
                      ]);
    }

    #[test]
    fn test_sticky_nonterminal() {
        evaluates_to!("<!foo>", [Token::StickyNonTerminal("foo".to_string())]);
        evaluates_to!("This is a <!sticky> nonterminal.",
                      [
                           Token::Literal("This is a ".to_string()),
                           Token::StickyNonTerminal("sticky".to_string()),
                           Token::Literal(" nonterminal.".to_string())
                      ]);
    }

    #[test]
    fn test_binding() {
        let exprs = make_expr("<@ab>").unwrap();
        assert_eq!(exprs[0], Token::Binding("ab".to_string()));

        let exprs = make_expr("<@bind1>").unwrap();
        assert_eq!(exprs[0], Token::Binding("bind1".to_string()));

        let exprs = make_expr("<@beautiful_snake>").unwrap();
        assert_eq!(exprs[0], Token::Binding("beautiful_snake".to_string()));
    }

    #[test]
    fn test_expression() {
        evaluates_to!("<(an animal)>",
                      [Token::Expression(Command::IndefiniteArticle,
                                         Box::new(Token::NonTerminal("animal".to_string())))]);
    }

    #[test]
    fn test_nested_expression() {
        let inner = Box::new(
            Token::Expression(Command::Capitalize,
                              Box::new(Token::NonTerminal("animal".to_string())))
        );
        evaluates_to!("<(an (cap animal))>",
                      [Token::Expression(Command::IndefiniteArticle, inner)]);

        // this would currently fail with 'invalid name'
        // make_expr("<(an (an (an (an animal)))))>").unwrap();
    }

    #[test]
    fn test_range() {
        evaluates_to!("<#39-100>", Token::Range(39, 100));
    }


    #[test]
    fn test_err_empty_token() {
        should_fail_with!("<>", ParseError::ZeroLengthSubst(1, 2));
        should_fail_with!("<@>", ParseError::InvalidName(1, 2));
        should_fail_with!("<!>", ParseError::InvalidName(1, 2));
        should_fail_with!("<()>", ParseError::ZeroLengthSubst(2, 3));
    }

    #[test]
    fn test_err_zero_length() {
        should_fail_with!("Zero-length <>", ParseError::ZeroLengthSubst(13, 14));
        should_fail_with!("Zero-length <> unaffected by post-string",
                          ParseError::ZeroLengthSubst(13, 14));
    }

    #[test]
    fn test_bad_binding() {
        should_fail_with!("<@>", ParseError::InvalidName(1, 2));
        should_fail_with!("<@some binding>", ParseError::InvalidName(1, 14));
        should_fail_with!("<@ binding>", ParseError::InvalidName(1, 10));
    }

    #[test]
    fn test_unbalanced() {
        should_fail_with!("<<", ParseError::UnbalancedBrackets);
        should_fail_with!("<>>", ParseError::UnbalancedBrackets);
        should_fail_with!("This is an <<unbalanced> expression", ParseError::UnbalancedBrackets)
    }
}

