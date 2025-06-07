use std::io;
use tokio_util::bytes::{Buf, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

const HEADER_LEN: usize = 4; // 消息长度字段占用4字节
const TYPE_LEN: usize = 4; // MessageType 占用4字节
const MAX_LEN: usize = 8 * 1024 * 1024; // 最大消息长度限制

/// 消息解码器
pub struct MessageDecoder {}

impl Decoder for MessageDecoder {
    type Item = (usize, String); // 返回值为 (message_type, message_body)
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < HEADER_LEN {
            return Ok(None); // 数据不足，等待更多
        }

        // 读取消息总长度（包含 type + body）
        let mut length_bytes = [0u8; HEADER_LEN];
        length_bytes.copy_from_slice(&src[0..HEADER_LEN]);
        let total_length = u32::from_le_bytes(length_bytes) as usize;

        if total_length > MAX_LEN {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Frame of length {} is too large", total_length),
            ));
        }

        // 数据未接收完整，等待更多数据
        if src.len() < HEADER_LEN + total_length {
            src.reserve(HEADER_LEN + total_length - src.len());
            return Ok(None);
        }

        // 跳过 HEADER_LEN 后读取 TYPE_LEN
        let mut type_bytes = [0u8; TYPE_LEN];
        type_bytes.copy_from_slice(&src[HEADER_LEN..HEADER_LEN + TYPE_LEN]);
        let message_type = u32::from_le_bytes(type_bytes) as usize;

        // 提取消息体
        let body_start = HEADER_LEN + TYPE_LEN;
        let body_end = body_start + total_length - TYPE_LEN;
        let data = &src[body_start..body_end];

        let message = String::from_utf8(data.to_vec())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid UTF-8"))?;

        src.advance(body_end); // 移动缓冲区指针

        Ok(Some((message_type, message)))
    }
}

/// 消息编码器
pub struct MessageEncoder {}

impl Encoder<(usize, String)> for MessageEncoder {
    type Error = io::Error;

    fn encode(&mut self, item: (usize, String), dst: &mut BytesMut) -> Result<(), Self::Error> {
        let (message_type, message) = item;

        let msg_len = message.len();
        let total_length = TYPE_LEN + msg_len;

        if msg_len > MAX_LEN {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Frame of length {} is too large", msg_len),
            ));
        }

        // 检查是否超出 u32 范围
        if total_length > u32::MAX as usize {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Message length exceeds u32::MAX",
            ));
        }

        // 写入消息总长度（TYPE_LEN + BODY）
        dst.extend_from_slice(&(total_length as u32).to_le_bytes());

        // 写入消息类型
        dst.extend_from_slice(&(message_type as u32).to_le_bytes());

        // 写入消息内容
        dst.extend_from_slice(message.as_bytes());

        Ok(())
    }
}
