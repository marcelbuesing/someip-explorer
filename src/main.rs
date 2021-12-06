use std::net::SocketAddrV4;

use anyhow::Result;
use someipsd::{FindServiceOpt, SomeIpClient};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(long)]
    sd_multicast_addr: SocketAddrV4,
    #[structopt(long, default_value = "65535")]
    service_id: u16,
    #[structopt(long, default_value = "65535")]
    instance_id: u16,
    #[structopt(long, default_value = "255")]
    major_version: u8,
    #[structopt(long, default_value = "65535")]
    ttl: u32,
    #[structopt(long, default_value = "4294967295")]
    minor_version: u32,
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = SomeIpClient::new();
    let opt = Opt::from_args();
    client.find_service(&FindServiceOpt::default()).await?;
    Ok(())
}
