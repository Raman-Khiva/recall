mod agent;
use bat::{Input, PrettyPrinter, WrappingMode};

use clap::{Parser, Subcommand};
use rusqlite::Connection;
use serde_json::json;
use std::error::Error;
use std::io::{self, BufRead};
use terminal_size::{Width, terminal_size};
use textwrap::Options;

use crate::agent::{Message, SnippetMeta, SnippetTitle, agent};

#[derive(Debug)]
#[allow(dead_code)]
struct Person {
    id: i32,
    name: String,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Snippet {
    title: String,
    summary: String,
    content: String,
}

#[derive(Parser)]
#[command(
    name = "rc",
    version,
    about = "a powerful lightweight quick captue and rapid retrieval engine"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Save {
        #[arg(value_name = "CONTENT")]
        content: Vec<String>,
    },
    Get {
        #[arg(value_name = "QUERY")]
        query: Vec<String>,
    },
}

// get function
async fn get(
    // get params
    query: &String,
    conn: &Connection,
) -> Result<(), Box<dyn Error>> {
    let mut stmt = conn.prepare("SELECT title, summary FROM snippets;")?;

    let rows = stmt.query_map([], |row| {
        Ok(SnippetMeta {
            title: row.get(0)?,
            summary: row.get(1)?,
        })
    })?;

    let meta_list: Vec<SnippetMeta> =
        rows.collect::<Result<Vec<SnippetMeta>, rusqlite::Error>>()?;

    let context = serde_json::to_string_pretty(&meta_list);

    let messages: Vec<Message> = vec![
            Message{role : "system".to_string(), content : "You are a brain of a quick snippet saving and finding program. you have given the user query and meta data of availbe snippets, your job is to select most appropirate snippet matching the user query and return it's title".to_string(),
        },
        Message{
            role:"user".to_string(),
            content: format!(
                "Snippets Meta: \n {:?} \n \n , find snippets relevent to : {}",
                 context, query
            ),
        }
        ];
    let required_props: Vec<String> = vec!["title".to_string()];

    let response_schema: serde_json::Value = json!({
        "title":{"type":"string"}
    });

    let content: String = agent(&messages, response_schema, &required_props).await?;

    let snippet_title: SnippetTitle = serde_json::from_str(&content.to_string())?;

    let snippet: Snippet = conn.query_row(
        "SELECT * FROM snippets Where title=$1",
        [snippet_title.title],
        |row| {
            Ok(Snippet {
                title: row.get(0)?,
                summary: row.get(1)?,
                content: row.get(2)?,
            })
        },
    )?;

    display_content(&snippet.content)?;

    Ok(())

    //get end get function end
}

// display content function
fn display_content(content: &str) -> Result<(), Box<dyn Error>> {
    let terminal_width = terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(80);

    let padding = 4;
    let content_width = terminal_width.saturating_sub(padding * 4);

    let gutter = 4;
    let wrapped = textwrap::fill(
        content,
        Options::new(content_width.saturating_sub(gutter)).break_words(false),
    );

    PrettyPrinter::new()
        .input(Input::from_bytes(wrapped.as_bytes()))
        .language("markdown")
        .grid(true)
        .line_numbers(true)
        .term_width(terminal_width)
        .wrapping_mode(WrappingMode::Character)
        .print()?;
    Ok(())
}

// save function
async fn save(
    // save params
    snippet: &String,
    conn: &Connection,
) -> Result<(), Box<dyn Error>> {
    // messages save

    // response save
    let messages: Vec<Message> = vec![
        Message{role : "system".to_string(), content :"You are a brain of a quick snippet saving and finding program. you have given the snippet user wants to save your job is to generate appropriate title and a short summary which will be used to semantically find the snippet later.".to_string(),
    },
    Message{
        role:"user".to_string(),
        content: snippet.clone(),    }
    ];
    let required_props: Vec<String> = vec!["title".to_string(), "summary".to_string()];

    let response_schema: serde_json::Value = json!({
        "title":{"type":"string"},
        "summary":{"type":"string"}
    });

    let content: String = agent(&messages, response_schema, &required_props).await?;

    let meta: SnippetMeta = serde_json::from_str(&content.as_str())?;

    // save to database
    let snippet_entry: Snippet = Snippet {
        title: meta.title,
        summary: meta.summary,
        content: snippet.to_string(),
    };

    conn.execute(
        "INSERT INTO snippets (title, summary, content) VALUES (?1, ?2, ?3)",
        rusqlite::params![
            snippet_entry.title,
            snippet_entry.summary,
            snippet_entry.content,
        ],
    )?;

    println!("\x1b[32m✓\x1b[0m Saved successfully!");
    Ok(())
    //save end
}

#[allow(dead_code)]
fn invalid_query() {
    println!("Plese put valid query");
    return;
}

// multiline read fucntion
fn read_multiline() -> io::Result<String> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    let mut content = String::new();
    let mut line = String::new();
    println!("Enter your content (:end to finish):");
    loop {
        line.clear();

        if reader.read_line(&mut line)? == 0 {
            break;
        }

        if line.trim_end_matches(['\r', '\n']) == ":end" {
            break;
        }

        content.push_str(&line);
    }

    Ok(content)
}

fn db_init() -> Result<Connection, Box<dyn Error>> {
    let data_dir = dirs::data_local_dir()
        .expect("Could not determine local data directory! (hint: ~/.local/share)")
        .join("recall");

    std::fs::create_dir_all(&data_dir)?;

    let db_path = data_dir.join("recall.db");

    let conn = rusqlite::Connection::open(db_path)?;

    Ok(conn)
}

#[tokio::main]
#[allow(dead_code)]
async fn main() -> Result<(), Box<dyn Error>> {
    // intialize db
    let conn = db_init()?;

    // inititalize cli
    let cli = Cli::parse();

    match cli.command {
        Commands::Save { content } => {
            let content = if content.is_empty() {
                let input = read_multiline()?;
                input
            } else {
                content.join(" ")
            };

            save(&content, &conn).await?;
        }

        Commands::Get { query } => {
            let query = if query.is_empty() {
                println!("Enter your query here");
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                input
            } else {
                query.join(" ")
            };

            get(&query, &conn).await?;
        }
    }

    Ok(())
}
