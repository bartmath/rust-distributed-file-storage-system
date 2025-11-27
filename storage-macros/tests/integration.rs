use serde::{Serialize, Deserialize};
use bincode::Options;
use anyhow;

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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TestMsg3 {
    vect: Vec<u8>,
}

storage_macros::register_types! {
    TestMsg1,
    TestMsg2,
    TestMsg3,
}

#[test]
fn test_register_messages_generates_impls() {
    // Verify const IDs are correct (0, 1, ...)
    assert_eq!(TestMsg1::ID, 0u8);
    assert_eq!(TestMsg2::ID, 1u8);
    assert_eq!(TestMsg3::ID, 2u8);

    // Verify traits are implemented
    fn requires_message_id<T: TypeId>() {}
    requires_message_id::<TestMsg1>();
    requires_message_id::<TestMsg2>();
    requires_message_id::<TestMsg3>();

    fn requires_serialization<T: Serialize>() {}
    requires_serialization::<TestMsg1>();
    requires_serialization::<TestMsg2>();
    requires_serialization::<TestMsg3>();
}