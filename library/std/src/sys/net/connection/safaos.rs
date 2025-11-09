use core::net::SocketAddrV4;
use core::num::NonZero;

use safa_api::abi::sockets::{InetV4SocketAddr, SockMsgFlags, ToSocketAddr};
use safa_api::errors::ErrorStatus;
use safa_api::sockets::socket::SocketOpt;
use safa_api::sockets::{Socket, SocketDomain, SocketKind};

use crate::fmt;
use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut};
use crate::net::{Ipv4Addr, Ipv6Addr, Shutdown, SocketAddr};
use crate::sync::RwLock;
use crate::sys::unsupported;
use crate::time::Duration;

pub struct TcpStream(!);

impl TcpStream {
    pub fn connect(_: io::Result<&SocketAddr>) -> io::Result<TcpStream> {
        unsupported()
    }

    pub fn connect_timeout(_: &SocketAddr, _: Duration) -> io::Result<TcpStream> {
        unsupported()
    }

    pub fn set_read_timeout(&self, _: Option<Duration>) -> io::Result<()> {
        self.0
    }

    pub fn set_write_timeout(&self, _: Option<Duration>) -> io::Result<()> {
        self.0
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        self.0
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        self.0
    }

    pub fn peek(&self, _: &mut [u8]) -> io::Result<usize> {
        self.0
    }

    pub fn read(&self, _: &mut [u8]) -> io::Result<usize> {
        self.0
    }

    pub fn read_buf(&self, _buf: BorrowedCursor<'_>) -> io::Result<()> {
        self.0
    }

    pub fn read_vectored(&self, _: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        self.0
    }

    pub fn is_read_vectored(&self) -> bool {
        self.0
    }

    pub fn write(&self, _: &[u8]) -> io::Result<usize> {
        self.0
    }

    pub fn write_vectored(&self, _: &[IoSlice<'_>]) -> io::Result<usize> {
        self.0
    }

    pub fn is_write_vectored(&self) -> bool {
        self.0
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.0
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        self.0
    }

    pub fn shutdown(&self, _: Shutdown) -> io::Result<()> {
        self.0
    }

    pub fn duplicate(&self) -> io::Result<TcpStream> {
        self.0
    }

    pub fn set_linger(&self, _: Option<Duration>) -> io::Result<()> {
        self.0
    }

    pub fn linger(&self) -> io::Result<Option<Duration>> {
        self.0
    }

    pub fn set_nodelay(&self, _: bool) -> io::Result<()> {
        self.0
    }

    pub fn nodelay(&self) -> io::Result<bool> {
        self.0
    }

    pub fn set_ttl(&self, _: u32) -> io::Result<()> {
        self.0
    }

    pub fn ttl(&self) -> io::Result<u32> {
        self.0
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.0
    }

    pub fn set_nonblocking(&self, _: bool) -> io::Result<()> {
        self.0
    }
}

impl fmt::Debug for TcpStream {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0
    }
}

pub struct TcpListener(!);

impl TcpListener {
    pub fn bind(_: io::Result<&SocketAddr>) -> io::Result<TcpListener> {
        unsupported()
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        self.0
    }

    pub fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        self.0
    }

    pub fn duplicate(&self) -> io::Result<TcpListener> {
        self.0
    }

    pub fn set_ttl(&self, _: u32) -> io::Result<()> {
        self.0
    }

    pub fn ttl(&self) -> io::Result<u32> {
        self.0
    }

    pub fn set_only_v6(&self, _: bool) -> io::Result<()> {
        self.0
    }

    pub fn only_v6(&self) -> io::Result<bool> {
        self.0
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.0
    }

    pub fn set_nonblocking(&self, _: bool) -> io::Result<()> {
        self.0
    }
}

impl fmt::Debug for TcpListener {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0
    }
}

use safa_api::abi::sockets::SocketAddr as AbiSocketAddr;

enum ApiSocketAddr {
    Ipv4(InetV4SocketAddr),
}

impl ApiSocketAddr {
    fn as_ref(&self) -> (&AbiSocketAddr, usize) {
        match self {
            ApiSocketAddr::Ipv4(addr) => (addr.as_generic(), size_of::<InetV4SocketAddr>()),
        }
    }
}

fn socket_addr_to_api(addr: &SocketAddr) -> io::Result<(SocketDomain, ApiSocketAddr)> {
    match addr {
        SocketAddr::V4(addr) => Ok((
            SocketDomain::Ipv4,
            ApiSocketAddr::Ipv4(InetV4SocketAddr::new(addr.port(), *addr.ip())),
        )),
        SocketAddr::V6(_) => unsupported(),
    }
}

fn api_addr_to_socket_addr(addr: ApiSocketAddr) -> SocketAddr {
    match addr {
        ApiSocketAddr::Ipv4(addr) => SocketAddr::V4(SocketAddrV4::new(addr.ip(), addr.port())),
    }
}

pub struct UdpSocket {
    inner: Socket,
    // TODO: get peer and local from the socket using syscalls?
    peer: RwLock<Option<SocketAddr>>,
    local: SocketAddr,
}

impl UdpSocket {
    pub fn bind(addr: io::Result<&SocketAddr>) -> io::Result<UdpSocket> {
        let addr = addr?;
        let (domain, api_addr) = socket_addr_to_api(addr)?;
        let kind = SocketKind::Datagram;
        let socket = Socket::builder(domain, kind, 0).build()?;

        let (addr_ref, addr_len) = api_addr.as_ref();
        socket.bind(addr_ref, addr_len)?;
        Ok(UdpSocket { inner: socket, peer: RwLock::new(None), local: *addr })
    }

    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.peer
            .read()
            .unwrap()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "not connected"))
    }

    pub fn socket_addr(&self) -> io::Result<SocketAddr> {
        Ok(self.local)
    }

    #[inline]
    fn recv_from_inner(
        &self,
        flags: SockMsgFlags,
        buf: &mut [u8],
    ) -> io::Result<(usize, SocketAddr)> {
        let mut addr = InetV4SocketAddr::new(0, Ipv4Addr::from_bits(0));
        let mut recvied_from = (addr.as_non_null(), size_of::<InetV4SocketAddr>());
        let received = self.inner.recv_from(buf, flags, &mut recvied_from)?;

        let (_, addr_len) = recvied_from;
        assert_eq!(
            addr_len,
            size_of::<InetV4SocketAddr>(),
            "Recv from on a UDP socket should always return a valid address"
        );

        Ok((received, api_addr_to_socket_addr(ApiSocketAddr::Ipv4(addr))))
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.recv_from_inner(SockMsgFlags::NONE, buf)
    }

    pub fn peek_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        self.recv_from_inner(SockMsgFlags::PEEK, buf)
    }

    pub fn send_to(&self, buf: &[u8], target: &SocketAddr) -> io::Result<usize> {
        let (_, api_addr) = socket_addr_to_api(target)?;
        Ok(self.inner.send_to(buf, SockMsgFlags::NONE, Some(api_addr.as_ref()))?)
    }

    pub fn duplicate(&self) -> io::Result<UdpSocket> {
        todo!("actual resource duplication...")
    }

    pub fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        let am = dur.map(|dur| dur.as_millis() as u64).unwrap_or(0);
        self.inner.set_sock_opt(SocketOpt::ReadTimeout, am)?;
        Ok(())
    }

    pub fn set_write_timeout(&self, dur: Option<Duration>) -> io::Result<()> {
        let am = dur.map(|dur| dur.as_millis() as u64).unwrap_or(0);
        self.inner.set_sock_opt(SocketOpt::WriteTimeout, am)?;
        Ok(())
    }

    pub fn read_timeout(&self) -> io::Result<Option<Duration>> {
        let mut am: u64 = 0;
        unsafe {
            self.inner.get_sock_opt(SocketOpt::ReadTimeout, &mut am)?;
        }
        let dur = NonZero::new(am).map(|n| Duration::from_millis(n.get()));
        Ok(dur)
    }

    pub fn write_timeout(&self) -> io::Result<Option<Duration>> {
        let mut am: u64 = 0;
        unsafe {
            self.inner.get_sock_opt(SocketOpt::WriteTimeout, &mut am)?;
        }
        let dur = NonZero::new(am).map(|n| Duration::from_millis(n.get()));
        Ok(dur)
    }

    pub fn set_broadcast(&self, can: bool) -> io::Result<()> {
        self.inner.set_sock_opt(SocketOpt::IpBroadcast, can)?;
        Ok(())
    }

    pub fn broadcast(&self) -> io::Result<bool> {
        let mut can = false;
        unsafe {
            self.inner.get_sock_opt(SocketOpt::IpBroadcast, &mut can)?;
        }
        Ok(can)
    }

    pub fn set_multicast_loop_v4(&self, _: bool) -> io::Result<()> {
        unsupported()
    }

    pub fn multicast_loop_v4(&self) -> io::Result<bool> {
        unsupported()
    }

    pub fn set_multicast_ttl_v4(&self, _: u32) -> io::Result<()> {
        unsupported()
    }

    pub fn multicast_ttl_v4(&self) -> io::Result<u32> {
        unsupported()
    }

    pub fn set_multicast_loop_v6(&self, _: bool) -> io::Result<()> {
        unsupported()
    }

    pub fn multicast_loop_v6(&self) -> io::Result<bool> {
        unsupported()
    }

    pub fn join_multicast_v4(&self, _: &Ipv4Addr, _: &Ipv4Addr) -> io::Result<()> {
        unsupported()
    }

    pub fn join_multicast_v6(&self, _: &Ipv6Addr, _: u32) -> io::Result<()> {
        unsupported()
    }

    pub fn leave_multicast_v4(&self, _: &Ipv4Addr, _: &Ipv4Addr) -> io::Result<()> {
        unsupported()
    }

    pub fn leave_multicast_v6(&self, _: &Ipv6Addr, _: u32) -> io::Result<()> {
        unsupported()
    }

    pub fn set_ttl(&self, ttl: u32) -> io::Result<()> {
        self.inner.set_sock_opt(SocketOpt::IpTTL, ttl)?;
        Ok(())
    }

    pub fn ttl(&self) -> io::Result<u32> {
        let mut ttl: u32 = 0;
        unsafe { self.inner.get_sock_opt(SocketOpt::IpTTL, &mut ttl)? };
        Ok(ttl)
    }

    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        let mut err: u16 = 0;
        unsafe {
            self.inner.get_sock_opt(SocketOpt::SocketError, &mut err)?;
        }
        if err == 0 { Ok(None) } else { Ok(Some(ErrorStatus::from_u16(err).into())) }
    }

    pub fn set_nonblocking(&self, non_blocking: bool) -> io::Result<()> {
        Ok(self.inner.set_blocking(!non_blocking)?)
    }

    pub fn recv(&self, buf: &mut [u8]) -> io::Result<usize> {
        Ok(self.inner.recv(buf, SockMsgFlags::NONE)?)
    }

    pub fn peek(&self, buf: &mut [u8]) -> io::Result<usize> {
        Ok(self.inner.recv(buf, SockMsgFlags::PEEK)?)
    }

    pub fn send(&self, buf: &[u8]) -> io::Result<usize> {
        Ok(self.inner.send(buf, SockMsgFlags::NONE)?)
    }

    pub fn connect(&self, addr: io::Result<&SocketAddr>) -> io::Result<()> {
        let peer_addr = addr?;
        let (_, api_addr) = socket_addr_to_api(peer_addr)?;
        let (addr_ref, addr_len) = api_addr.as_ref();
        self.inner.connect(addr_ref, addr_len)?;
        *self.peer.write().unwrap() = Some(*peer_addr);
        Ok(())
    }
}

impl fmt::Debug for UdpSocket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(UdpSocket))
            .field("ri", &self.inner.ri())
            .field("addr", &self.socket_addr().expect("Shouldn't fail"))
            .finish()
    }
}

use safa_api::net::AddrHints;
use safa_api::net::AddrInfo;
use safa_api::net::LookupError;
use safa_api::net::lookup_addr_info;

#[stable(feature = "rust1", since = "1.0.0")]
impl From<LookupError> for io::Error {
    fn from(err: LookupError) -> io::Error {
        match err {
            LookupError::System(e) => e.into(),
            LookupError::InvalidFamily => unreachable!("Invalid Family given to lookup_addr_info"),
            LookupError::NoSuchNode | LookupError::NoSuchService => {
                io::const_error!(io::ErrorKind::InvalidInput, "DNS couldn't resolve host name")
            }
            LookupError::ServerRefused => {
                io::const_error!(io::ErrorKind::Uncategorized, "DNS Server Refused responding")
            }
            LookupError::NoData => {
                io::const_error!(io::ErrorKind::Uncategorized, "DNS No Data for given host name")
            }
            LookupError::TemporaryFailure => {
                io::const_error!(io::ErrorKind::HostUnreachable, "DNS Temporary Failure")
            }
        }
    }
}

pub struct LookupHost {
    current: Option<AddrInfo>,
    port: u16,
}

impl LookupHost {
    pub fn port(&self) -> u16 {
        self.port
    }
}

impl Iterator for LookupHost {
    type Item = SocketAddr;
    fn next(&mut self) -> Option<SocketAddr> {
        match self.current.take() {
            Some(mut info) => {
                self.current = info.take_next();
                Some(info.ip_socket_addr())
            }
            None => None,
        }
    }
}

impl TryFrom<&str> for LookupHost {
    type Error = io::Error;

    fn try_from(s: &str) -> io::Result<LookupHost> {
        macro_rules! try_opt {
            ($e:expr, $msg:expr) => {
                match $e {
                    Some(r) => r,
                    None => return Err(io::const_error!(io::ErrorKind::InvalidInput, $msg)),
                }
            };
        }

        // split the string by ':' and convert the second part to u16
        let (host, port_str) = try_opt!(s.rsplit_once(':'), "invalid socket address");
        let port: u16 = try_opt!(port_str.parse().ok(), "invalid port value");
        (host, port).try_into()
    }
}

impl<'a> TryFrom<(&'a str, u16)> for LookupHost {
    type Error = io::Error;

    fn try_from((node, service): (&'a str, u16)) -> io::Result<LookupHost> {
        let hints = AddrHints::new(Some(SocketKind::Stream), None, 0);
        lookup_addr_info(Some(node), None, Some(&hints))
            .map(|info| LookupHost { current: Some(info), port: service })
            .map_err(|e| e.into())
    }
}
