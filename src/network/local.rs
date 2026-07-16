//! ## Local Discovery / Локальное обнаружение
//!
//! Discovers local IP addresses (loopback + LAN).
//! / Обнаруживает локальные IP-адреса (loopback + LAN).

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub struct LocalDiscovery;

impl LocalDiscovery {
    /// Returns all local addresses (loopback + LAN).
    /// / Возвращает все локальные адреса (loopback + LAN).
    pub fn get_local_addrs(port: u16) -> Vec<SocketAddr> {
        let mut addrs = Vec::new();

        // Loopback / Локальный адрес
        addrs.push(SocketAddr::from(([127, 0, 0, 1], port)));

        // LAN IP / LAN IP
        if let Some(ip) = Self::guess_lan_ip() {
            addrs.push(SocketAddr::from((ip, port)));
        }

        addrs
    }

    /// Guesses LAN IP using UDP trick (connects to 8.8.8.8:80).
    /// / Угадывает LAN IP через UDP-трюк (подключается к 8.8.8.8:80).
    fn guess_lan_ip() -> Option<Ipv4Addr> {
        use std::net::UdpSocket;

        let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
        socket.connect("8.8.8.8:80").ok()?;

        let local = socket.local_addr().ok()?;
        match local.ip() {
            IpAddr::V4(ip) if !ip.is_loopback() => Some(ip),
            _ => None,
        }
    }

    /// Checks if an address is local (private or loopback).
    /// / Проверяет, является ли адрес локальным (приватным или loopback).
    pub fn is_local(addr: &SocketAddr) -> bool {
        match addr.ip() {
            IpAddr::V4(ip) => ip.is_private() || ip.is_loopback(),
            IpAddr::V6(ip) => ip.is_loopback(),
        }
    }
}
