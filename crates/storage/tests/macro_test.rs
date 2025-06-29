use guardian_macros::frame;

#[frame]
pub struct TestFrame {
    id: u32,
    data: rest,
}

#[test]
fn test_frame_macro() {
    // Test data: id=1234567890, data=[1,2,3]
    let data = [
        0x49, 0x96, 0x02, 0xD2, // u32: 1234567890
        0x01, 0x02, 0x03, // rest data
    ];
    
    let frame = TestFrame::new(&data).unwrap();
    assert_eq!(frame.id(), 1234567890);
    assert_eq!(frame.data(), &[0x01, 0x02, 0x03]);
} 