use addr::parse_domain_name;
use clap::Parser;
use imap;
use lettre::address::AddressError;
use lettre::message::header::ContentTransferEncoding;
use lettre::message::header::ContentType;
use lettre::message::Attachment;
use lettre::message::Body;
use lettre::message::MessageBuilder;
use lettre::message::MultiPartBuilder;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use mailparse::parse_mail;
use mailparse::MailHeaderMap;

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
    #[clap(short, long)]
    to: String, // whitespace separated
}

fn main() {
    let args = Args::parse();
    loop {
        println!("Checking for new mail...");
        fetch_inbox_top(&args).unwrap();
        println!("Sleeping for 5 minutes...");
        std::thread::sleep(std::time::Duration::from_secs(60 * 5));
    }
}

fn fetch_inbox_top(args: &Args) -> imap::error::Result<Option<imap::types::Fetch>> {
    let tls = native_tls::TlsConnector::builder().build().unwrap();

    // we pass in the domain twice to check that the server's TLS
    // certificate is valid for the domain we're connecting to.
    let _domain = parse_domain_name(&args.server).unwrap();
    let domain = _domain.root().unwrap();

    let client = imap::connect((args.server.clone(), args.imap_port), &args.server, &tls).unwrap();

    // the client we have here is unauthenticated.
    // to do anything useful with the e-mails, we need to log in
    let mut imap_session = client
        .login(&args.username, &args.password)
        .map_err(|e| e.0)?;

    // we want to fetch the first email in the INBOX mailbox
    imap_session.select("INBOX")?;

    // this returns a list of new emails
    let results = imap_session.search("NEW")?;

    if results.is_empty() {
        println!("No new emails");
    }

    for result in results {
        let messages = imap_session.uid_fetch(result.to_string(), "RFC822")?;
        let message = if let Some(m) = messages.iter().next() {
            m
        } else {
            return Ok(None);
        };
        let body = message.body().expect("message did not have a body!");
        // convert body to vector
        let body_vec = body.to_vec();
        // println!("{:?}", body);
        let parsed = parse_mail(body).unwrap();
        send_email(domain, &args, body_vec, parsed);
    }

    // be nice to the server and log out
    imap_session.logout()?;

    Ok(None)
}

// https://github.com/lettre/lettre/discussions/746#discussioncomment-2506754
pub trait MultipleAddressParser {
    fn to_addresses(self, addresses: &str) -> Result<MessageBuilder, AddressError>;
}

impl MultipleAddressParser for MessageBuilder {
    fn to_addresses(mut self, addresses: &str) -> Result<Self, AddressError> {
        for address in addresses.split_whitespace() {
            self = self.to(address.parse()?);
        }
        Ok(self)
    }
}

fn send_email(domain: &str, args: &Args, body: Vec<u8>, parsed: mailparse::ParsedMail) {
    let content_type = ContentType::parse("message/rfc822").unwrap();
    let body = Body::new_with_encoding(body, ContentTransferEncoding::EightBit).unwrap();
    let attachment = Attachment::new("original.eml".to_string()).body(body, content_type);
    let subject = parsed.headers.get_first_value("Subject").unwrap();

    let email = Message::builder()
        .from(format!("{}@{}", args.username, domain).parse().unwrap())
        .to_addresses(&args.to)
        .unwrap()
        .subject(format!("Fwd: {}", subject))
        .multipart(
            MultiPartBuilder::new()
                .kind(lettre::message::MultiPartKind::Mixed)
                .singlepart(attachment),
        )
        .unwrap();

    let creds = Credentials::new(args.username.clone(), args.password.clone());

    // Open a remote connection to gmail
    let mailer = SmtpTransport::starttls_relay(&args.server)
        .unwrap()
        .port(args.smtp_port)
        .credentials(creds)
        .build();

    // Send the email
    match mailer.send(&email) {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => panic!("Could not send email: {:?}", e),
    }
}
