use crate::util::tcp_pool::*;

pub struct Context {
    pub pool: Pool,
    // TODO: A connection to backend
    // TODO: monitor, LOGGER are needed
}

impl Context {
    pub fn new() -> Self {
        // TODO: configable
        let m = Manager::new("127.0.0.1:8082").unwrap();
        Context {
            pool: Pool::new(m, 1000),
        }
    }
}
