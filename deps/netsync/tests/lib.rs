extern crate netsync;

use state::{Input, Value, ClientState, ServerState};


// Mocks ----------------------------------------------------------------------
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct TestInput {
    tick: u8,
    buttons: u8
}

impl TestInput {
    fn new(tick: u8, buttons: u8) -> TestInput {
        TestInput {
            tick: tick,
            buttons: buttons
        }
    }
}

impl Input for TestInput {

    fn tick(&self) -> u8 {
        self.tick
    }

    fn to_bytes(&self) -> Vec<u8> {
        vec![self.tick, self.buttons]
    }

    fn from_bytes(bytes: &[u8]) -> Option<(usize, Self)> where Self: Sized {
        if bytes.len() >= 2 {
            Some((2, TestInput {
                tick: bytes[0],
                buttons: bytes[1]
            }))

        } else {
            None
        }
    }

}

#[derive(Debug, Eq, PartialEq, Clone, Default)]
struct TestValue {
    x: u8,
    y: u8
}

impl Value for TestValue {

    fn interpolate_from(&self, last: &Self, u: f64) -> Self {
        let dx = self.x as f32 - last.x as f32;
        let dy = self.y as f32 - last.y as f32;
        TestValue {
            x: (last.x as f32 + (dx * u as f32).floor()) as u8,
            y: (last.y as f32 + (dy * u as f32).floor()) as u8
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        vec![self.x, self.y]
    }

    fn from_bytes(bytes: &[u8]) -> Self where Self: Sized {
        TestValue {
            x: bytes[0],
            y: bytes[1]
        }
    }

}


// Tests ----------------------------------------------------------------------
#[test]
fn test_input() {


    let input = TestInput {
        tick: 128,
        buttons: 7
    };

    assert_eq!(input.tick(), 128);
    assert_eq!(input.to_bytes(), [128, 7]);
    assert_eq!(TestInput::from_bytes(&input.to_bytes()), Some((2, input)));
    assert_eq!(TestInput::from_bytes(&[128, 7, 9, 2]), Some((2, input)));
    assert_eq!(TestInput::from_bytes(&[128]), None);

}

#[test]
fn test_value() {

    let value = TestValue {
        x: 100,
        y: 150
    };

    assert_eq!(value.to_bytes(), [100, 150]);
    assert_eq!(TestValue::from_bytes(&value.to_bytes()), value);

    let next_value = TestValue {
        x: 150,
        y: 200
    };

    assert_eq!(next_value.interpolate_from(&value, 0.25), TestValue {
        x: 112,
        y: 162
    });

    assert_eq!(next_value.interpolate_from(&value, 0.5), TestValue {
        x: 125,
        y: 175
    });

    assert_eq!(next_value.interpolate_from(&value, 0.75), TestValue {
        x: 137,
        y: 187
    });

}

#[test]
fn test_client() {

    let mut client = ClientState::<TestValue, TestInput>::new();

    assert_eq!(client.current, TestValue::default(), "should use default state as current state");
    assert_eq!(client.last, TestValue::default(), "should use default state as last state");

    client.apply_input(TestInput {
        tick: 0,
        buttons: 2
    });

    client.apply_input(TestInput {
        tick: 1,
        buttons: 1
    });

    client.apply_input(TestInput {
        tick: 2,
        buttons: 0
    });

    assert_eq!(client.send_inputs(), vec![0, 2, 1, 1, 2, 0], "should serialize all unconfirmed inputs");

    client.receive_state(&vec![100, 100], None);

    assert_eq!(client.last, TestValue::default(), "receive_state() should not modify last state");
    assert_eq!(client.current, TestValue::default(), "receive_state() should not modify current state");
    assert_eq!(client.send_inputs(), vec![0, 2, 1, 1, 2, 0], "receive_state() without tick should not confirm any buffered inputs");

    // Test update
    let mut count = 0;
    client.update_with(|state, input| {
        assert_eq!(input.tick(), count, "update_with() should be called for each buffered input");
        assert_eq!(input.buttons, 2 - count, "update_with() should be called for each buffered input");
        assert_eq!(*state, TestValue {
            x: 100,
            y: 100

        }, "update_with() should be called with the current state which is derived from the base state");
        count += 1;
    });

    assert_eq!(count, 3, "update_with() should be called for each buffered input");
    assert_eq!(client.send_inputs(), vec![0, 2, 1, 1, 2, 0], "update_with() should not confirmd any buffered inputs");


    assert_eq!(client.interpolate(0.25), TestValue {
        x: 25,
        y: 25

    }, "sould interpolate between the last and current state");

    assert_eq!(client.interpolate(0.5), TestValue {
        x: 50,
        y: 50

    }, "sould interpolate between the last and current state");

    assert_eq!(client.interpolate(0.75), TestValue {
        x: 75,
        y: 75

    }, "sould interpolate between the last and current state");

    assert_eq!(client.last, TestValue::default(), "update_with() should update the last with current state");
    assert_eq!(client.current, TestValue {
        x: 100,
        y: 100

    }, "update_with() should use the last received state as the new base for the current state");

    count = 0;
    client.update_with(|mut state, _| {

        assert_eq!(*state, TestValue {
            x: 100 + count * 10,
            y: 100 + count * 20

        }, "update_with() should allow to modify the current state");

        state.x += 10;
        state.y += 20;

        count += 1;

    });

    assert_eq!(client.current, TestValue {
        x: 130,
        y: 160

    }, "update_with() should allow to modify the current base state");

    client.update_with(|&mut _, _| {});

    let current_value = TestValue {
        x: 100,
        y: 100
    };

    assert_eq!(client.current, current_value, "update_with() should always reset the current state to the base start when first invoked");

    let last_value = TestValue {
        x: 130,
        y: 160
    };

    assert_eq!(client.last, last_value, "update_with() should set the last state to the previously calculate current state");

    client.receive_state(&vec![200, 200], Some(0));
    assert_eq!(client.send_inputs(), vec![0, 2, 1, 1, 2, 0], "receive state should not directly update unconfirmed inputs");

    client.update_with(|&mut _, _| {});

    assert_eq!(client.last, current_value, "update_with() should set last state to current state");
    assert_eq!(client.current, TestValue {
        x: 200,
        y: 200

    }, "update_with() should set current state to confirmed state");

    assert_eq!(client.send_inputs(), vec![1, 1, 2, 0], "update_with() should drop all confirmed buffered inputs");

}

#[test]
fn test_server() {

}

