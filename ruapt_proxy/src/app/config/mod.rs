mod client;

pub fn default_num_want() -> isize {
    50
}

pub fn accept_client_list() -> Vec<client::Client> {
    vec![]
}


/// 将所有的配置项放在这个结构里应该是不合理的，但是为每个sub服务添加专门的config
/// 更加不合理，因为这会导致update代码重复
///
/// 现在有几个想法
/// - 使用include引入不同服务的cfg
/// - 用enum引入
struct Configuration {}

impl Configuration {
    fn new() -> Self {
        Configuration {}
    }
}
