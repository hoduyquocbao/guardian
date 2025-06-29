use guardian_macros::frame;

#[frame]
pub struct Basic {
    id: u32,
    data: rest,
}

fn main() {
    let data = [0x00, 0x00, 0x00, 0x42, 0x01, 0x02, 0x03];
    let frame = Basic::new(&data).unwrap();
    assert_eq!(frame.id(), 66); // 0x42 in decimal
    assert_eq!(frame.data(), &[0x01, 0x02, 0x03]);
} 