use lightspeed_core::model::boolean::Boolean;
use lightspeed_email::config::EmailClientConfig;
use lightspeed_email::model::email::{EmailAttachment, EmailMessage};
use lightspeed_email::repository::email::{new, EmailClientType};
use lightspeed_email::service::EmailService;
use testcontainers::*;

pub fn new_mail_server(docker: &clients::Cli) -> (u16, Container<clients::Cli, images::generic::GenericImage>) {
    let node = docker.run(
        images::generic::GenericImage::new("mailhog/mailhog:v1.0.0")
            .with_wait_for(images::generic::WaitFor::message_on_stdout("Creating API v2 with WebPath:")),
    );

    (node.get_host_port(1025).unwrap(), node)
}

#[tokio::test]
async fn should_start_the_mailserver() {
    // Arrange
    let docker = clients::Cli::default();
    let server = new_mail_server(&docker);
    let server_port = server.0;
    println!("using port: {}", server_port);

    let config = EmailClientConfig {
        email_server_port: server_port,
        email_server_address: "127.0.0.1".to_owned(),
        email_client_type: EmailClientType::Full,
        email_client_timeout_seconds: 60,
        email_server_username: "".to_owned(),
        email_server_password: "".to_owned(),
        email_server_use_tls: Boolean::False,
        forward_all_emails_to_fixed_recipients: None,
    };

    let email_service = EmailService::new(new(config).unwrap());

    let mut message = EmailMessage::new();
    message.from = Some("UFO <ufoscout@gmail.com>".to_owned());
    message.to.push("ufoscout@gmail.com".to_owned());
    message.to.push("NAME <test@gmail.com>".to_owned());
    message.subject = Some("subject".to_owned());
    message.attachments.push(EmailAttachment::FromFile {
        mime_type: "plain/text".to_owned(),
        path: "./Cargo.toml".to_owned(),
        filename: Some("cargo.txt".to_owned()),
    });

    // Act
    email_service.send(message.clone()).await.unwrap();
    assert!(email_service.send(message.clone()).await.is_ok());
    // should reuse the client
    assert!(email_service.send(message.clone()).await.is_ok());
    assert!(email_service.send(message.clone()).await.is_ok());
}
/*
#[tokio::test]
async fn full_client_should_use_gmail() {
    // Arrange
    let config = EmailClientConfig {
        server_port: 465,
        server_address: "smtp.gmail.com".to_owned(),
        client_type: EmailClientType::Full,
        server_username: "ufoscout@gmail.com".to_owned(),
        server_password: "".to_owned(),
        server_use_tls: Boolean::True,
        forward_all_emails_to_fixed_recipients: None,
    };

    let email_service = new(config).unwrap();

    let mut message = EmailMessage::new();
    message.from = Some("UFOSCOUT <ufoscout@gmail.com>".to_owned());
    message.to.push("FRANCESCO <ufoscout@gmail.com>".to_owned());
    message.subject = Some("EMAIL FROM RUST!!".to_owned());
    message.html = Some("HTML body".to_owned());
    message.attachments.push(EmailAttachment::FromFile {
        mime_type: "plain/text".to_owned(),
        path: "./Cargo.toml".to_owned(),
        filename: Some("cargo.txt".to_owned())
    });

    // Act
    email_service.send(message.clone()).await.unwrap();

}
*/
