use guardian_macros::frame;

#[frame]
pub struct Invalid {
    id: u32,
    invalid: MyCustomType, // This should cause a compilation error
    data: rest,
} 