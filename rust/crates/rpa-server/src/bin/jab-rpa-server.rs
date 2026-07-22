use jab_rpa_server::JabService;
use jab_rpa_server::proto::jab_service_server::JabServiceServer;
use jab_wrapper::wrapper::JabWrapper;
use tonic::transport::Server;

use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(default_value = "127.0.0.1")]
    address: String,

    #[arg(default_value = "50051")]
    port: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let wrapper = JabWrapper::new()?;

    // Wait for JAB init
    let service = JabService::new(wrapper);

    let addr = format!("{}:{}", args.address, args.port).parse()?;
    println!("JAB gRPC Server listening on {}", addr);

    Server::builder()
        .add_service(JabServiceServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
