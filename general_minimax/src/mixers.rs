pub const trait Splitmix {
    fn splitmix(self) -> Self;
}

macro_rules! impl_splitmix {
    ($($t:ty: $g:expr, $s1:expr, $m1:expr, $s2:expr, $m2:expr, $s3:expr;)*) => {$(
        const impl Splitmix for $t {
            fn splitmix(self) -> Self {
                let mut x = self.wrapping_add($g);
                x = (x ^ x >> $s1).wrapping_mul($m1);
                x = (x ^ x >> $s2).wrapping_mul($m2);
                x ^ x >> $s3
            }
        }
    )*};
}
impl_splitmix! {
    u32: 0x9E37_79B9, 16, 0x85EB_CA6B, 13, 0xC2B2_AE35, 16;
    u64: 0x9E37_79B9_7F4A_7C15, 30, 0xBF58_476D_1CE4_E7B5, 27, 0x94D0_49BB_1331_11EB, 31;
}

pub const trait Xorshift {
    fn xorshift(self) -> Self;
}

macro_rules! impl_xorshift {
    ($($t:ty: $a:expr, $b:expr, $c:expr;)*) => {$(
        const impl Xorshift for $t {
            fn xorshift(self) -> Self {
                let mut x = self;
                x ^= x << $a;
                x ^= x >> $b;
                x ^= x << $c;
                x
            }
        }
    )*};
}
impl_xorshift! {
    u16: 7, 9, 8;
    u32: 13, 17, 5;
    u64: 13, 7, 17;
    u128: 23, 17, 26;
}

#[cfg(test)]
mod tests {
    use super::Splitmix;
    const _: u32 = 0u32.splitmix();

    #[test]
    fn splitmix_zero_seed_is_nonzero() {
        assert_ne!(0u32.splitmix(), 0);
        assert_ne!(0u64.splitmix(), 0);
    }
}
