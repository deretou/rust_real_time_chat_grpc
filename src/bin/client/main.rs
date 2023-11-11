// src/bin/client/main.rs

use chat::chat_client::ChatClient;
use chat::ChatMessage;
use tokio::sync::mpsc;

use tokio::io::{self, AsyncBufReadExt, BufReader};
use tonic::Request;

pub mod chat {
    tonic::include_proto!("chat");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = ChatClient::connect("http://[::1]:50051").await?;

    let (tx, rx) = mpsc::channel(10);

    // Read messages from stdin and send them to the server
    tokio::spawn(async move {
        let mut reader = BufReader::new(io::stdin());
        let mut line = String::new();
        while reader.read_line(&mut line).await.unwrap() != 0 {
            let msg = ChatMessage {
                // Assuming your ChatMessage has a `sender` field.
                sender: "username".to_string(),
                content: line.clone(),
            };
            // Send msg to the channel
            tx.send(msg).await.unwrap();
            line.clear();
        }
    });

    let response = client.chat_stream(Request::new(tokio_stream::wrappers::ReceiverStream::new(rx))).await?;

    let mut stream = response.into_inner();

    while let Some(message) = stream.message().await? {
        println!("{}: {}", message.sender, message.content);
    }

    Ok(())
}

