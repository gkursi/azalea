use uuid::Uuid;

pub trait SerializableUuid {
    fn to_int_array(&self) -> [u32; 4];
    fn from_int_array(array: [u32; 4]) -> Self;
}

// private static int[] leastMostToIntArray(long l, long l2) {
// 	return new int[]{(int)(l >> 32), (int)l, (int)(l2 >> 32), (int)l2};
// }

fn least_most_to_int_array(most: u64, least: u64) -> [u32; 4] {
    [
        (most >> 32) as u32,
        most as u32,
        (least >> 32) as u32,
        least as u32,
    ]
}

impl SerializableUuid for Uuid {
    fn to_int_array(&self) -> [u32; 4] {
        let most_significant_bits = (self.as_u128() >> 64) as u64;
        let least_significant_bits = (self.as_u128() & 0xffffffffffffffff) as u64;

        least_most_to_int_array(most_significant_bits, least_significant_bits)
    }

    fn from_int_array(array: [u32; 4]) -> Self {
        let most = ((array[0] as u64) << 32) | ((array[1] as u64) & 0xFFFFFFFF);
        let least = ((array[2] as u64) << 32) | ((array[3] as u64) & 0xFFFFFFFF);

        Uuid::from_u128((((most as u128) << 64) | least as u128).into())
    }
}

mod tests {
    use super::*;

    #[test]
    fn to_int_array() {
        let u = Uuid::parse_str("6536bfed-8695-48fd-83a1-ecd24cf2a0fd").unwrap();
        assert_eq!(
            u.to_int_array(),
            [0x6536bfed, 0x869548fd, 0x83a1ecd2, 0x4cf2a0fd]
        );
    }

    #[test]
    fn from_int_array() {
        let u = Uuid::from_int_array([0x6536bfed, 0x869548fd, 0x83a1ecd2, 0x4cf2a0fd]);
        assert_eq!(u.to_string(), "6536bfed-8695-48fd-83a1-ecd24cf2a0fd");
    }
}