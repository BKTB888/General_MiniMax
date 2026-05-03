pub macro splitmix {
    ($x:expr, $mac:ident!()) => {
        $crate::tt_call::tt_call! {
            macro = [{ $mac }]
            ~~> $crate::mixers::splitmix_with_type! {
                x = [{ $x }]
            }
        }
    },
    ($x:expr, u32) => {
        splitmix!($x, u32, 0x9E37_79B9_u32, 16, 0x85EB_CA6B, 13, 0xC2B2_AE35, 16)
    },
    ($x:expr, u64) => {
        splitmix!($x, u64, 0x9E37_79B9_7F4A_7C15_u64, 30, 0xBF58_476D_1CE4_E7B5, 27, 0x94D0_49BB_1331_11EB, 31)
    },
    ($x:expr, $t:ty, $g:expr, $s1:expr, $m1:expr, $s2:expr, $m2:expr, $s3:expr) => {{
        let mut x: $t = $x;
        x = x.wrapping_add($g);
        x = (x ^ x >> $s1).wrapping_mul($m1);
        x = (x ^ x >> $s2).wrapping_mul($m2);
        x ^ x >> $s3
    }}
}

pub macro xorshift {
    ($x:expr, $mac:ident!()) => {
        $crate::tt_call::tt_call! {
            macro = [{ $mac }]
            ~~> $crate::mixers::xorshift_with_type! {
                x = [{ $x }]
            }
        }
    },
    ($x:expr, u16) => { xorshift!($x, u16, 7, 9, 8) },
    ($x:expr, u32) => { xorshift!($x, u32, 13, 17, 5) },
    ($x:expr, u64) => { xorshift!($x, u64, 13, 7, 17) },
    ($x:expr, u128) => { xorshift!($x, u128, 23, 17, 26) },
    ($x:expr, $t:ty, $a:expr, $b:expr, $c:expr) => {{
        let mut x: $t = $x;
        x ^= x << $a;
        x ^= x >> $b;
        x ^= x << $c;
        x
    }}
}

macro xorshift_with_type {
    {
        x = [{ $x:expr }]
        type = [{ $t:tt }]
    } => {
        $crate::mixers::xorshift!($x, $t)
    }
}
macro splitmix_with_type {
    {
        x = [{ $x:expr }]
        type = [{ $t:tt }]
    } => {
        $crate::mixers::splitmix!($x, $t)
    }
}

#[cfg(test)]
mod tests {
    use super::splitmix;
    const _: u32 = splitmix!(0, u32);

    #[test]
    fn splitmix_zero_seed_is_nonzero() {
        assert_ne!(splitmix!(0, u32), 0);
        assert_ne!(splitmix!(0, u64), 0);
    }
}
