use jab_rpa_server::JabService;
use jab_rpa_server::proto::jab_service_server::JabServiceServer;
use jab_wrapper::wrapper::JabWrapper;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wrapper = JabWrapper::new()?;

    // Wait for JAB init
    let service = JabService::new(wrapper);

    let addr = "127.0.0.1:50051".parse()?;
    println!("JAB gRPC Server listening on {}", addr);

    Server::builder()
        .add_service(JabServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
