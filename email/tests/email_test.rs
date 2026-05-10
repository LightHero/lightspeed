use lightspeed_email::config::EmailClientConfig;
use lightspeed_email::model::email::{EmailAttachment, EmailMessage};
use lightspeed_email::repository::email::{EmailClientType, new};
use lightspeed_email::service::LsEmailService;
use testcontainers::testcontainers::core::WaitFor;
use testcontainers::testcontainers::runners::AsyncRunner;
use testcontainers::testcontainers::{ContainerAsync, GenericImage};

pub async fn new_mail_server() -> (u16, u16, ContainerAsync<GenericImage>) {
    let node = GenericImage::new("axllent/mailpit", "latest")
        .with_wait_for(WaitFor::message_on_stdout("[http] accessible via"))
        .start()
        .await
        .unwrap();

    let smtp_port = node.get_host_port_ipv4(1025).await.unwrap();
    let api_port = node.get_host_port_ipv4(8025).await.unwrap();
    (smtp_port, api_port, node)
}

#[tokio::test]
async fn should_start_the_mailserver() {
    // Arrange
    let (smtp_port, api_port, _container) = new_mail_server().await;
    println!("using SMTP port: {smtp_port}, API port: {api_port}");

    let config = EmailClientConfig {
        email_server_port: smtp_port,
        email_server_address: "127.0.0.1".to_owned(),
        email_client_type: EmailClientType::Full,
        email_client_timeout_seconds: 60,
        email_server_username: "".to_owned(),
        email_server_password: "".to_owned(),
        // Local Mailcrab container — no TLS. Production must keep this false.
        dangerous_no_tls: true,
        forward_all_emails_to_fixed_recipients: None,
    };

    let email_service = LsEmailService::new(new(config).unwrap());

    let mut message = EmailMessage::new();
    message.from = Some("UFO <ufoscout@gmail.com>".to_owned());
    message.to.push("ufoscout@gmail.com".to_owned());
    message.to.push("NAME <test@gmail.com>".to_owned());
    message.subject = Some("subject".to_owned());
    message.text = Some("hello from lightspeed test".to_owned());
    message.attachments.push(EmailAttachment::FromFile {
        mime_type: "plain/text".to_owned(),
        path: "./Cargo.toml".to_owned(),
        filename: Some("cargo.txt".to_owned()),
    });

    // Act
    email_service.send(message.clone()).await.unwrap();

    // Assert: query Mailpit's REST API and verify the email was received
    let messages_url = format!("http://127.0.0.1:{api_port}/api/v1/messages");
    let response: serde_json::Value = reqwest::get(&messages_url).await.unwrap().json().await.unwrap();

    let messages = response["messages"].as_array().expect("messages array");
    assert_eq!(1, messages.len(), "expected exactly one message in Mailpit");

    let received = &messages[0];
    assert_eq!("subject", received["Subject"].as_str().unwrap());
    assert_eq!("ufoscout@gmail.com", received["From"]["Address"].as_str().unwrap());

    let to_addresses: Vec<&str> =
        received["To"].as_array().unwrap().iter().map(|v| v["Address"].as_str().unwrap()).collect();
    assert!(to_addresses.contains(&"ufoscout@gmail.com"), "missing recipient ufoscout@gmail.com");
    assert!(to_addresses.contains(&"test@gmail.com"), "missing recipient test@gmail.com");

    assert_eq!(1, received["Attachments"].as_i64().unwrap(), "expected one attachment");

    // Fetch the full message to verify the text body
    let message_id = received["ID"].as_str().unwrap();
    let message_url = format!("http://127.0.0.1:{api_port}/api/v1/message/{message_id}");
    let full_message: serde_json::Value = reqwest::get(&message_url).await.unwrap().json().await.unwrap();
    assert_eq!("hello from lightspeed test", full_message["Text"].as_str().unwrap().trim());
}
