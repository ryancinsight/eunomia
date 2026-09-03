use eunomia::layout::{bytes_of, cast_slice, cast_slice_mut};
use eunomia::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
struct DeviceParameters {
    count: u32,
    scale: f32,
    lanes: [u16; 2],
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
struct DeviceIndex(u32);

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
struct GenericParameters<T> {
    first: T,
    second: T,
}

#[test]
fn derives_provide_zeroed_and_byte_views() {
    let zero = DeviceParameters::zeroed();
    assert_eq!(
        zero,
        DeviceParameters {
            count: 0,
            scale: 0.0,
            lanes: [0, 0],
        }
    );

    let mut values = [DeviceIndex(7), DeviceIndex(11)];
    let bytes = cast_slice::<DeviceIndex, u8>(&values);
    assert_eq!(bytes.len(), core::mem::size_of_val(&values));
    assert_eq!(
        bytes_of(&values[0]).len(),
        core::mem::size_of::<DeviceIndex>()
    );

    let words = cast_slice_mut::<DeviceIndex, u32>(&mut values);
    words.copy_from_slice(&[13, 17]);
    assert_eq!(values, [DeviceIndex(13), DeviceIndex(17)]);

    assert_eq!(GenericParameters::<u32>::zeroed().first, 0);
}
