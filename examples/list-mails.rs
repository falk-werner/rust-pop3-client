use std::error::Error;
use std::io::{self, Write};
use clap::Parser;

extern crate rust_pop3_client;

use rust_pop3_client::Pop3ConnectionFactory;
use rust_pop3_client::Pop3Connection;


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

    let mut connection: Box<dyn Pop3Connection> = match args.disable_tls {
        true => Box::new(Pop3ConnectionFactory::without_tls(&args.server, args.port)?),
        false => Box::new(Pop3ConnectionFactory::new(&args.server, args.port)?),
    };


    connection.login(&args.username, &password)?;
    let stat = connection.stat()?;
    println!("message count: {}", stat.message_count);
    println!("maildrop size: {}", stat.maildrop_size);
    println!();

    println!("id\tsize\tsubject");
    let infos = connection.list()?;
    for info in infos {
        let header = connection.top(info.message_id, 0)?;
        let mut subject = String::from("unknown");
        for line in header.lines() {
            if line.starts_with("Subject:") {
                let sub: &str = &line[8..].trim();
                subject.replace_range(..,sub);
            }
        }

        println!("{}\t{}\t{}", info.message_id, info.message_size, subject);
    }
    Ok(())
}
