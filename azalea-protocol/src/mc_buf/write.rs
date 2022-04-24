use async_trait::async_trait;
use azalea_chat::component::Component;
use azalea_core::{
    difficulty::Difficulty, game_type::GameType, resource_location::ResourceLocation,
};
use byteorder::{BigEndian, WriteBytesExt};
use std::io::Write;

use super::MAX_STRING_LENGTH;

#[async_trait]
pub trait Writable {
    fn write_list<F, T>(&mut self, list: &[T], writer: F) -> Result<(), std::io::Error>
    where
        F: FnOnce(&mut Self, &T) -> Result<(), std::io::Error> + Copy,
        T: Sized,
        Self: Sized;
    fn write_int_id_list(&mut self, list: &Vec<i32>) -> Result<(), std::io::Error>;
    fn write_map<KF, VF, KT, VT>(
        &mut self,
        map: Vec<(KT, VT)>,
        key_writer: KF,
        value_writer: VF,
    ) -> Result<(), std::io::Error>
    where
        KF: Fn(&mut Self, KT) -> Result<(), std::io::Error> + Copy,
        VF: Fn(&mut Self, VT) -> Result<(), std::io::Error> + Copy,
        Self: Sized;

    fn write_byte(&mut self, n: u8) -> Result<(), std::io::Error>;
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), std::io::Error>;
    fn write_varint(&mut self, value: i32) -> Result<(), std::io::Error>;
    fn write_utf_with_len(&mut self, string: &str, len: usize) -> Result<(), std::io::Error>;
    fn write_utf(&mut self, string: &str) -> Result<(), std::io::Error>;
    fn write_short(&mut self, n: i16) -> Result<(), std::io::Error>;
    fn write_byte_array(&mut self, bytes: &[u8]) -> Result<(), std::io::Error>;
    fn write_int(&mut self, n: i32) -> Result<(), std::io::Error>;
    fn write_boolean(&mut self, b: bool) -> Result<(), std::io::Error>;
    fn write_nbt(&mut self, nbt: &azalea_nbt::Tag) -> Result<(), std::io::Error>;
    fn write_long(&mut self, n: i64) -> Result<(), std::io::Error>;
    fn write_resource_location(
        &mut self,
        location: &ResourceLocation,
    ) -> Result<(), std::io::Error>;
    fn write_float(&mut self, n: f32) -> Result<(), std::io::Error>;
}

#[async_trait]
impl Writable for Vec<u8> {
    fn write_list<F, T>(&mut self, list: &[T], writer: F) -> Result<(), std::io::Error>
    where
        F: FnOnce(&mut Self, &T) -> Result<(), std::io::Error> + Copy,
        Self: Sized,
    {
        self.write_varint(list.len() as i32)?;
        for item in list {
            writer(self, item)?;
        }
        Ok(())
    }

    fn write_int_id_list(&mut self, list: &Vec<i32>) -> Result<(), std::io::Error> {
        self.write_list(&list, |buf, n| buf.write_varint(*n))
    }

    fn write_map<KF, VF, KT, VT>(
        &mut self,
        map: Vec<(KT, VT)>,
        key_writer: KF,
        value_writer: VF,
    ) -> Result<(), std::io::Error>
    where
        KF: Fn(&mut Self, KT) -> Result<(), std::io::Error> + Copy,
        VF: Fn(&mut Self, VT) -> Result<(), std::io::Error> + Copy,
        Self: Sized,
    {
        self.write_varint(map.len() as i32)?;
        for (key, value) in map {
            key_writer(self, key)?;
            value_writer(self, value)?;
        }
        Ok(())
    }

    fn write_byte(&mut self, n: u8) -> Result<(), std::io::Error> {
        WriteBytesExt::write_u8(self, n)
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), std::io::Error> {
        self.extend_from_slice(bytes);
        Ok(())
    }

    fn write_varint(&mut self, mut value: i32) -> Result<(), std::io::Error> {
        let mut buffer = [0];
        if value == 0 {
            self.write_all(&buffer).unwrap();
        }
        while value != 0 {
            buffer[0] = (value & 0b0111_1111) as u8;
            value = (value >> 7) & (i32::max_value() >> 6);
            if value != 0 {
                buffer[0] |= 0b1000_0000;
            }
            self.write_all(&buffer)?;
        }
        Ok(())
    }

    fn write_utf_with_len(&mut self, string: &str, len: usize) -> Result<(), std::io::Error> {
        if string.len() > len {
            panic!(
                "String too big (was {} bytes encoded, max {})",
                string.len(),
                len
            );
        }
        self.write_varint(string.len() as i32)?;
        self.write_bytes(string.as_bytes())
    }

    fn write_utf(&mut self, string: &str) -> Result<(), std::io::Error> {
        self.write_utf_with_len(string, MAX_STRING_LENGTH.into())
    }

    fn write_short(&mut self, n: i16) -> Result<(), std::io::Error> {
        WriteBytesExt::write_i16::<BigEndian>(self, n)
    }

    fn write_byte_array(&mut self, bytes: &[u8]) -> Result<(), std::io::Error> {
        self.write_varint(bytes.len() as i32)?;
        self.write_bytes(bytes)
    }

    fn write_int(&mut self, n: i32) -> Result<(), std::io::Error> {
        WriteBytesExt::write_i32::<BigEndian>(self, n)
    }

    fn write_boolean(&mut self, b: bool) -> Result<(), std::io::Error> {
        self.write_byte(if b { 1 } else { 0 })
    }

    fn write_nbt(&mut self, nbt: &azalea_nbt::Tag) -> Result<(), std::io::Error> {
        nbt.write(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }

    fn write_long(&mut self, n: i64) -> Result<(), std::io::Error> {
        WriteBytesExt::write_i64::<BigEndian>(self, n)
    }

    fn write_float(&mut self, n: f32) -> Result<(), std::io::Error> {
        WriteBytesExt::write_f32::<BigEndian>(self, n)
    }

    fn write_resource_location(
        &mut self,
        location: &ResourceLocation,
    ) -> Result<(), std::io::Error> {
        self.write_utf(&location.to_string())
    }
}

pub trait McBufWritable
where
    Self: Sized,
{
    fn write_into(&self, buf: &mut Vec<u8>) -> Result<(), std::io::Error>;
}

pub trait McBufVarintWritable
where
    Self: Sized,
{
    fn varint_write_into(&self, buf: &mut Vec<u8>) -> Result<(), std::io::Error>;
}

impl McBufWritable for i32 {
    fn write_into(&self, buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
        Writable::write_int(buf, *self)
    }
}

impl McBufVarintWritable for i32 {
    fn varint_write_into(&self, buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
        buf.write_varint(*self)
    }
}

impl McBufWritable for Vec<u8> {
    fn write_into(&self, buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
        buf.write_bytes(self)
    }
}

// string
impl McBufWritable for String {
    fn write_into(&self, buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
        buf.write_utf(self)
    }
}

// ResourceLocation
impl McBufWritable for ResourceLocation {
    fn write_into(&self, buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
        buf.write_resource_location(self)
    }
}

// u32
impl McBufWritable for u32 {
    fn write_into(&self, buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
        i16::write_into(&(*self as i16), buf)
    }
}

// u32 varint
impl McBufVarintWritable for u32 {
    fn varint_write_into(&self, buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
        i32::varint_write_into(&(*self as i32), buf)
    }
}

// u16
impl McBufWritable for u16 {
    fn write_into(&self, buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
        i16::write_into(&(*self as i16), buf)
    }
}

// u16 varint
impl McBufVarintWritable for u16 {
    fn varint_write_into(&self, buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
        i32::varint_write_into(&(*self as i32), buf)
    }
}

// u8
impl McBufWritable for u8 {
    fn write_into(&self, buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
        buf.write_byte(*self)
    }
}

// i16
impl McBufWritable for i16 {
    fn write_into(&self, buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
        Writable::write_short(buf, *self)
    }
}

// i64
impl McBufWritable for i64 {
    fn write_into(&self, buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
        Writable::write_long(buf, *self)
    }
}

// u64
impl McBufWritable for u64 {
    fn write_into(&self, buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
        i64::write_into(&(*self as i64), buf)
    }
}

// bool
impl McBufWritable for bool {
    fn write_into(&self, buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
        buf.write_boolean(*self)
    }
}

// i8
impl McBufWritable for i8 {
    fn write_into(&self, buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
        buf.write_byte(*self as u8)
    }
}

// f32
impl McBufWritable for f32 {
    fn write_into(&self, buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
        buf.write_float(*self)
    }
}

// GameType
impl McBufWritable for GameType {
    fn write_into(&self, buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
        u8::write_into(&self.to_id(), buf)
    }
}

// Option<GameType>
impl McBufWritable for Option<GameType> {
    fn write_into(&self, buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
        buf.write_byte(GameType::to_optional_id(self) as u8)
    }
}

// Vec<ResourceLocation>
impl McBufWritable for Vec<ResourceLocation> {
    fn write_into(&self, buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
        buf.write_list(self, |buf, resource_location| {
            buf.write_resource_location(resource_location)
        })
    }
}

// azalea_nbt::Tag
impl McBufWritable for azalea_nbt::Tag {
    fn write_into(&self, buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
        buf.write_nbt(self)
    }
}

// Difficulty
impl McBufWritable for Difficulty {
    fn write_into(&self, buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
        u8::write_into(&self.id(), buf)
    }
}

// Component
#[async_trait]
impl McBufWritable for Component {
    // async fn read_into<R>(buf: &mut R) -> Result<Self, String>
    // where
    //     R: AsyncRead + std::marker::Unpin + std::marker::Send,
    // {
    //     let string = buf.read_utf().await?;
    //     let json: serde_json::Value = serde_json::from_str(string.as_str())
    //         .map_err(|e| "Component isn't valid JSON".to_string())?;
    //     let component = Component::deserialize(json).map_err(|e| e.to_string())?;
    //     Ok(component)
    // }
    fn write_into(&self, buf: &mut Vec<u8>) -> Result<(), std::io::Error> {
        // component doesn't have serialize implemented yet
        todo!()
    }
}