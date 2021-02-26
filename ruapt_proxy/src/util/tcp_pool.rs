use std::ops::{Deref, DerefMut};

use async_trait::async_trait;
use std::io;
use std::net::SocketAddr;
use tokio::net::TcpStream;

pub type Pool = deadpool::managed::Pool<ConnectionWrapper, io::Error>;
pub type PoolError = deadpool::managed::PoolError<io::Error>;
pub type Connection = deadpool::managed::Object<ConnectionWrapper, io::Error>;
type RecycleResult = deadpool::managed::RecycleResult<io::Error>;

pub struct ConnectionWrapper {
    conn: TcpStream,
}

impl Deref for ConnectionWrapper {
    type Target = TcpStream;
    fn deref(&self) -> &TcpStream {
        &self.conn
    }
}

impl DerefMut for ConnectionWrapper {
    fn deref_mut(&mut self) -> &mut TcpStream {
        &mut self.conn
    }
}

pub struct Manager {
    server: SocketAddr,
}

impl Manager {
    pub fn new(server_addr: &str) -> Result<Self, std::net::AddrParseError> {
        let server = server_addr.parse()?;
        Ok(Self { server })
    }
}

#[async_trait]
impl deadpool::managed::Manager<ConnectionWrapper, io::Error> for Manager {
    async fn create(&self) -> Result<ConnectionWrapper, io::Error> {
        let stream = TcpStream::connect(self.server).await?;
        // keepalive 15 minutes
        stream.set_keepalive(Some(tokio::time::Duration::from_secs(15 * 60)))?;
        Ok(ConnectionWrapper { conn: stream })
    }

    async fn recycle(&self, _conn: &mut ConnectionWrapper) -> RecycleResult {
        Ok(())
    }
}
