
    pub trait HasAtomic {
        type Atomic;
    }
    
    impl HasAtomic for u32 {
        type Atomic = core::sync::atomic::AtomicU32;
    }
    
    impl HasAtomic for u16 {
        type Atomic = core::sync::atomic::AtomicU16;
    }
    
    impl HasAtomic for u8 {
        type Atomic = core::sync::atomic::AtomicU8;
    }