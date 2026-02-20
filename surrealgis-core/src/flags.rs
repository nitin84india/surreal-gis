use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct GeometryFlags: u8 {
        const HAS_Z    = 0b0000_0001;
        const HAS_M    = 0b0000_0010;
        const IS_EMPTY = 0b0000_0100;
        const HAS_BBOX = 0b0000_1000;
        const HAS_SRID = 0b0001_0000;
    }
}

impl Default for GeometryFlags {
    fn default() -> Self {
        GeometryFlags::empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_flags_are_empty() {
        let flags = GeometryFlags::default();
        assert!(flags.is_empty());
    }

    #[test]
    fn can_set_and_check_flags() {
        let flags = GeometryFlags::HAS_Z | GeometryFlags::HAS_SRID;
        assert!(flags.contains(GeometryFlags::HAS_Z));
        assert!(flags.contains(GeometryFlags::HAS_SRID));
        assert!(!flags.contains(GeometryFlags::HAS_M));
    }

    #[test]
    fn flags_clone_and_eq() {
        let flags = GeometryFlags::HAS_BBOX;
        let cloned = flags;
        assert_eq!(flags, cloned);
    }
}
