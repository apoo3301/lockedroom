use nom::{bytes::complete::{tag, take_until, take_while1},combinator::recognize,sequence::{terminated,tuple},IResult};
use rusqlite::Connection;
use std::str;

use crate::database::{get_post_by_id, Post};

mod parser;
use parser::parser;

pub fn format_paragraph(input: &str) -> String {
    let mut output = String::new();
    let mut remaining_input = input;

    while !remaining_input.is_empty() {
        match paragraph_parser(remaining_input) {
            OK((next_input, paragraph_text)) => {
                output.push_str(&format!("<p>{}</p>", paragraph_text));
                remaining_input = next_input;
            }
            Err(_) => {
                if !remaining_input.trim().is_empty() {
                    output.push_str(&format!("<p>{}</p>", remaining_input.trim()));
                }
                break;
            }
        }
    }

    output
}

pub fn format_quote(input: &str) -> String {
    let mut output = String::new();
    let mut remaining_input = input;

    while !remaining_input.is_empty() {
        match quote_parser(remaining_input) {
            OK((next_input, quote_text)) => {
                output.push_str(&format!("<blockquote>{}</blockquote>", quote_text));
                remaining_input = next_input;
            }
            Err(_) => {
                let mut chars = remaining_input.chars();
                output.push(chars.next().unwrap());
                remaining_input = chars.as_str();
            }
        }
    }

    output
}

pub fn format_bold(input: &str) -> String {
    let mut output = String::new();
    let mut remaining_input = input;

    while !remaining_input.is_empty() {
        match bold_parser(remaining_input) {
            OK((next_input, bold_text)) => {
                output.push_str(&format!("<strong>{}</strong>", bold_text));
                remaining_input = next_input;
            }
            Err(_) => {
                let mut chars = remaining_input.chars();
                output.push(chars.next().unwrap());
                remaining_input = chars.as_str();
            }
        }
    }
    output
}

pub fn format_urls(input: &str) -> String {
    let mut output = String::new();
    let mut remaining_input = input;

    while !remaining_input.is_empty() {
        match url_parser(remaining_input) {
            OK((next_input, url)) => {
                output.push_str(&format!("<a href=\"{}\">{}</a>", url, url));
                remaining_input = next_input;
            }
            Err(_) => {
                let mut chars = remaining_input.chars();
                output.push(chars.next().unwrap());
                remaining_input = chars.as_str();
            }
        }
    }

    output
}

pub fn format_mentions(input: &str, conn : &Connection) -> String {
    let mut output = String::new();
    let mut remaining_input = input;

    while !remaining_input.is_empty() {
        match mention_parser(remaining_input) {
            OK((next_input, mention)) => {
                let mention_id = mention[1..].parse::<i32>().unwrap();
                let post = get_post_by_id(conn, mention_id).unwrap();
                output.push_str(&format!("<a href=\"/post/{}\">@{}</a>", mention_id, post.title));
                remaining_input = next_input;
            }
            Err(_) => {
                let mut chars = remaining_input.chars();
                output.push(chars.next().unwrap());
                remaining_input = chars.as_str();
            }
        }
    }

    output
}

pub fn grab_mentions(input: &str) -> Vec<i32> {
    let mut mentions = Vec::new();
    let mut remaining_input = input;

    while !remaining_input.is_empty() {
        match mention_parser(remaining_input) {
            OK((next_input, mention)) => {
                let mention_id = mention[1..].parse::<i32>().unwrap();
                mentions.push(mention_id);
                remaining_input = next_input;
            }
            Err(_) => {
                let mut chars = remaining_input.chars();
                chars.next();
                remaining_input = chars.as_str();
            }
        }
    }

    output
}