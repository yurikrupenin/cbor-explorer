use std::fs::File;
use std::io::Write;
use ciborium::value::Value;

fn main() -> std::io::Result<()> {
    let mut file = File::create("test/data/embedded.bin")?;

    // 1. Random garbage (100 bytes)
    let garbage1: Vec<u8> = (0..100).map(|i| (i % 255) as u8).collect();
    file.write_all(&garbage1)?;

    // 2. Valid CBOR Map
    let map_data = vec![
        (Value::Text("key".to_string()), Value::Text("value".to_string())),
        (Value::Text("id".to_string()), Value::Integer(123.into())),
    ];
    let map_value = Value::Map(map_data);
    let mut map_bytes = Vec::new();
    ciborium::ser::into_writer(&map_value, &mut map_bytes).unwrap();
    file.write_all(&map_bytes)?;

    // 3. More garbage (50 bytes)
    let garbage2: Vec<u8> = (0..50).map(|i| ((i + 100) % 255) as u8).collect();
    file.write_all(&garbage2)?;

    // 4. Valid CBOR Array
    let array_value = Value::Array(vec![
        Value::Integer(1.into()),
        Value::Integer(2.into()),
        Value::Integer(3.into()),
        Value::Text("nested".to_string()),
    ]);
    let mut array_bytes = Vec::new();
    ciborium::ser::into_writer(&array_value, &mut array_bytes).unwrap();
    file.write_all(&array_bytes)?;

    // 5. More garbage (20 bytes)
    let garbage3: Vec<u8> = (0..20).map(|i| ((i + 200) % 255) as u8).collect();
    file.write_all(&garbage3)?;

    // 6. Sequence of small integers (valid CBOR but could be noise)
    // We want to test if our heuristic picks this up or ignores it. 
    // Ideally, a sequence of valid items should be detected.
    for i in 0..5 {
        let val = Value::Integer(i.into());
        let mut bytes = Vec::new();
        ciborium::ser::into_writer(&val, &mut bytes).unwrap();
        file.write_all(&bytes)?;
    }

    println!("Generated test/data/embedded.bin");
    Ok(())
}
