use lightspeed_core::model::boolean::Boolean;
use lightspeed_email::config::EmailClientConfig;
use lightspeed_email::model::email::EmailMessage;
use lightspeed_email::repository::email::{new, EmailClientType};
use lightspeed_email::service::EmailService;
use testcontainers::*;

pub fn new_mail_server(
    docker: &clients::Cli,
) -> (u16, Container<clients::Cli, images::generic::GenericImage>) {
    let node = docker.run(
        images::generic::GenericImage::new("mailhog/mailhog:v1.0.0").with_wait_for(
            images::generic::WaitFor::message_on_stdout("Creating API v2 with WebPath:"),
        ),
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
        server_port,
        server_address: "127.0.0.1".to_owned(),
        client_type: EmailClientType::Full,
        server_username: "".to_owned(),
        server_password: "".to_owned(),
        server_use_tls: Boolean::False,
    };

    let email_service = EmailService::new(new(config).unwrap());

    let mut message = EmailMessage::new();
    message.from = Some("UFO <ufoscout@gmail.com>".to_owned());
    message.to.push("ufoscout@gmail.com".to_owned());
    message.to.push("NAME <test@gmail.com>".to_owned());
    message.subject = Some("subject".to_owned());

    // Act
    assert!(email_service.send(message.clone()).await.is_ok());
    // should reuse the client
    assert!(email_service.send(message.clone()).await.is_ok());
    assert!(email_service.send(message.clone()).await.is_ok());
}

// #[tokio::test]
// async fn full_client_should_use_gmail() {
//     // Arrange
//
//     let config = EmailClientConfig {
//         server_port: 465,
//         server_address: "smtp.gmail.com".to_owned(),
//         client_type: EmailClientType::Full,
//         server_username: "ufoscout@gmail.com".to_owned(),
//         server_password: "".to_owned(),
//         server_use_tls: Boolean::True,
//     };
//
//     let email_service = new(config).unwrap();
//
//     let mut message = EmailMessage::new();
//     message.from = Some("UFOSCOUT <ufoscout@gmail.com>".to_owned());
//     message.to.push("FRANCESCO <ufoscout@gmail.com>".to_owned());
//     message.subject = Some("EMAIL FROM RUST!!".to_owned());
//
//     // Act
//     assert!(email_service.send(message.clone()).await.is_ok());
//
// }
