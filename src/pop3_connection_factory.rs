use std::sync::Arc;
use std::net::TcpStream;
use std::error::Error;

use rustls::{RootCertStore, ClientConnection, StreamOwned};

use crate::LineReader;
use crate::Pop3ConnectionImpl;

pub struct Pop3ConnectionFactory {

}

impl Pop3ConnectionFactory {
    /// Returns a new POP3 connection.
    ///
    /// # Arguments
    ///
    /// * `host` - IP-Address or host name of the POP3 server to connect
    /// * `port` - Port of the POP3 server to connect
    pub fn new(host: &str, port: u16) -> Result<Pop3ConnectionImpl<StreamOwned<ClientConnection, TcpStream>>, Box<dyn Error>> {
        let mut root_store = RootCertStore::empty();
        for cert in rustls_native_certs::load_native_certs().certs {
            root_store.add(&rustls::Certificate(cert.to_vec()))?;
        }

        Pop3ConnectionFactory::with_custom_certs(host, port, root_store)
    }

    /// Returns a new POP3 connection with custom certificates.
    ///
    /// # Arguments
    ///
    /// * `host` - IP-Address or host name of the POP3 server to connect
    /// * `port` - Port of the POP3 server to connect
    /// * `root_store` - Store of trusted (root) certificates.
    ///
    /// # Examples
    ///
    /// ```
    /// use rust_pop3_client::Pop3ConnectionFactory;
    /// use rustls::RootCertStore;
    ///
    /// let mut root_store = RootCertStore::empty();
    /// for cert in rustls_native_certs::load_native_certs().unwrap() {
    ///     root_store.add(&rustls::Certificate(cert.0)).unwrap();
    /// }
    /// 
    /// let connection = Pop3ConnectionFactory::with_custom_certs("", 995, root_store);
    /// ```
    pub fn with_custom_certs(host: &str, port: u16, root_store: RootCertStore) -> Result<Pop3ConnectionImpl<StreamOwned<ClientConnection, TcpStream>>, Box<dyn Error>> {
        let config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let server_name = host.try_into()?;

        let connection = rustls::ClientConnection::new(Arc::new(config), server_name)?;
        let stream =  TcpStream::connect(format!("{}:{}", host, port))?;
        let stream = rustls::StreamOwned::new(connection, stream);

        let mut client = Pop3ConnectionImpl { 
            stream,
            reader: LineReader::new()
        };

        client.read_status_line()?;
        Ok(client)
    }

    /// Returns a new POP3 conneciton without TLS.
    /// 
    /// # Argumetns
    /// 
    /// * `host` - IP-Address or host name of the POP3 server to connect
    /// * `port` - Port of the POP3 server to connect
    pub fn without_tls(host: &str, port: u16) -> Result<Pop3ConnectionImpl<TcpStream>, Box<dyn Error>> {
        let stream =  TcpStream::connect(format!("{}:{}", host, port))?;
        let mut client = Pop3ConnectionImpl { 
            stream,
            reader: LineReader::new()
        };
        client.read_status_line()?;
        Ok(client)
    }

}
