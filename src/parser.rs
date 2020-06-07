use crate::Node;

pub enum Token {
    LParen,
    RParen,
    Bytes(Vec<u8>),
}

pub fn tokenize(s: &str) -> Result<Vec<Token>, String> {
    let mut char_iter = s.chars().into_iter().peekable();
    let mut tokens = vec!();

    while let Some(c) = char_iter.next() {
        match c {
            '(' => tokens.push(Token::LParen),
            ')' => tokens.push(Token::RParen),
            '0' => {
                if char_iter.next() != Some('x') {
                    return Err("Expected 'x'!".to_string());
                }
                
                let mut ret = vec!();
                let mut first_char_val = None;
                while let Some(c) = char_iter.peek() {
                    if let Some(d) = c.to_digit(16) {
                        // Append char/byte
                        if first_char_val.is_none() {
                            first_char_val = Some(d as u8)
                        } else {
                            ret.push(first_char_val.unwrap()*16+d as u8);
                            first_char_val = None;
                        }
                        char_iter.next();
                    } else if *c == '_' {
                        // ignore underscores
                        char_iter.next();
                    }else {
                        break;
                    }
                }
                if first_char_val.is_some() {
                    return Err("byte sequence incomplete".to_string());
                }
                tokens.push(Token::Bytes(ret));
            }
            c if c.is_ascii_whitespace() => {
                // ignore
            }
            _ => {
                return Err("Unexpected character!".to_string());
            }
        }
    }
    Ok(tokens)
}

pub fn parse(tokens: Vec<Token>) -> Result<Node, String> {
    let mut token_iter = tokens.into_iter().peekable();
    let res = parse_node(&mut token_iter)?;
    if let Some(_) = token_iter.next() {
        return Err("Unexpected characters after node.".to_string());
    }
    Ok(res)
}

type Tokens<'a> = std::iter::Peekable<std::vec::IntoIter<Token>>;


fn parse_node(data: &mut Tokens) -> Result<Node, String> {
    match data.next() {
        Some(Token::Bytes(b)) => Ok(Node::Leaf(b)),
        Some(Token::LParen) => parse_inner_node(data),
        _ => Err("Parse Error".to_string()),
    }
}

fn parse_inner_node(data: &mut Tokens) -> Result<Node, String> {
    let mut children = vec!();
    loop {
        if let Some(Token::RParen) = data.peek() {
            data.next();
            break;
        }
        children.push(parse_node(data)?);
    }
    Ok(Node::Inner(children))
}

pub enum ParseResult {
    LexingError(String),
    ParsingError(String),
    Ok(Node)
}

impl ParseResult {
    pub fn is_lexing_ok(&self) -> bool {
        match self {
            ParseResult::LexingError(_) => false,
            _ => true,
        }
    }
    
    pub fn is_ok(&self) -> bool {
        match self {
            ParseResult::Ok(_) => true,
            _ => false,
        }
    }

    pub fn err_message(&self) -> &str {
        match self {
            ParseResult::Ok(_) => "",
            ParseResult::LexingError(s) => &s,
            ParseResult::ParsingError(s) => &s,
        }
    }

    pub fn parse(s: &str) -> ParseResult {
        match tokenize(s) {
            Ok(tokens) => {
                match parse(tokens) {
                    Ok(node) => ParseResult::Ok(node),
                    Err(s) => ParseResult::ParsingError(s)
                }
            },
            Err(s) => ParseResult::LexingError(s)
        }
    }
}