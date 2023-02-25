use std::error::Error;

use addr::parse_domain_name;
use clap::Parser;
use imap;
use lettre::address::Envelope;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::response::Response;
use lettre::Address;
use lettre::{SmtpTransport, Transport};
use log::{info, warn};
use simple_logger::SimpleLogger;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long)]
    username: String,
    #[clap(short, long)]
    password: String,
    #[clap(short, long)]
    server: String,
    #[clap(long, default_value_t = 587)]
    smtp_port: u16,
    #[clap(long, default_value_t = 993)]
    imap_port: u16,
    #[clap(long, default_value_t = false)]
    insecure: bool,
    #[clap(short, long)]
    to: String, // whitespace separated
}

fn main() {
    let args = Args::parse();
    SimpleLogger::new().init().unwrap();
    loop {
        info!("Checking for new mail...");
        match fetch_and_send(&args) {
            Ok(_) => info!("Successfully sent email!"),
            Err(x) => warn!("{}", x),
        }
        info!("Sleeping for 5 minutes...");
        std::thread::sleep(std::time::Duration::from_secs(60 * 5));
    }
}

fn fetch_and_send(args: &Args) -> Result<(), Box<dyn Error>> {
    let tls = native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(args.insecure)
        .build()
        .unwrap();

    // We pass in the domain twice to check that the server's TLS
    // certificate is valid for the domain we're connecting to.
    let _domain = parse_domain_name(&args.server).unwrap();
    let domain = _domain.root().unwrap();

    let client = imap::connect((args.server.clone(), args.imap_port), &args.server, &tls).unwrap();

    // The client we have here is unauthenticated.
    // To do anything useful with the e-mails, we need to log in
    let mut imap_session = client
        .login(&args.username, &args.password)
        .map_err(|e| e.0)?;

    // We want to fetch emails in the INBOX
    imap_session.select("INBOX")?;

    // This returns a list of unseen emails, which are all emails not marked as read.
    let results = imap_session.search("UNSEEN")?;

    if results.is_empty() {
        Err("No new emails")?
    }

    for result in results {
        let messages = imap_session.uid_fetch(result.to_string(), "RFC822")?;
        for message in &messages {
            let body = message.body().expect("message did not have a body!");
            send_email(domain, &args, body.to_vec())?;
        }
    }
    // Be nice to the server and log out
    imap_session.logout()?;
    Ok(())
}

fn send_email(
    domain: &str,
    args: &Args,
    body: Vec<u8>,
) -> Result<Response, lettre::transport::smtp::Error> {
    let creds = Credentials::new(args.username.clone(), args.password.clone());

    // Open a remote connection to source
    let mailer = SmtpTransport::starttls_relay(&args.server)
        .unwrap()
        .port(args.smtp_port)
        .credentials(creds)
        .build();

    // Build envelope
    let from = format!("{}@{}", args.username, domain)
        .parse::<Address>()
        .unwrap();
    let recipients = args
        .to
        .split_whitespace()
        .map(|recipient| recipient.parse::<Address>().unwrap())
        .collect();
    let envelope = Envelope::new(Some(from), recipients).unwrap();
    // Send the email
    mailer.send_raw(&envelope, &body)
}
