use std::error::Error;
use std::io::{self, Write};
use clap::Parser;

extern crate rust_pop3_client;

use rust_pop3_client::Pop3Connection;
use rust_pop3_client::Pop3Conn;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Hostname, e.g. pop.gmail.com
    #[arg(short, long)]
    server: String,

    /// Port number
    #[arg(short, long, default_value_t = 995)]
    port: u16,

    /// Disable TLS
    #[arg(short, long, default_value_t = false)]
    disable_tls: bool,

    /// Username, e.g. e-mail address
    #[arg(short, long)]
    username: String,
}

fn read_password(prompt: &str) -> Result<String, Box<dyn Error>> {
    print!("{}: ", prompt);
    io::stdout().flush()?;
    Ok(rpassword::read_password()?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let password = read_password("password")?;

    let mut connection: Box<dyn Pop3Conn> = match args.disable_tls {
        true => Box::new(Pop3Connection::new(&args.server, args.port)?),
        false => Box::new(Pop3Connection::without_tls(&args.server, args.port)?)
    };


    connection.login(&args.username, &password)?;
    Ok(())

//        println!("id\tsize");
//        let infos = connection.list()?;
//        for info in infos {
//            println!("{}\t{}", info.message_id, info.message_size);
//        }
//
}
