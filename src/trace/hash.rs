use std::hash::Hasher;

pub(super) struct IdentityHasher(pub(super) u64);

impl Hasher for IdentityHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes.into_iter().rev() {
            self.0 <<= 8;
            self.0 |= u64::from(byte);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::IdentityHasher;
    use std::{
        hash::{Hash, Hasher},
        mem::discriminant,
    };

    #[test]
    fn hashes_nothing() {
        let hasher = IdentityHasher(0);

        assert_eq!(hasher.finish(), 0);
    }

    #[test]
    fn hashes_zero() {
        let mut hasher = IdentityHasher(0);

        hasher.write(&[0]);

        assert_eq!(hasher.finish(), 0);
    }

    #[test]
    fn hashes_single_byte() {
        let mut hasher = IdentityHasher(0);

        hasher.write(&[42]);

        assert_eq!(hasher.finish(), 42);
    }

    #[test]
    fn hashes_multiple_bytes() {
        let mut hasher = IdentityHasher(0);

        hasher.write(&[0x1a, 0xb4]);

        assert_eq!(hasher.finish(), 0xb41a);
    }

    #[test]
    fn hashes_max() {
        let mut hasher = IdentityHasher(0);

        hasher.write(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);

        assert_eq!(hasher.finish(), u64::MAX);
    }

    #[test]
    fn hashes_overflow() {
        let mut hasher = IdentityHasher(0);

        hasher.write(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

        assert_eq!(hasher.finish(), 0x807060504030201);
    }

    // Hashing discriminants is the primary use case of this hasher.
    #[test]
    fn hashes_discriminant() {
        enum Foo {
            Bar = 256,
        }

        let mut hasher = IdentityHasher(0);

        discriminant(&Foo::Bar).hash(&mut hasher);

        assert_eq!(hasher.finish(), 256);
    }
}
