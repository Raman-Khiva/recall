mod agent;
use bat::{Input, PrettyPrinter, WrappingMode};

use clap::{Parser, Subcommand};
use groqai::{ChatMessage, GroqClient, MessageContent, ResponseFormat, Role};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;
use std::io::{self, BufRead};
use terminal_size::{Width, terminal_size};
use textwrap::Options;

use crate::agent::{Message, SnippetTitle, agent};

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

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
struct SnippetMeta {
    title: String,
    summary: String,
}

// #[derive(Debug, serde::Deserialize)]
// #[allow(dead_code)]
// struct SnippetTitle {
//     title: String,
// }

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

// #[allow(dead_code)]
// async fn agent() -> Result<(), Box<dyn Error>> {
//     let client = GroqClient::new()?;

//     let messages = vec![ChatMessage::new_text(
//         Role::User,
//         "Explain rust language in simple language and as simple text not json string",
//     )];

//     let response = client
//         .chat("openai/gpt-oss-20b")
//         .messages(messages)
//         .send()
//         .await?;

//     if let Some(choice) = response.choices.first() {
//         println!("Response: {:?}", choice.message.content);
//     }

//     Ok(())
// }

// get function
async fn get(
    // get params
    query: &String,
    client: &GroqClient,
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

    // for meta in &meta_list {
    //     println!("{:?}", meta);
    // }

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
    // let messages = vec![
    //     ChatMessage::new_text(
    //         Role::System,
    //         "You are a brain of a quick snippet saving and finding program. you have given the user query and meta data of availbe snippets, your job is to select most appropirate snippet matching the user query and return it's title",
    //     ),
    //     ChatMessage::new_text(
    //         Role::User,
    //         format!(
    //             "Snippets Meta: \n {:?} \n \n , find snippets relevent to : {}",
    //             context, query
    //         ),
    //     ),
    // ];

    // get response
    // let response = client
    //     .chat("openai/gpt-oss-120b")
    //     .messages(messages)
    //     .response_format(ResponseFormat {
    //         type_: "json_schema".to_string(),
    //         json_schema: Some(json!({
    //             "name" : "snippet_title",
    //             "strcit" : true,
    //             "schema" : {
    //                 "type" : "object",
    //                 "properties":{
    //                     "title":{"type":"string"}
    //                 },
    //                 "required":["title"],
    //                 "additionalProperties": false,
    //             }
    //         })),
    //     })
    //     .send()
    //     .await?;
    //
    let required_props: Vec<String> = vec!["title".to_string()];

    let response_schema: serde_json::Value = json!({
        "title":{"type":"string"}
    });

    let content: String = agent(&messages, response_schema, &required_props).await?;
    // &response.choices[0].message.content;
    // let raw_json = match content {
    //     MessageContent::Text(s) => s.as_str(),
    //     _ => return Err("Expected text from groq but not found".into()),
    // };

    let snippet_title: SnippetTitle = serde_json::from_str(&content.to_string())?;

    // println!(
    //     "target title given by groqai! title: {}",
    //     snippet_title.title
    // );

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
    // println!("{}", snippet.content);

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

    // let options = Options::new(content_width).break_words(false);

    // let line = "─".repeat(content_width);

    // println!();
    // println!("{}\x1b[32m{}\x1b[0m", " ".repeat(padding), line);
    // println!();

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
    // PrettyPrinter::new()
    //        .input(Input::from_bytes(content.as_bytes()))
    //        .language("markdown")
    //        .grid(true)
    //        .line_numbers(true)
    //        .term_width(terminal_width)
    //        .wrapping_mode(WrappingMode::Character)
    //        .print();

    // for paragraph in content.split("\n\n") {
    //     if paragraph.trim().is_empty() {
    //         println!();
    //         continue;
    //     }

    //     for line in wrap(paragraph.trim(), &options) {
    //         // println!("{}{}", " ".repeat(padding * 2), line);
    //         PrettyPrinter::new()
    //             .input(Input::from_bytes(line.as_bytes()))
    //             .language("markdown")
    //             .grid(true)
    //             .line_numbers(true)
    //             .term_width(terminal_width)
    //             .wrapping_mode(WrappingMode::Character)
    //             .print();
    //         // PrettyPrinter::new()
    //         //     .input(Input::from_bytes(line.as_bytes()).title(title))
    //         //     .language(lang)
    //         //     .header(true)
    //         //     .grid(true)
    //         //     .line_numbers(false)
    //         //     .term_width(term_width.min(100))
    //         //     .wrapping_mode(WrappingMode::Character)
    //         //     .print();
    //     }

    //     println!();
    // }

    // println!();
    // println!("{}\x1b[32m{}\x1b[0m", " ".repeat(padding), line);
    // println!();
    //
    Ok(())
}

// save function
async fn save(
    // save params
    snippet: &String,
    client: &GroqClient,
    conn: &Connection,
) -> Result<(), Box<dyn Error>> {
    // messages save
    let messages = vec![
        ChatMessage::new_text(
            Role::System,
            "You are a brain of a quick snippet saving and finding program. you have given the snippet user wants to save your job is to generate appropriate title and a short summary which will be used to semantically find the snippet later.",
        ),
        ChatMessage::new_text(Role::User, snippet.clone()),
    ];

    // response save
    let response = client
        .chat("openai/gpt-oss-20b")
        .messages(messages)
        .response_format(ResponseFormat {
            type_: "json_schema".to_string(),
            json_schema: Some(json!({
                "name": "snippet_metadata",
                "strict": true,
                "schema": {
                    "type": "object",
                    "properties": {
                        "title": { "type": "string" },
                        "summary": { "type": "string" }
                    },
                    "required": ["title", "summary"],
                    "additionalProperties": false
                }
            })),
        })
        .send()
        .await?;

    let content = &response.choices[0].message.content;

    let raw_json = match content {
        MessageContent::Text(s) => s.as_str(),
        _ => return Err("Expected text response from Groq".into()),
    };

    let meta: SnippetMeta = serde_json::from_str(raw_json)?;

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
    // println!("generated content {:?}", snippet_entry);
    // println!(
    //     "Generated  \n title: {}  \n summary: {}",
    //     content.title, content.summary
    // );
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
    // println!("Hello, world!");
    //

    let cli = Cli::parse();

    // let client = GroqClient::new()?;
    // create conn
    let conn = db_init()?;

    let client = GroqClient::new()?;

    //create table snippets
    // conn.execute(
    //     "CREATE TABLE snippets (
    //     title TEXT NOT NULL UNIQUE,
    //     summary TEXT NOT NULL,
    //     content TEXT NOT NULL
    //     );",
    //     (),
    // )?;

    match cli.command {
        Commands::Save { content } => {
            let content = if content.is_empty() {
                let input = read_multiline()?;
                input
            } else {
                content.join(" ")
            };

            save(&content, &client, &conn).await?;
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

            get(&query, &client, &conn).await?;
        }
    }
    let snippet_1 = r#"================================================================================
    UBUNTU CADDY REVERSE PROXY & AUTOMATIC HTTPS RUNBOOK
    ================================================================================

    1. INSTALL CADDY ON UBUNTU
    --------------------------------------------------------------------------------
      sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https curl
      curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
      curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list
      sudo apt update && sudo apt install -y caddy

    2. SAMPLE /etc/caddy/Caddyfile FOR LOCAL CONTAINER / APP PORT
    --------------------------------------------------------------------------------
      api.example.com {
          reverse_proxy 127.0.0.1:8080
      }

    3. VALIDATE, RELOAD & CHECK LOGS
    --------------------------------------------------------------------------------
      caddy validate --config /etc/caddy/Caddyfile
      sudo systemctl reload caddy
      journalctl -u caddy -f --no-pager
    ================================================================================
    "#
        .to_string();

    let snippet_2 =
        r#"================================================================================
    DOCKER & COMPOSE CLEANUP / DEBUGGING CHEAT SHEET
    ================================================================================

    1. INSPECT RUNNING CONTAINERS & LIVE LOGS
    --------------------------------------------------------------------------------
      docker ps --format "table {{.Names}}\t{{.Image}}\t{{.Status}}\t{{.Ports}}"
      docker logs -f --tail 100 <container_name>

    2. SHELL INTO A CONTAINER
    --------------------------------------------------------------------------------
      docker exec -it <container_name> sh

    3. RECLAIM DISK SPACE SAFELY
    --------------------------------------------------------------------------------
      docker system df
      docker image prune -f
      docker builder prune -f
      # Warning: removes unused volumes too:
      docker volume prune -f
    ================================================================================
    "#
        .to_string();

    let snippet_3 =
        r#"================================================================================
    POSTGRESQL & REDIS QUICK CONNECTION / INSPECTION COMMANDS
    ================================================================================

    1. POSTGRESQL VIA PSQL
    --------------------------------------------------------------------------------
      psql "$DATABASE_URL"
      \dt
      \d+ users
      SELECT pid, state, query_start, left(query, 80) FROM pg_stat_activity WHERE state != 'idle';

    2. REDIS VIA REDIS-CLI
    --------------------------------------------------------------------------------
      redis-cli -u "$REDIS_URL" ping
      redis-cli -u "$REDIS_URL" --scan --pattern "cache:*" | head -n 30
      redis-cli -u "$REDIS_URL" info memory | grep -E "used_memory_human|maxmemory_human"
    ================================================================================
    "#
        .to_string();

    let snippet_4 =
        r#"================================================================================
    SYSTEMD SERVICE & PORT / SOCKET TROUBLESHOOTING
    ================================================================================

    1. CHECK SERVICE STATUS & RECENT ERRORS
    --------------------------------------------------------------------------------
      systemctl status <service_name> --no-pager
      journalctl -u <service_name> -n 80 --no-pager

    2. FIND WHICH PROCESS OWNS A PORT
    --------------------------------------------------------------------------------
      ss -tulpn | grep :3000
      lsof -i :3000

    3. RESTART AFTER UNIT / ENV EDIT
    --------------------------------------------------------------------------------
      sudo systemctl daemon-reload
      sudo systemctl restart <service_name>
    ================================================================================
    "#
        .to_string();

    let snippet_5 =
        r#"================================================================================
    SSH KEY SETUP & REMOTE LINUX FILE SYNC (RSYNC)
    ================================================================================

    1. GENERATE ED25519 KEY & COPY TO REMOTE HOST
    --------------------------------------------------------------------------------
      ssh-keygen -t ed25519 -C "dev-machine"
      ssh-copy-id user@remote-host

    2. SYNC PROJECT ARTIFACTS EXCLUDING BUILD/CACHE DIRS
    --------------------------------------------------------------------------------
      rsync -avz --progress \
        --exclude 'target/' \
        --exclude 'node_modules/' \
        --exclude '.git/' \
        ./ user@remote-host:~/app/
    ================================================================================
    "#
        .to_string();

    let _snippets: Vec<String> = vec![snippet_1, snippet_2, snippet_3, snippet_4, snippet_5];

    //     // Direct index access (panics if index is out of bounds)
    //     println!("{}", snippets[0]);

    //     // Safe index access (returns Option<&String>)
    //     if let Some(item) = snippets.get(3) {
    //         println!("{item}");
    //     }
    // }
    // let snippet  = r#"================================================================================
    // RUST SETUP GUIDE FOR UBUNTU (WITH CLI / TUI / CLIPBOARD DEPENDENCIES)
    // ================================================================================

    // 1. UPDATE PACKAGES & INSTALL BUILD TOOLS
    // --------------------------------------------------------------------------------
    // Rust needs a C linker (gcc/build-essential) and a few native libraries so crates
    // like reqwest (HTTPS), rusqlite (SQLite), and arboard (clipboard) compile cleanly.

    // Run:
    //   sudo apt update && sudo apt upgrade -y
    //   sudo apt install -y curl build-essential pkg-config libssl-dev libsqlite3-dev xclip wl-clipboard

    // Note:
    // - xclip is used for X11 clipboard support.
    // - wl-clipboard provides wl-copy/wl-paste for Wayland sessions.

    // 2. INSTALL RUST USING RUSTUP (OFFICIAL TOOLCHAIN INSTALLER)
    // --------------------------------------------------------------------------------
    // Run the official rustup installer:
    //   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

    // When prompted:
    //   Press 1 (Proceed with standard installation) and hit Enter.

    // 3. LOAD RUST INTO YOUR CURRENT SHELL
    // --------------------------------------------------------------------------------
    // Either restart your terminal, or run:
    //   source "$HOME/.cargo/env"

    // 4. VERIFY INSTALLATION
    // --------------------------------------------------------------------------------
    // Run:
    //   rustc --version
    //   cargo --version

    // You should see version numbers printed for both rustc (compiler) and cargo
    // (package manager & build tool).

    // 5. CREATE & TEST YOUR PROJECT ("agent")
    // --------------------------------------------------------------------------------
    // Run:
    //   cargo new agent
    //   cd agent
    //   cargo run

    // If it prints "Hello, world!", your Rust environment is ready.

    // 6. USEFUL CARGO COMMANDS FOR TONIGHT
    // --------------------------------------------------------------------------------
    //   cargo check             # Fast syntax & type check without building a binary
    //   cargo build             # Build debug binary (target/debug/agent)
    //   cargo build --release   # Build fast optimized binary (target/release/agent)
    //   cargo add <crate>       # Add a dependency to Cargo.toml

    // To make your compiled binary runnable anywhere as `agent`:
    //   sudo cp target/release/agent /usr/local/bin/agent
    //   # OR symlink without sudo if ~/.local/bin is in your PATH:
    //   mkdir -p ~/.local/bin && ln -sf "$(pwd)/target/release/agent" ~/.local/bin/agent

    // 7. UPDATING OR UNINSTALLING RUST LATER
    // --------------------------------------------------------------------------------
    // Update Rust:
    //   rustup update

    // Uninstall Rust completely:
    //   rustup self uninstall
    // ================================================================================
    // "#.to_string();

    // println!("Agent is here...");
    /*
    let mut flag = true;

    while flag {
        let mut s = String::new();

        //read input
        match io::stdin().read_line(&mut s) {
            Ok(_) => (),
            Err(e) => return Err(e.into()),
        };

        let s = s.trim();
        if s.is_empty() {
            invalid_query();
            continue;
        }

        if s.starts_with("save") {
            //handle save handle
            // let content = s.strip_prefix("save").unwrap_or(s).trim().to_string();
            let content = read_multiline()?;
            save(&content, &client, &conn).await?;
        } else if s.starts_with("get") {
            // get handle get
            // println!("getting..");
            let query = s.strip_prefix("get").unwrap_or(s).trim().to_string();
            get(&query, &client, &conn).await?;
        } else if s.starts_with("end") {
            flag = false;
        } else {
            invalid_query();
        }
    }

    println!("Goode bye");

    // agent().await?;
    */
    Ok(())
}
