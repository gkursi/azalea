use crate::mc_buf::UnsizedByteArray;
use azalea_core::resource_location::ResourceLocation;
use packet_macros::LoginPacket;
use std::hash::Hash;

#[derive(Hash, Clone, Debug, LoginPacket)]
pub struct ClientboundCustomQueryPacket {
    #[var]
    pub transaction_id: u32,
    pub identifier: ResourceLocation,
    pub data: UnsizedByteArray,
}
