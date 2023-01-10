use std::sync::{Arc, Mutex};

use super::Server as BasicServer;
pub struct Server {
    server: Arc<Mutex<BasicServer>>,
}
impl Server {
    pub fn new(port: usize) -> Self {
        Self {
            server: BasicServer::start_server(format!("127.0.0.1:{}", port)),
        }
    }
}
