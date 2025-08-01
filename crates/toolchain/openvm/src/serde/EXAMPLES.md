# OpenVM Serde - Practical Examples

This document provides comprehensive examples of using the OpenVM serde component for various serialization and deserialization tasks in zkVM environments.

## Basic Usage

### Simple Types

```rust
use openvm::serde::{to_vec, from_slice};

// Serialize and deserialize primitive types
fn example_primitives() -> Result<(), openvm::serde::Error> {
    // Numbers
    let value: u32 = 42;
    let serialized = to_vec(&value)?;
    let deserialized: u32 = from_slice(&serialized)?;
    assert_eq!(value, deserialized);

    // Strings
    let text = "Hello, zkVM!".to_string();
    let serialized = to_vec(&text)?;
    let deserialized: String = from_slice(&serialized)?;
    assert_eq!(text, deserialized);

    // Booleans
    let flag = true;
    let serialized = to_vec(&flag)?;
    let deserialized: bool = from_slice(&serialized)?;
    assert_eq!(flag, deserialized);

    Ok(())
}
```

### Collections

```rust
use alloc::{vec::Vec, collections::BTreeMap};
use openvm::serde::{to_vec, from_slice};

fn example_collections() -> Result<(), openvm::serde::Error> {
    // Vectors
    let numbers = vec![1u64, 2, 3, 4, 5];
    let serialized = to_vec(&numbers)?;
    let deserialized: Vec<u64> = from_slice(&serialized)?;
    assert_eq!(numbers, deserialized);

    // Maps
    let mut scores = BTreeMap::new();
    scores.insert("Alice".to_string(), 100u32);
    scores.insert("Bob".to_string(), 85);
    scores.insert("Charlie".to_string(), 92);
    
    let serialized = to_vec(&scores)?;
    let deserialized: BTreeMap<String, u32> = from_slice(&serialized)?;
    assert_eq!(scores, deserialized);

    Ok(())
}
```

### Custom Structs

```rust
use serde::{Serialize, Deserialize};
use openvm::serde::{to_vec, from_slice};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Player {
    name: String,
    level: u32,
    health: f32,
    active: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct GameState {
    players: Vec<Player>,
    round: u32,
    started: bool,
}

fn example_custom_structs() -> Result<(), openvm::serde::Error> {
    let game_state = GameState {
        players: vec![
            Player {
                name: "Player1".to_string(),
                level: 10,
                health: 100.0,
                active: true,
            },
            Player {
                name: "Player2".to_string(),
                level: 8,
                health: 75.5,
                active: false,
            },
        ],
        round: 3,
        started: true,
    };

    let serialized = to_vec(&game_state)?;
    let deserialized: GameState = from_slice(&serialized)?;
    assert_eq!(game_state, deserialized);

    Ok(())
}
```

## Advanced Usage

### Enums and Options

```rust
use serde::{Serialize, Deserialize};
use openvm::serde::{to_vec, from_slice};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum Message {
    Text(String),
    Number(u64),
    Compound { id: u32, data: Vec<u8> },
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Envelope {
    sender: Option<String>,
    message: Message,
    timestamp: u64,
}

fn example_enums_options() -> Result<(), openvm::serde::Error> {
    let envelope = Envelope {
        sender: Some("zkVM".to_string()),
        message: Message::Compound {
            id: 42,
            data: vec![1, 2, 3, 4],
        },
        timestamp: 1234567890,
    };

    let serialized = to_vec(&envelope)?;
    let deserialized: Envelope = from_slice(&serialized)?;
    assert_eq!(envelope, deserialized);

    // Example with None
    let envelope_no_sender = Envelope {
        sender: None,
        message: Message::Text("Anonymous message".to_string()),
        timestamp: 1234567891,
    };

    let serialized = to_vec(&envelope_no_sender)?;
    let deserialized: Envelope = from_slice(&serialized)?;
    assert_eq!(envelope_no_sender, deserialized);

    Ok(())
}
```

### Tuples and Arrays

```rust
use openvm::serde::{to_vec, from_slice};

fn example_tuples_arrays() -> Result<(), openvm::serde::Error> {
    // Tuples
    let coordinate: (f64, f64, f64) = (1.0, 2.5, -3.7);
    let serialized = to_vec(&coordinate)?;
    let deserialized: (f64, f64, f64) = from_slice(&serialized)?;
    assert_eq!(coordinate, deserialized);

    // Arrays
    let matrix: [[u32; 3]; 3] = [
        [1, 2, 3],
        [4, 5, 6],
        [7, 8, 9],
    ];
    let serialized = to_vec(&matrix)?;
    let deserialized: [[u32; 3]; 3] = from_slice(&serialized)?;
    assert_eq!(matrix, deserialized);

    Ok(())
}
```

## Performance Optimizations

### Pre-allocated Serialization

```rust
use openvm::serde::{to_vec_with_capacity, from_slice};

fn example_preallocation() -> Result<(), openvm::serde::Error> {
    let large_data: Vec<u64> = (0..1000).collect();
    
    // Estimate capacity to avoid reallocations
    // Rough estimate: each u64 takes 2 words (8 bytes = 2 * 4-byte words)
    // Plus overhead for length prefix
    let estimated_capacity = large_data.len() * 2 + 1;
    
    let serialized = to_vec_with_capacity(&large_data, estimated_capacity)?;
    let deserialized: Vec<u64> = from_slice(&serialized)?;
    assert_eq!(large_data, deserialized);

    Ok(())
}
```

### Batch Operations

```rust
use openvm::serde::{to_vec, from_slice};
use alloc::vec::Vec;

fn example_batch_operations() -> Result<(), openvm::serde::Error> {
    // Serialize multiple items at once
    let items = vec![
        ("item1", 100u32),
        ("item2", 200),
        ("item3", 300),
    ];
    
    let serialized = to_vec(&items)?;
    let deserialized: Vec<(&str, u32)> = from_slice(&serialized)?;
    assert_eq!(items, deserialized);

    Ok(())
}
```

## Error Handling

### Comprehensive Error Handling

```rust
use openvm::serde::{to_vec, from_slice, Error};

fn example_error_handling() {
    // Handle serialization errors
    let data = "test data";
    match to_vec(&data) {
        Ok(serialized) => {
            // Handle deserialization errors
            match from_slice::<String>(&serialized) {
                Ok(deserialized) => println!("Success: {}", deserialized),
                Err(Error::DeserializeUnexpectedEnd) => {
                    println!("Data was truncated during transmission");
                },
                Err(Error::DeserializeBadUtf8) => {
                    println!("Invalid UTF-8 in string data");
                },
                Err(Error::Custom(msg)) => {
                    println!("Custom error: {}", msg);
                },
                Err(e) => {
                    println!("Other deserialization error: {:?}", e);
                },
            }
        },
        Err(Error::SerializeBufferFull) => {
            println!("Insufficient buffer capacity");
        },
        Err(e) => {
            println!("Serialization error: {:?}", e);
        },
    }
}
```

## zkVM-Specific Patterns

### Guest Program Data Exchange

```rust
use serde::{Serialize, Deserialize};
use openvm::serde::{to_vec, from_slice};

#[derive(Serialize, Deserialize)]
struct ComputationInput {
    values: Vec<u64>,
    parameters: (u32, u32),
}

#[derive(Serialize, Deserialize)]
struct ComputationResult {
    sum: u64,
    product: u64,
    max_value: u64,
}

fn zkvm_computation_example() -> Result<(), openvm::serde::Error> {
    // Simulate receiving input in guest program
    let input = ComputationInput {
        values: vec![10, 20, 30, 40, 50],
        parameters: (2, 3),
    };
    
    // Serialize input for processing
    let input_data = to_vec(&input)?;
    let processed_input: ComputationInput = from_slice(&input_data)?;
    
    // Perform computation
    let result = ComputationResult {
        sum: processed_input.values.iter().sum(),
        product: processed_input.values.iter().product(),
        max_value: processed_input.values.iter().max().copied().unwrap_or(0),
    };
    
    // Serialize result for output
    let output_data = to_vec(&result)?;
    let _verified_result: ComputationResult = from_slice(&output_data)?;
    
    Ok(())
}
```

### State Persistence

```rust
use serde::{Serialize, Deserialize};
use openvm::serde::{to_vec, from_slice};

#[derive(Serialize, Deserialize, Clone)]
struct PersistentState {
    counter: u64,
    last_update: u64,
    metadata: Vec<(String, String)>,
}

fn example_state_persistence() -> Result<(), openvm::serde::Error> {
    let mut state = PersistentState {
        counter: 100,
        last_update: 1234567890,
        metadata: vec![
            ("version".to_string(), "1.0".to_string()),
            ("author".to_string(), "zkVM".to_string()),
        ],
    };
    
    // Save state
    let saved_state = to_vec(&state)?;
    
    // Simulate state modification
    state.counter += 1;
    state.last_update += 1;
    
    // Restore state
    let restored_state: PersistentState = from_slice(&saved_state)?;
    assert_eq!(restored_state.counter, 100); // Original value
    
    Ok(())
}
```

## Testing Patterns

### Round-trip Testing Template

```rust
use openvm::serde::{to_vec, from_slice};

fn test_round_trip<T>(value: T) 
where
    T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + core::fmt::Debug,
{
    let serialized = to_vec(&value).expect("Serialization failed");
    let deserialized: T = from_slice(&serialized).expect("Deserialization failed");
    assert_eq!(value, deserialized, "Round-trip failed for value: {:?}", value);
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{vec::Vec, string::String};

    #[test]
    fn comprehensive_round_trip_tests() {
        // Test various types
        test_round_trip(42u32);
        test_round_trip("Hello World".to_string());
        test_round_trip(vec![1, 2, 3, 4, 5]);
        test_round_trip((1u32, "test".to_string(), true));
        test_round_trip(Some(42u64));
        test_round_trip(None::<u64>);
    }
}
```

## Performance Benchmarking

### Basic Benchmarking Template

```rust
use openvm::serde::{to_vec, from_slice};

fn benchmark_serialization_performance() {
    let large_data: Vec<u64> = (0..10000).collect();
    
    // Measure serialization time
    let start = std::time::Instant::now();
    let _serialized = to_vec(&large_data).unwrap();
    let serialize_duration = start.elapsed();
    
    let serialized = to_vec(&large_data).unwrap();
    
    // Measure deserialization time
    let start = std::time::Instant::now();
    let _deserialized: Vec<u64> = from_slice(&serialized).unwrap();
    let deserialize_duration = start.elapsed();
    
    println!("Serialization took: {:?}", serialize_duration);
    println!("Deserialization took: {:?}", deserialize_duration);
    println!("Serialized size: {} words", serialized.len());
}
```

These examples demonstrate the practical usage patterns for the OpenVM serde component, from basic serialization to advanced zkVM-specific scenarios. The component's word-aligned design ensures optimal performance in zero-knowledge virtual machine environments while maintaining compatibility with standard Rust serde patterns.