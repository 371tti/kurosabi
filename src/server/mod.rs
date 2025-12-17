/// mod server では TCPレイヤーからのサーバー実装が行われます
/// 
/// kurosabiはその柔軟性を維持するためにTCPレイヤーで可能な限り開発者が直接操作可能なメソッドを提供することを目指す
/// TCPレイヤーを最も下とする これはOS上で動くことを想定しているから
/// 
/// 


pub struct TCPListener {
    pub listener: compio::net::TcpListener,
}

pub struct TCPConnection {
    pub stream: compio::net::TcpStream,
}

pub struct ConnectionQueue {
    
}

pub struct HTTP {}