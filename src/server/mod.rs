

/// mod server では TCPレイヤーからのサーバー実装が行われます
/// 
/// kurosabiはその柔軟性を維持するためにTCPレイヤーで可能な限り開発者が直接操作可能なメソッドを提供することを目指す
/// TCPレイヤーを最も下とする これはOS上で動くことを想定しているから
/// 
/// 



pub struct Ctx<C> {
    pub c: C,
}


pub struct KurosabiRouterBuilder {

}

/// Kurosabi Server の設定
pub struct KurosabiServerBuilder {

}