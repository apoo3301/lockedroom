use nom::{bytes::complete::{tag, take_until, take_while1},combinator::recognize,sequence::{terminated,tuple},IResult};
use rusqlite::Connection;
use std::str;

use crate::database::{get_post_by_id, Post};

fn is_urs_char(c: char) -> bool {
    match c {
        '.' | '/' | ':' | '-' | '_' | '?' | '#' | '=' => true,
        _ => c.is_alphanumeric(),
    }
}

fn url_parser(input: &str) -> IResult<&str, &str> {
    recognize(tuple((
        tag("https://"),
        take_while1(is_urs_char),
    )))(input)?;

    Ok(("", input))
}

fn is_digit(c: char) -> bool {
    c.is_digit(10)
}

fn mention_parser(input: &str) -> IResult<&str, &str> {
    let (input, mention) = recognize(tuple((tag("#"), take_while1(is_digit))))(input)?;
    Ok((input, mention))
}

fn bold_parser(input: &str) -> IResult<&str, &str> {
    let (input, bold) = recognize(tuple((tag("**"), take_until("**"))))(input)?;
    Ok((input, bold))
}

fn quote_parser(input: &str) -> IResult<&str, &str> {
    let (input, quote) = recognize(tuple((tag(">"), take_until("\n"))))(input)?;
    Ok((input, quote))
}

fn paragraph_parser(input: &str) -> IResult<&str, &str> {
    let (input, paragraph) = recognize(take_until("\n\n"))(input)?;
    Ok((input, paragraph))
}

