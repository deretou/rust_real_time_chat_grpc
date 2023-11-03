// src/bin/server/main.rs

use tonic::{transport::Server, Request, Response, Status};
use chat::chat_server::{Chat, ChatServer};
use chat::{ChatMessage};
use futures::Stream;
use std::pin::Pin;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

pub mod chat {
    tonic::include_proto!("chat"); // The string specified here must match the proto package name
}

#[derive(Debug)]
pub struct MyChatService {
    sender: broadcast::Sender<ChatMessage>,
}

impl Default for MyChatService {
    fn default() -> Self {
        let (sender, _) = broadcast::channel(10);
        MyChatService { sender }
    }
}

impl MyChatService {
    fn new() -> Self {
        let (sender, _) = broadcast::channel(10);
        Self { sender }
    }
}

type ChatStreamType = Pin<Box<dyn Stream<Item = Result<ChatMessage, Status>> + Send + Sync>>;

#[tonic::async_trait]
impl Chat for MyChatService {
    type ChatStreamStream = ChatStreamType;

    async fn chat_stream(
        &self,
        request: Request<tonic::Streaming<ChatMessage>>,
    ) -> Result<Response<Self::ChatStreamStream>, Status> {
        let mut stream = request.into_inner();
        let sender = self.sender.clone();
        let receiver = sender.subscribe();

        tokio::spawn(async move {
            while let Some(message) = stream.message().await.unwrap() {
                sender.send(message).unwrap();
            }
        });

        let output_stream = BroadcastStream::new(receiver).map(|message| {
            Ok(message.unwrap())
        });

        Ok(Response::new(Box::pin(output_stream) as Self::ChatStreamStream))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let chat_service = MyChatService::new();

    Server::builder()
        .add_service(ChatServer::new(chat_service))
        .serve(addr)
        .await?;

    Ok(())
}
