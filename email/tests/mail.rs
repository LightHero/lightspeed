use testcontainers::*;
use lightspeed_email::config::EmailConfig;
use lightspeed_email::model::email::EmailMessage;
use lightspeed_email::service::email::{EmailServiceType, new};

pub fn new_mail_server(
    docker: &clients::Cli,
) -> (
    u16,
    Container<clients::Cli, images::generic::GenericImage>,
) {
    let node =
        docker.run(images::generic::GenericImage::new("mailhog/mailhog:v1.0.0")
                 .with_wait_for(images::generic::WaitFor::message_on_stdout(
                    "Creating API v2 with WebPath:",
                 )));

    (node.get_host_port(1025).unwrap() as u16, node)
}

#[test]
pub fn should_start_the_mailserver() {
    // Arrange
    let docker = clients::Cli::default();
    let server_port = new_mail_server(&docker).0;
    println!("using port: {}", server_port);

    let config = EmailConfig {
        server_port,
        server_address: "127.0.0.1".to_owned(),
        service_type: EmailServiceType::Full
    };

    let email_service = new(config);

    let mut message  = EmailMessage::new();
    message.from = Some("ufoscout@gmail.com".to_owned());
    message.to.push("ufoscout@gmail.com".to_owned());
    message.subject = Some("subject".to_owned());

    // Act
    let result = email_service.send(message);

    // Assert
    assert!(result.is_ok());
}