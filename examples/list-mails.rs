use std::error::Error;
use std::io::{self, Write};

extern crate rust_pop3_client;

use rust_pop3_client::Pop3Connection;

fn read_value(prompt: &str) -> Result<String, Box<dyn Error>> {
    print!("{}: ", prompt);
    io::stdout().flush()?;
    let mut value = String::new();
    io::stdin().read_line(&mut value)?;
    Ok(String::from(value.trim()))
}

fn read_password(prompt: &str) -> Result<String, Box<dyn Error>> {
    print!("{}: ", prompt);
    io::stdout().flush()?;
    Ok(rpassword::read_password()?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let host = read_value("host (e.g. pop.gmail.com)")?;
    let port = read_value("port (e.g. 995)")?.parse::<u16>()?;
    let user = read_value("user (e-mail address)")?;
    let password = read_password("password")?;

    let mut connection = Pop3Connection::new(&host, port)?;
    connection.login(&user, &password)?;

    println!("id\tsize");
    let infos = connection.list()?;
    for info in infos {
        println!("{}\t{}", info.message_id, info.message_size);
    }

    Ok(())
}
