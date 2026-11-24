use rusqlite::{Connection, Result};

#[derive(Debug)]
struct Person {
    id: i32,
    name: String,
}

fn main() -> Result<()> {
    println!("Hello, world!");

    let conn = Connection::open("recall.db")?;

    // let mut s = String::new();

    conn.execute(
        "CREATE TABLE person(
       id INTEGER PRIMARY KEY,
       name TEXT NOT NULL
    );",
        (),
    )?;

    let person = Person {
        id: 23,
        name: "raman".to_string(),
    };

    conn.execute(
        "INSERT INTO person (id,name) VALUES ($1, $2)",
        (&person.id, &person.name),
    )?;

    // match io::stdin().read_line(&mut s) {
    //     Ok(_) => (),
    // };

    // let list = s.trim().split(" ");

    // for item in list {
    //     println!("{}", item);
    // }

    Ok(())
}
