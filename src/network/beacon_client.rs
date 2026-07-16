// use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use tokio::net::TcpStream;

// use crate::protocol::packet::Packet;
// use crate::protocol::peer_id::PeerId;

// pub struct BeaconClient {
//     stream: TcpStream,
//     peer_id: PeerId,
// }

// impl BeaconClient {
//     pub async fn connect(beacon_addr: &str, peer_id: PeerId) -> Result<Self, String> {
//         let mut stream = TcpStream::connect(beacon_addr)
//             .await
//             .map_err(|e| format!("Connect to beacon failed: {}", e))?;

//         // Отправляем свой PeerId для аутентификации
//         stream
//             .write_all(peer_id.as_bytes())
//             .await
//             .map_err(|e| format!("Auth failed: {}", e))?;

//         println!("[BeaconClient] Connected to {} as {}", beacon_addr, peer_id);

//         Ok(Self { stream, peer_id })
//     }

//     pub async fn send_packet(&mut self, packet: &Packet) -> Result<(), String> {
//         let bytes = packet.serialize();
//         let len = (bytes.len() as u32).to_be_bytes();

//         self.stream
//             .write_all(&len)
//             .await
//             .map_err(|e| format!("Write failed: {}", e))?;
//         self.stream
//             .write_all(&bytes)
//             .await
//             .map_err(|e| format!("Write failed: {}", e))?;

//         Ok(())
//     }

//     pub async fn receive_packet(&mut self) -> Result<Packet, String> {
//         let mut len_buf = [0u8; 4];
//         self.stream
//             .read_exact(&mut len_buf)
//             .await
//             .map_err(|e| format!("Read length failed: {}", e))?;

//         let packet_len = u32::from_be_bytes(len_buf) as usize;
//         let mut packet_buf = vec![0u8; packet_len];

//         self.stream
//             .read_exact(&mut packet_buf)
//             .await
//             .map_err(|e| format!("Read packet failed: {}", e))?;

//         Packet::deserialize(&packet_buf)
//     }

//     pub fn peer_id(&self) -> &PeerId {
//         &self.peer_id
//     }
// }
