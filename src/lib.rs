use anyhow::{anyhow, Result};
use colored::Colorize;
use someip_parse::{SomeIpHeader, SomeIpSdEntry, SomeIpSdHeader, SomeIpSdOption};
use std::{
    io::Cursor,
    net::{Ipv4Addr, Ipv6Addr, SocketAddrV4},
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
    sd_multicast_addr: SocketAddrV4,
    service_id: u16,
    instance_id: u16,
    major_version: u8,
    ttl: u32,
    minor_version: u32,
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
        let find_service = SomeIpSdEntry::new_find_service_entry(
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
        let someip_sd_header = SomeIpSdHeader::new(false, entries, vec![]);
        let someip_sd_header_bytes = someip_sd_header.to_bytes();

        // Init someip header
        let length = (4 + 1 + 1 + 1 + 1 + someip_sd_header_bytes.len()) as u32;
        let someip_header = SomeIpHeader::new_sd_header(length, 0x01, None);
        let someip_header_bytes = someip_header.base_to_bytes();

        // Combine someip header and someip sd header
        Ok([&someip_header_bytes[..], &someip_sd_header_bytes].concat())
    }

    pub async fn find_service(&self, opt: &FindServiceOpt) -> Result<()> {
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

        loop {
            match read_socket.recv_from(&mut buffer).await {
                Ok((_received, _addr)) => {
                    let mut cursor = Cursor::new(&buffer);
                    let someip_header = SomeIpHeader::read(&mut cursor)
                        .map_err(|err| anyhow!("Failed to read someip header: {:?}", err))?;

                    if someip_header.is_someip_sd() {
                        let someip_sd = SomeIpSdHeader::read(&mut cursor)
                            .map_err(|err| anyhow!("Failed to read someip sd header: {:?}", err))?;

                        println!(
                            "\n\n{}",
                            format!("\nSOME/IP SD {:?}\n", someip_header.message_type)
                                .bold()
                                .magenta()
                        );

                        let event_id = someip_header
                            .event_id()
                            .map(|event_id| format!("Event Id: {:?}", event_id));
                        let method_id = someip_header
                            .method_id()
                            .map(|method_id| format!("Method Id: {:?}", method_id));
                        let event_or_method_id = event_id
                            .or(method_id)
                            .ok_or(anyhow!("Missing event and method id"))?;

                        println!(
                            "{} Length:\t{} Request Id: {}\tInterface Version: {}\tReturn Code: {}\nReboot: {}\tUnicast: {}\tExplicit Initial Data Control: {}",
                            event_or_method_id,
                            someip_header.length,
                            someip_header.request_id,
                            someip_header.interface_version,
                            someip_header.return_code,
                            someip_sd.reboot,
                            someip_sd.unicast,
                            someip_sd.explicit_initial_data_control
                        );

                        println!("\n{}", "Entries:\n".underline());
                        for entry in someip_sd.entries {
                            display_entry(&entry);
                        }

                        println!("\n{}", "Options:\n".underline());
                        for option in someip_sd.options {
                            display_option(&option);
                        }
                    }
                }
                Err(e) => println!("recv function failed: {:?}", e),
            }
        }
    }
}

fn display_entry(entry: &SomeIpSdEntry) {
    match entry {
        SomeIpSdEntry::Service {
            _type,
            index_first_option_run: _,
            index_second_option_run: _,
            number_of_options_1: _,
            number_of_options_2: _,
            service_id,
            instance_id,
            major_version,
            ttl,
            minor_version,
        } => {
            println!(
                "{} Service Id: {} Instance Id: {} Version: {} TTL: {}",
                format!("{:?}", _type).cyan(),
                format!("{}", service_id).red(),
                format!("{}", instance_id).green(),
                format!("{}.{}", major_version, minor_version).blue(),
                format!("{}", ttl).yellow(),
            );
        }
        SomeIpSdEntry::Eventgroup {
            _type,
            index_first_option_run,
            index_second_option_run,
            number_of_options_1,
            number_of_options_2,
            service_id,
            instance_id,
            major_version,
            ttl,
            initial_data_requested,
            counter,
            eventgroup_id,
        } => todo!(),
    }
}

fn display_option(option: &SomeIpSdOption) {
    match option {
        SomeIpSdOption::Configuration {
            configuration_string: _,
        } => todo!(),
        SomeIpSdOption::LoadBalancing { priority, weight } => {
            println!("LoadBalancing priority: {} weight: {}", priority, weight)
        }
        SomeIpSdOption::Ipv4Endpoint {
            ipv4_address,
            transport_protocol,
            transport_protocol_number: _,
        } => println!(
            "Ipv4Endpoint {} via {:?}",
            Ipv4Addr::from(*ipv4_address),
            transport_protocol
        ),
        SomeIpSdOption::Ipv6Endpoint {
            ipv6_address,
            transport_protocol,
            transport_protocol_number: _,
        } => println!(
            "Ipv6Endpoint {} via {:?}",
            Ipv6Addr::from(*ipv6_address),
            transport_protocol
        ),
        SomeIpSdOption::Ipv4Multicast {
            ipv4_address,
            transport_protocol,
            transport_protocol_number: _,
        } => println!(
            "Ipv4Multicast {} via {:?}",
            Ipv4Addr::from(*ipv4_address),
            transport_protocol
        ),
        SomeIpSdOption::Ipv6Multicast {
            ipv6_address,
            transport_protocol,
            transport_protocol_number: _,
        } => println!(
            "Ipv6Multicast {} via {:?}",
            Ipv6Addr::from(*ipv6_address),
            transport_protocol
        ),
        SomeIpSdOption::Ipv4SdEndpoint {
            ipv4_address,
            transport_protocol,
            transport_protocol_number: _,
        } => println!(
            "Ipv4SdEndpoint {} via {:?}",
            Ipv4Addr::from(*ipv4_address),
            transport_protocol
        ),
        SomeIpSdOption::Ipv6SdEndpoint {
            ipv6_address,
            transport_protocol,
            transport_protocol_number: _,
        } => println!(
            "Ipv6SdEndpoint {} via {:?}",
            Ipv6Addr::from(*ipv6_address),
            transport_protocol
        ),
    }
}
