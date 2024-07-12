use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

use chrono::prelude::*;
use html::content::children::MainChild;
use html::content::{Article, Footer, Main};
use html::root::children::BodyChild;
use html::root::Html;
use html::text_content::{Division, Figure, ListItem, UnorderedList};
use html::{content::Header, forms::Forms};
use htmlentity::entity::{encode, CharacterSet, EncodeType, ICodedDataTrait};
use rusqlite::Connection;
use rand::Rng;

pub fn main_page<'a, T: Iterator<Item = &'a Post>>(posts: T) -> String {
    let mut c: Vec<BodyChild> = Vec::new();

    c.push(
        Main::builder()
            .heading_1(|h1| h1.text("Fils de discussions"))
            .anchor(|a| a.href("#post").text("Créer un fil de discussion"))
            .push(thread_list(posts))
            .heading_2(|h2| h2.text("Créer un fil de discussion"))
            .push(post_form("/".to_owned()))
            .build()
            .into(),
    );

    base_template(c).to_string()
}