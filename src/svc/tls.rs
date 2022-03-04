use tonic::transport::{Certificate, ClientTlsConfig, Identity};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::error::Error;


#[tokio::main]
async fn build_tls_config(cli_key_file: &String, cli_cert_file: &String, ca_cert_file: &String)
                          -> Result<ClientTlsConfig, Box<dyn Error>> {
    let mut cli_key_bytes: Vec<u8> = Vec::new();
    let mut cli_cert_bytes: Vec<u8> = Vec::new();
    let mut ca_cert_bytes: Vec<u8> = Vec::new();

    File::open(Path::new(cli_key_file))?.read_to_end(&mut cli_key_bytes)?;
    File::open(Path::new(cli_cert_file))?.read_to_end(&mut cli_cert_bytes)?;
    File::open(Path::new(ca_cert_file))?.read_to_end(&mut ca_cert_bytes)?;

    let tls_config = ClientTlsConfig::new();
    let tls_config = tls_config.ca_certificate(Certificate::from_pem(ca_cert_bytes));
    let tls_config = tls_config.identity(Identity::from_pem(cli_cert_bytes, cli_key_bytes));

    Ok(tls_config)
}