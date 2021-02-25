## Proxy干什么？
- 剥离tracker与后端，提供热更新能力，
- 预先处理数据，减少tracker需要考虑的情况，降低tracker代码复杂度
- 方便管控黑名单、站点信息（全站free）
- 在最坏情况下抛弃后端，尽量保证tracker运行正常。

## 预想设计细节
### 几个原则
1. 尽量**只做转发**，避免数据库操作
2. 做尽可能多的log

### 主要流程
- 对于tracker请求
    - bt部分维护TCP长连接给tracker部分
    - pt部分交给backend处理
    - 黑名单：这里要查passkey，不查数据库，用过滤器
        - 对合法用户维护一个filter
        - 对黑名单维护一个？可能会误伤
        - 启动时全量初始化，然后热更新。
        - 允许错误率 10%？可能并不会有太多误伤情况。

一个announce请求模板
```log
[2021-02-16T14:02:13Z INFO  actix_web::middleware::logger] 192.168.31.97:6293 "GET /tracker/announce?info_hash=%08Y%df%97%fb4%60%fa%e6%d4%d9%14d8%b4Moh%d4x&peer_id=-qB4230-3!k*_X5pFXMt&port=1&uploaded=0&downloaded=0&left=7112249065&corrupt=0&key=C22A8AAA&event=started&numwant=200&compact=1&no_peer_id=1&supportcrypto=1&redundant=0 HTTP/1.1" 400 43 "-" "qBittorrent/4.2.3" 0.000520
```

- 对于backend请求
    - 依旧走http直接转发，外部来看应该是透明的

## 控制结构  
- 按scope分服务
- TODO

### config  
没想到太好的方案  
全局挂config_data会被scope遮蔽  
每个scope自己挂会需要各自重写update   
目前几个想法  
- 全部放global里面，要方便维护就写在服务对应目录然后用include导入
- 用反射派发出去然后上个serdejson之类的

## 目前的问题
1. info_hash是urlencode的binary，actix解query用的serde_urlencode，会把不能解成utf-8的换成�（不是乱码，value是65533），目前是专门给info_hash重写了parse，后面看情况fork下来重新改一下
