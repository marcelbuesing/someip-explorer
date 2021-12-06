use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4};

use anyhow::{anyhow, Result};
use colored::Colorize;
use futures::{pin_mut, StreamExt};
use someip_parse::{SomeIpSdEntry, SomeIpSdOption};
use someipsd::{FindServiceOpt, SomeIpClient};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
enum Opt {
    FindService {
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
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let client = SomeIpClient::new();
    let opt = Opt::from_args();
    match opt {
        Opt::FindService {
            sd_multicast_addr,
            service_id,
            instance_id,
            major_version,
            ttl,
            minor_version,
        } => {
            let find_service_opt = FindServiceOpt {
                sd_multicast_addr,
                service_id,
                instance_id,
                major_version,
                ttl,
                minor_version,
            };
            let someip_stream = client.find_service(&find_service_opt).await?;
            pin_mut!(someip_stream);

            while let Some(someip_header) = someip_stream.next().await {
                let (someip_header, someip_sd_header) = someip_header?;

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
                    someip_sd_header.reboot,
                    someip_sd_header.unicast,
                    someip_sd_header.explicit_initial_data_control
                );

                println!("\n{}", "Entries:\n".underline());
                for entry in someip_sd_header.entries {
                    display_entry(&entry);
                }

                println!("\n{}", "Options:\n".underline());
                for option in someip_sd_header.options {
                    display_option(&option);
                }
            }
        }
    }
    Ok(())
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
