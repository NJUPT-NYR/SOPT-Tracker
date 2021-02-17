mod client;

pub fn default_num_want() -> u16 {
    50
}

pub fn accept_client_list() -> Vec<client::Client> {
    vec![]
}


struct Configuration {}

impl Configuration {
    fn new() -> Self {
        Configuration {}
    }
}
