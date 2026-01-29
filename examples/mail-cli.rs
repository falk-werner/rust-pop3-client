use std::error::Error;
use std::io::{self, Write, stdout};
use clap::{Parser, Subcommand};

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

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Get mailbox statistics
    Stat,

    /// List messages
    List,

    /// Get Message Header
    Top {
        /// Unique Message ID
        id: String,

        #[arg(short, long, default_value_t=0)]
        lines: u32
    },

    /// Retrieve Message
    Retrieve {
        /// Unique Message ID
        id: String,
    },
}

fn read_password(prompt: &str) -> Result<String, Box<dyn Error>> {
    print!("{}: ", prompt);
    io::stdout().flush()?;
    Ok(rpassword::read_password()?)
}

fn stat(connection: &mut Box<dyn Pop3Connection>) -> Result<(), Box<dyn Error>> {
    let stat = connection.stat()?;
    println!("MESSAGE_COUNT: {}", stat.message_count);
    println!("MAILDROP_SIZE: {}", stat.maildrop_size);

    Ok(())
}

fn list(connection: &mut Box<dyn Pop3Connection>) -> Result<(), Box<dyn Error>> {
    let infos = connection.list()?;
    for info in infos {
        let unique_id = connection.get_unique_id(info.message_id)?;
        let header = connection.top(info.message_id, 0)?;
        let mut subject = String::from("unknown");
        let mut from: String = String::from("unknown");
        for line in header.lines() {
            if line.starts_with("Subject:") {
                subject.replace_range(..,line);
            }
            else if line.starts_with("From:") {
                from.replace_range(.., line);
            }
        }

        println!("ID: {}", unique_id);
        println!("SIZE: {}", info.message_size);
        println!("{}", subject);
        println!("{}", from);
        println!();
    }

    Ok(())
}

fn top(connection: &mut Box<dyn Pop3Connection>, id: &str, lines: u32) -> Result<(), Box<dyn Error>> {
    let infos = connection.list()?;
    for info in infos {
        let unique_id = connection.get_unique_id(info.message_id)?;
        if id == unique_id {
            let header = connection.top(info.message_id, lines)?;
            println!("{}", header);
            break;
        }
    }

    Ok(())
}

fn retrieve(connection: &mut Box<dyn Pop3Connection>, id: &str) -> Result<(), Box<dyn Error>> {
    let infos = connection.list()?;
    for info in infos {
        let unique_id = connection.get_unique_id(info.message_id)?;
        if id == unique_id {
            connection.retrieve(info.message_id, &mut stdout())?;
            break;
        }
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let password = read_password("password")?;

    let mut connection: Box<dyn Pop3Connection> = match args.disable_tls {
        true => Box::new(Pop3ConnectionFactory::without_tls(&args.server, args.port)?),
        false => Box::new(Pop3ConnectionFactory::new(&args.server, args.port)?),
    };


    connection.login(&args.username, &password)?;


    match args.command {
        Commands::Stat => stat(&mut connection)?,
        Commands::List => list(&mut connection)?,
        Commands::Top{id, lines} => top(&mut connection, &id, lines)?,
        Commands::Retrieve{id} => retrieve(&mut connection, &id)?,
    }
    Ok(())
}
