use bytes::{Buf, BufMut, BytesMut};
use prost::Message;
use tokio_im::protobuf::im::ImMessage; // 导入 Protobuf 生成的结构体
use tokio_util::codec::{Decoder, Encoder};

/// Protobuf 编解码器
pub struct ProtobufCodec;

impl ProtobufCodec {
    #[allow(dead_code)]
    pub fn new() -> Self {
        ProtobufCodec {}
    }
}

impl Decoder for ProtobufCodec {
    type Item = ImMessage; // Protobuf 生成的通用消息类型
    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 4 {
            return Ok(None); // 数据不足，等待更多数据
        }

        // 从缓冲区前 4 字节读取消息总长度（小端序）
        let mut length_bytes = [0u8; 4];
        length_bytes.copy_from_slice(&src[..4]);
        let length = u32::from_le_bytes(length_bytes) as usize; // 转换为 usize

        // 当前缓冲区数据不完整，预留空间并返回 None
        if src.len() < 4 + length {
            src.reserve(4 + length - src.len()); // 预留足够空间
            return Ok(None); // 等待更多数据
        }

        // 提取消息体
        let data = &src[4..4 + length].to_vec();
        src.advance(4 + length); // 移动缓冲区指针，跳过已处理的数据

        // 使用 prost 反序列化为 ImMessage
        match ImMessage::decode(data.as_slice()) {
            Ok(msg) => Ok(Some(msg)),
            Err(e) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            )),
        }
    }
}

impl Encoder<ImMessage> for ProtobufCodec {
    type Error = std::io::Error;

    fn encode(&mut self, item: ImMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // 使用 Protobuf 的 Message trait 将结构体序列化为 Vec<u8>
        let mut buf = Vec::with_capacity(item.encoded_len());
        item.encode(&mut buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?; // 错误处理

        // 预留消息总长度的空间
        let len = buf.len();
        dst.reserve(4 + len);

        // 写入消息总长度（小端序）
        dst.put_u32_le(len as u32);

        // 写入消息体（Protobuf编码）
        dst.extend_from_slice(&buf);

        Ok(())
    }
}
