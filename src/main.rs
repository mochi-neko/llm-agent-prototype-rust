mod chat;

use chat::my_chat::chat_rpc::chat_server::ChatServer;
use chat::my_chat::MyChat;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let address = "0.0.0.0:8000".parse()?;
    let chat = MyChat::default();

    let reflection_server = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(chat::my_chat::chat_rpc::FILE_DESCRIPTOR_SET)
        .build()?;

    Server::builder()
        .add_service(ChatServer::new(chat))
        .add_service(reflection_server)
        .serve(address)
        .await?;

    Ok(())
}
