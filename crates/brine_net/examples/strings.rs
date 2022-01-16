use std::{io::Read, io::Write, net::TcpListener};

use bevy::{
    log::{Level, LogPlugin, LogSettings},
    prelude::*,
};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

use brine_net::{codec::StringCodec, NetworkEvent, NetworkPlugin, NetworkResource};

const SERVER: &str = "127.0.0.1:7779";

fn main() {
    let listener = TcpListener::bind(SERVER).unwrap();

    std::thread::spawn(move || echo_server(listener));

    App::new()
        .add_plugins(MinimalPlugins)
        .insert_resource(LogSettings {
            level: Level::DEBUG,
            ..Default::default()
        })
        .add_plugin(LogPlugin)
        .add_plugin(NetworkPlugin::<StringCodec>::default())
        .add_startup_system(connect)
        .add_system(read_net_events)
        .add_system(read_packets)
        .run();
}

fn echo_server(tcp_listener: TcpListener) {
    for stream in tcp_listener.incoming() {
        match stream {
            Ok(mut stream) => loop {
                /*let mut buf = [0u8; 100];
                let size = stream.read(&mut buf).unwrap();
                println!("received: {:?}", &buf[..size]);
                if size == 0 {
                    break;
                }*/

                let len = stream.read_u32::<BigEndian>().unwrap();
                let mut string_bytes = vec![0u8; len as usize];
                stream.read_exact(&mut string_bytes[..]).unwrap();
                let string = std::str::from_utf8(&string_bytes[..]).unwrap();

                println!("Server sees '{}'", string);

                stream.write_u32::<BigEndian>(len).unwrap();
                stream.write_all(string.as_bytes()).unwrap();
                stream.flush().unwrap();
            },
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
    }
}

fn connect(mut net_resource: ResMut<NetworkResource<StringCodec>>) {
    net_resource.connect(SERVER.to_string());
}

fn read_net_events(
    mut reader: EventReader<NetworkEvent<StringCodec>>,
    mut net_resource: ResMut<NetworkResource<StringCodec>>,
) {
    for event in reader.iter() {
        println!("NetworkEvent: {:?}", &event);

        if let NetworkEvent::Connected = event {
            let packet = String::from("hello world");
            net_resource.send_packet(packet);
        }
    }
}

fn read_packets(mut net_resource: ResMut<NetworkResource<StringCodec>>) {
    if let Some(packet) = net_resource.try_recv_packet() {
        println!("Packet received by client: {}", packet);
    }
}
