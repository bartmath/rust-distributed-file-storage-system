use serde::{Serialize, Deserialize};

trait TypeId {
    const ID: u8;
}

// Test message types (must implement Serialize/Deserialize)
#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TestMsg1 {
    id: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TestMsg2 {
    name: String,
}

storage_macros::register_types! {
    TestMsg1,
    TestMsg2,
}

#[test]
fn test_register_messages_generates_impls() {
    // Verify const IDs are correct (0, 1, ...)
    assert_eq!(TestMsg1::ID, 0u8);
    assert_eq!(TestMsg2::ID, 1u8);

    // Verify traits are implemented
    fn requires_message_id<T: TypeId>() {}
    requires_message_id::<TestMsg1>();
    requires_message_id::<TestMsg2>();
}