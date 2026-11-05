use eunomia::{Bf4, Packed4Cow, Packed4Slice, Packed4Vec, F4};

fn codes(vec: &Packed4Vec<Bf4>) -> Vec<u8> {
    (0..vec.len())
        .map(|index| vec.get(index).expect("index below length").0)
        .collect()
}

fn check_appends(mut vec: Packed4Vec<Bf4>, mut expected: Vec<u8>) {
    assert_eq!(codes(&vec), expected);
    for code in [14, 11, 2] {
        vec.push(Bf4(code));
        expected.push(code);
        assert_eq!(codes(&vec), expected);
        assert_eq!(vec.as_packed_slice().len(), vec.len().div_ceil(2));
    }
}

#[test]
fn borrowed_cow_with_excess_backing_appends_at_logical_end() {
    let backing = [0x10, 0x32, 0x54, 0x76, 0x98, 0xba];
    for len in 0..=5_usize {
        for excess in 1..=3 {
            let bytes = &backing[..len.div_ceil(2) + excess];
            let view = Packed4Slice::<Bf4>::new(bytes, len).expect("enough backing bytes");
            let expected =
                (0..u8::try_from(len).expect("length is at most five")).collect::<Vec<_>>();
            check_appends(Packed4Cow::borrowed(view).into_owned(), expected.clone());
            let mut cow = Packed4Cow::borrowed(view);
            let owned = core::mem::take(cow.to_mut());
            check_appends(owned, expected);
        }
    }
}

#[test]
fn f4_copy_on_write_preserves_every_code_and_appends() {
    let backing = [0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc, 0xfe, 0xa5, 0x5a];
    for len in 0..=16_usize {
        for excess in 0..=2 {
            let bytes = &backing[..len.div_ceil(2) + excess];
            let view = Packed4Slice::<F4>::new(bytes, len).expect("enough backing bytes");
            assert_eq!(view.as_packed_slice(), bytes);
            let mut cow = Packed4Cow::borrowed(view);
            let mut expected =
                (0..u8::try_from(len).expect("length is at most sixteen")).collect::<Vec<_>>();
            let owned = cow.to_mut();
            assert_eq!(owned.as_packed_slice().len(), len.div_ceil(2));
            for code in 0..16_u8 {
                owned.push(F4(code));
                expected.push(code);
                let actual = (0..owned.len())
                    .map(|index| owned.get(index).expect("index below length").0)
                    .collect::<Vec<_>>();
                assert_eq!(actual, expected);
                assert_eq!(owned.as_packed_slice().len(), owned.len().div_ceil(2));
            }
        }
    }
}

#[cfg(feature = "rkyv")]
#[test]
fn archived_borrowed_cow_with_excess_backing_appends_at_logical_end() {
    use rkyv::rancor::Error;

    let backing = [0x10, 0x32, 0x54, 0x76, 0x98, 0xba];
    for len in 0..=5_usize {
        for excess in 1..=3 {
            let view = Packed4Slice::<Bf4>::new(&backing[..len.div_ceil(2) + excess], len)
                .expect("enough backing bytes");
            let archive = rkyv::to_bytes::<Error>(&Packed4Cow::borrowed(view)).expect("serialize");
            let restored: Packed4Cow<'static, Bf4> =
                rkyv::from_bytes::<_, Error>(&archive).expect("validated deserialize");
            check_appends(
                restored.into_owned(),
                (0..u8::try_from(len).expect("length is at most five")).collect(),
            );
        }
    }
}
