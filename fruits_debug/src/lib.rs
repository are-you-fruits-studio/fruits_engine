use std::{collections::VecDeque, io::{Read, Write}, net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream}, time::Instant};

use fruits_ecs::{Entity, ExclusiveWorldAccess, ResMut, Resource, Schedule, WorldBuilder};

pub const SYSTEM_GROUP: &str = "fruits_debug";

pub fn add_module_to(world: &mut WorldBuilder) {
    world.data_mut().resources_mut().insert(DebugServerResource::default()).ok().unwrap();
    world.data_mut().resources_mut().insert(DebugConnectionResource::default()).ok().unwrap();

    world.behavior_mut().get_mut(Schedule::Update).group(SYSTEM_GROUP)
        .add_child_system(host_debug_server)
        .add_child_system(debug_connection_recv_system)
        .add_child_system(debug_connection_send_system)
        .add_child_system(generate_response_system)
        .add_child_system(debug_connection_ping_system);

    world.behavior_mut().get_mut(Schedule::Update).order_system(host_debug_server).before_system(debug_connection_recv_system);
    world.behavior_mut().get_mut(Schedule::Update).order_system(host_debug_server).before_system(debug_connection_send_system);
    world.behavior_mut().get_mut(Schedule::Update).order_system(host_debug_server).before_system(debug_connection_ping_system);
    world.behavior_mut().get_mut(Schedule::Update).order_system(debug_connection_recv_system).before_system(generate_response_system);
    world.behavior_mut().get_mut(Schedule::Update).order_system(generate_response_system).before_system(debug_connection_send_system);
}

pub fn add_module_as_client_to(world: &mut WorldBuilder) {
    world.data_mut().resources_mut().insert(DebugServerResource::default()).ok().unwrap();
    world.data_mut().resources_mut().insert(DebugConnectionResource::default()).ok().unwrap();

    world.behavior_mut().get_mut(Schedule::Update).group(SYSTEM_GROUP)
        .add_child_system(debug_connection_recv_system)
        .add_child_system(debug_connection_send_system)
        .add_child_system(debug_connection_ping_system);
}

pub mod msg_types {
    pub const HIERARCHY: u32 = 1;
}

pub fn generate_response_system(
    mut world: ExclusiveWorldAccess,
) {
    let (res, ec, _evt) = world.as_tuple_mut();

    let connection_res = res.get_mut::<DebugConnectionResource>().unwrap();

    let Some(msg) = connection_res.recv_msg_queue.pop_back() else {
        return;
    };
    
    if msg.0 == msg_types::HIERARCHY {
        let mut response = Vec::new();
        for ent in ec.query::<Entity>().iter() {
            let vi = ent.version_index();

            response.extend_from_slice(&(vi.index as u32).to_le_bytes());
            response.extend_from_slice(&(vi.version as u32).to_le_bytes());
        }
        connection_res.send_msg_queue.push_back((msg_types::HIERARCHY, response));
    }
}

pub fn host_debug_server(
    mut server_res: ResMut<DebugServerResource>,
    mut connection_res: ResMut<DebugConnectionResource>,
) {
    if server_res.listener.is_none() {
        let listener = TcpListener::bind(SocketAddrV4::new(
            Ipv4Addr::new(127, 0, 0, 1),
            55643,
        )).unwrap();
        server_res.listener = Some(listener);
    }

    if connection_res.active_stream.is_none() {
        connection_res.reset();

        let listener = server_res.listener.as_ref().unwrap();
        listener.set_nonblocking(true).unwrap();
        let Ok(connection) = listener.accept() else {
            return;
        };

        connection.0.set_nonblocking(true).unwrap();
        connection_res.active_stream = Some(connection.0);
        println!("connected");
    }
}

pub fn debug_connection_ping_system(
    mut connection_res: ResMut<DebugConnectionResource>,
) {
    if connection_res.active_stream.is_none() {
        return;
    };

    let should_ping = match connection_res.last_msg_time {
        Some(last_msg_time) => last_msg_time.elapsed().as_secs_f32() >= 1.0,
        None => true,
    };

    if should_ping {
        connection_res.send_msg_queue.push_back((0, Vec::new()));
    }
}

pub fn debug_connection_recv_system(
    mut connection_res: ResMut<DebugConnectionResource>,
) {
    let Some(stream) = &mut connection_res.active_stream else {
        return;
    };

    let mut buf = [0_u8; 1024];

    let bytes_read = match stream.read(&mut buf) {
        Ok(bytes_read) => bytes_read,
        Err(err) => {
            match err.kind() {
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut => (),
                _ => {
                    connection_res.reset();
                    println!("disconnected");
                },
            }

            return;
        }
    };

    if bytes_read == 0 {
        return;
    }

    connection_res.last_msg_time = Some(Instant::now());

    connection_res.recv_buffer.extend(buf[..bytes_read].into_iter());

    if connection_res.recv_active_msg_metadata.is_none() {
        if connection_res.recv_buffer.len() < 8 {
            return;
        }

        let mut payload_size_bytes = [0_u8; 4];
        for b in payload_size_bytes.iter_mut() {
            *b = connection_res.recv_buffer.pop_front().unwrap();
        }

        let payload_size = u32::from_le_bytes(payload_size_bytes);

        let mut payload_type_bytes = [0_u8; 4];
        for b in payload_type_bytes.iter_mut() {
            *b = connection_res.recv_buffer.pop_front().unwrap();
        }

        let payload_type = u32::from_le_bytes(payload_type_bytes);

        connection_res.recv_active_msg_metadata = Some(DebugMessageMetaData {
            payload_size,
            payload_type,
        });
    }

    let metadata = connection_res.recv_active_msg_metadata.unwrap();

    if connection_res.recv_buffer.len() < metadata.payload_size as usize {
        return;
    }

    let mut msg_buffer = Vec::with_capacity(metadata.payload_size as usize);

    for _ in 0..metadata.payload_size {
        msg_buffer.push(connection_res.recv_buffer.pop_front().unwrap());
    }

    connection_res.recv_active_msg_metadata = None;

    println!("msg received ({}, {}): {:?}", metadata.payload_size, metadata.payload_type, msg_buffer);
    connection_res.recv_msg_queue.push_back((metadata.payload_type, msg_buffer));

}

pub fn debug_connection_send_system(
    mut connection_res: ResMut<DebugConnectionResource>,
) {
    if connection_res.active_stream.is_none() {
        return;
    }

    let Some(msg) = connection_res.send_msg_queue.pop_front() else {
        return;
    };

    let stream = connection_res.active_stream.as_mut().unwrap();
    
    if let Err(err) = stream.write_all(&(msg.1.len() as u32).to_le_bytes()) {
        eprintln!("{}", err);
        connection_res.reset();
        println!("disconnected");
        return;
    }
    if let Err(err) = stream.write_all(&(msg.0).to_le_bytes()) {
        eprintln!("{}", err);
        connection_res.reset();
        println!("disconnected");
        return;
    }
    if let Err(err) = stream.write_all(&msg.1) {
        eprintln!("{}", err);
        connection_res.reset();
        println!("disconnected");
        return;
    }

    stream.flush().unwrap();
    
    connection_res.last_msg_time = Some(Instant::now());
}

#[derive(Resource, Default)]
pub struct DebugServerResource {
    listener: Option<TcpListener>,
}

#[derive(Resource, Default)]
pub struct DebugConnectionResource {
    pub active_stream: Option<TcpStream>,
    pub send_msg_queue: VecDeque<(u32, Vec<u8>)>,
    pub recv_msg_queue: VecDeque<(u32, Vec<u8>)>,
    recv_buffer: VecDeque<u8>,
    recv_active_msg_metadata: Option<DebugMessageMetaData>,
    pub last_msg_time: Option<Instant>,
}

impl DebugConnectionResource {
    pub fn reset(&mut self) {
        self.active_stream = None;
        self.send_msg_queue.clear();
        self.recv_msg_queue.clear();
        self.recv_buffer.clear();
        self.recv_active_msg_metadata = None;
        self.last_msg_time = None;
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct DebugMessageMetaData {
    pub payload_size: u32,
    pub payload_type: u32,
}