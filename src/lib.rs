use anyhow::{anyhow, Result};
use async_stream::try_stream;
use futures::stream::Stream;
use someip_parse::{SdEntry, SdHeader, SomeIpHeader};
use std::{
    io::Cursor,
    net::{Ipv4Addr, SocketAddrV4},
};
use tokio::net::UdpSocket;

pub struct SomeIpServer {
    sd_multicast_addr: SocketAddrV4,
    service_id: u16,
    instance_id: u16,
    ttl: u32,
    minor_version: u32,
}

impl SomeIpServer {
    async fn serve() -> Result<()> {
        Ok(())
    }
}

pub struct FindServiceOpt {
    pub sd_multicast_addr: SocketAddrV4,
    pub service_id: u16,
    pub instance_id: u16,
    pub major_version: u8,
    pub ttl: u32,
    pub minor_version: u32,
}

impl Default for FindServiceOpt {
    fn default() -> Self {
        Self {
            sd_multicast_addr: SocketAddrV4::new(Ipv4Addr::new(224, 244, 224, 245), 30490),
            service_id: u16::MAX,
            instance_id: u16::MAX,
            major_version: u8::MAX,
            ttl: 65535,
            minor_version: u32::MAX,
        }
    }
}

#[derive(Debug)]
pub struct SomeIpClientOpt {
    sd_multicast_addr: SocketAddrV4,
    service_id: u16,
    instance_id: u16,
    major_version: u8,
    ttl: u32,
    minor_version: u32,
}

pub struct SomeIpClient {
    // sd_multicast_addr: SocketAddrV4,
// service_id: u16,
// instance_id: u16,
// major_version: u8,
// ttl: u32,
// minor_version: u32,
}

impl SomeIpClient {
    pub fn new() -> Self {
        Self {
            // sd_multicast_addr: opt.sd_multicast_addr,
            // service_id: opt.service_id,
            // instance_id: opt.instance_id,
            // major_version: opt.major_version,
            // ttl: opt.ttl,
            // minor_version: opt.minor_version,
        }
    }

    fn find_service_message(opt: &FindServiceOpt) -> Result<Vec<u8>, anyhow::Error> {
        // Init someip sd header
        let find_service = SdEntry::new_find_service_entry(
            0,
            0,
            0,
            0,
            opt.service_id,
            opt.instance_id,
            opt.major_version,
            opt.ttl,
            opt.minor_version,
        )
        .map_err(|err| anyhow!("Failed to create find_service entry: {:?}", err))?;
        let entries = vec![find_service];
        let someip_sd_header = SdHeader::new(false, entries, vec![]);
        let someip_sd_header_bytes = someip_sd_header.to_bytes_vec().unwrap();

        // Init someip header
        let length = (4 + 1 + 1 + 1 + 1 + someip_sd_header_bytes.len()) as u32;
        let someip_header = SomeIpHeader::new_sd_header(length, 0x01, None);
        let someip_header_bytes = someip_header.base_to_bytes();

        // Combine someip header and someip sd header
        Ok([&someip_header_bytes[..], &someip_sd_header_bytes].concat())
    }

    pub async fn find_service(
        &self,
        opt: &FindServiceOpt,
    ) -> Result<impl Stream<Item = Result<(SomeIpHeader, SdHeader)>>> {
        let inaddr_any = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 0);

        // Setup reading socket
        let read_socket = UdpSocket::bind(&opt.sd_multicast_addr).await?;
        read_socket.join_multicast_v4(*opt.sd_multicast_addr.ip(), *inaddr_any.ip())?;

        let message_bytes = Self::find_service_message(&opt)?;

        let socket = UdpSocket::bind(inaddr_any).await?;
        socket.connect(&opt.sd_multicast_addr).await?;
        socket.set_multicast_ttl_v4(10)?;
        socket.join_multicast_v4(*opt.sd_multicast_addr.ip(), *inaddr_any.ip())?;
        socket.send(&message_bytes).await?;
        println!("Waiting for response");

        let mut buffer = Vec::new();
        buffer.resize(1500, 0x00);

        let s = try_stream! {
            loop {
                let (_received, _addr) = read_socket.recv_from(&mut buffer).await?;

                let mut cursor = Cursor::new(&buffer);
                let someip_header = SomeIpHeader::read(&mut cursor)
                    .map_err(|err| anyhow!("Failed to read someip header: {:?}", err))?;

                if someip_header.is_someip_sd() {
                    let someip_sd = SdHeader::read(&mut cursor)
                        .map_err(|err| anyhow!("Failed to read someip sd header: {:?}", err))?;

                    yield (someip_header, someip_sd);
                } else {
                    continue;
                }
            }
        };
        Ok(s)
    }

    pub async fn subscribe() -> Result<()> {
        Ok(())
    }
}
