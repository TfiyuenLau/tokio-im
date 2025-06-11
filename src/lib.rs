// 引入构建的protobuf模块
pub mod protobuf {
    pub mod im {
        include!(concat!(env!("OUT_DIR"), "/im.protobuf.rs"));
    }
}
