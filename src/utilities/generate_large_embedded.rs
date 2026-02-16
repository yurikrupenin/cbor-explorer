use std::fs::File;
use std::io::Write;
use ciborium::value::Value;
use rand::Rng;

fn main() -> std::io::Result<()> {
    let mut file = File::create("test/data/large_embedded.bin")?;
    let mut rng = rand::thread_rng();
    
    // Target size ~500KB
    let target_size = 500 * 1024;
    let mut current_size = 0;

    // Define some "real-world" CBOR payloads
    let payloads = vec![
        // 1. Mission Log Entry
        Value::Map(vec![
            (Value::Text("id".to_string()), Value::Text("LOG-2026-X88".to_string())),
            (Value::Text("timestamp".to_string()), Value::Float(1771234567.89)),
            (Value::Text("coordinates".to_string()), Value::Array(vec![
                Value::Float(42.1234),
                Value::Float(-71.5678),
                Value::Float(1200.5),
            ])),
            (Value::Text("readings".to_string()), Value::Map(vec![
                (Value::Text("temp".to_string()), Value::Float(22.5)),
                (Value::Text("humidity".to_string()), Value::Float(45.2)),
                (Value::Text("pressure".to_string()), Value::Float(1013.2)),
            ])),
        ]),

        // 2. User Profile Chunk
        Value::Map(vec![
            (Value::Text("user_id".to_string()), Value::Integer(99887766.into())),
            (Value::Text("username".to_string()), Value::Text("cyber_nomad_2077".to_string())),
            (Value::Text("preferences".to_string()), Value::Map(vec![
                (Value::Text("theme".to_string()), Value::Text("dark_neon".to_string())),
                (Value::Text("notifications".to_string()), Value::Bool(true)),
                (Value::Text("auto_save".to_string()), Value::Bool(false)),
            ])),
            (Value::Text("inventory".to_string()), Value::Array(vec![
                Value::Text("item_sword_laser".to_string()),
                Value::Text("item_shield_plasma".to_string()),
                Value::Text("item_potion_health_xl".to_string()),
            ])),
        ]),

        // 3. Firmware Header
        Value::Map(vec![
            (Value::Text("magic".to_string()), Value::Integer((0xCAFEBABEu32 as i64).into())),
            (Value::Text("version".to_string()), Value::Text("v3.14.15".to_string())),
            (Value::Text("build_date".to_string()), Value::Text("2026-02-14T12:00:00Z".to_string())),
            (Value::Text("checksum".to_string()), Value::Bytes(vec![0xAA, 0xBB, 0xCC, 0xDD])),
            (Value::Text("segments".to_string()), Value::Array(vec![
                Value::Map(vec![
                    (Value::Text("addr".to_string()), Value::Integer(0x1000.into())),
                    (Value::Text("size".to_string()), Value::Integer(0x2000.into())),
                    (Value::Text("flags".to_string()), Value::Text("RX".to_string())),
                ]),
                Value::Map(vec![
                    (Value::Text("addr".to_string()), Value::Integer(0x4000.into())),
                    (Value::Text("size".to_string()), Value::Integer(0x1000.into())),
                    (Value::Text("flags".to_string()), Value::Text("RW".to_string())),
                ]),
            ])),
        ]),

        // 4. Encrypted Message (simulated)
        Value::Tag(24, Box::new(Value::Bytes(vec![
            0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x11, 0x22, 0x33,
            0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xAA, 0xBB
        ]))),

        // 5. Complex Configuration (High Nesting)
        Value::Map(vec![
            (Value::Text("system".to_string()), Value::Map(vec![
                (Value::Text("network".to_string()), Value::Map(vec![
                    (Value::Text("interfaces".to_string()), Value::Array(vec![
                        Value::Map(vec![
                            (Value::Text("name".to_string()), Value::Text("eth0".to_string())),
                            (Value::Text("ip".to_string()), Value::Text("192.168.1.10".to_string())),
                            (Value::Text("mask".to_string()), Value::Text("255.255.255.0".to_string())),
                        ]),
                        Value::Map(vec![
                            (Value::Text("name".to_string()), Value::Text("wlan0".to_string())),
                            (Value::Text("ssid".to_string()), Value::Text("DeepSpace".to_string())),
                            (Value::Text("security".to_string()), Value::Text("WPA3".to_string())),
                        ]),
                    ])),
                    (Value::Text("dns".to_string()), Value::Array(vec![
                        Value::Text("8.8.8.8".to_string()),
                        Value::Text("1.1.1.1".to_string()),
                    ])),
                ])),
                (Value::Text("storage".to_string()), Value::Map(vec![
                    (Value::Text("volumes".to_string()), Value::Array(vec![
                        Value::Map(vec![
                            (Value::Text("label".to_string()), Value::Text("ROOT".to_string())),
                            (Value::Text("fs".to_string()), Value::Text("btrfs".to_string())),
                            (Value::Text("used_percent".to_string()), Value::Integer(78.into())),
                        ]),
                    ])),
                ])),
            ])),
        ]),

        // 6. Large Data Array (Homogeneous)
        Value::Array((0..50).map(|i| Value::Integer((i * 100).into())).collect()),

        // 7. Recursive-like Structure
        Value::Map(vec![
            (Value::Text("node".to_string()), Value::Text("root".to_string())),
            (Value::Text("left".to_string()), Value::Map(vec![
                (Value::Text("node".to_string()), Value::Text("L".to_string())),
                (Value::Text("left".to_string()), Value::Map(vec![
                   (Value::Text("node".to_string()), Value::Text("LL".to_string())),
                ])),
                (Value::Text("right".to_string()), Value::Map(vec![
                   (Value::Text("node".to_string()), Value::Text("LR".to_string())),
                ])),
            ])),
            (Value::Text("right".to_string()), Value::Map(vec![
                (Value::Text("node".to_string()), Value::Text("R".to_string())),
            ])),
        ]),
    ];

    println!("Generating ~500KB binary file with embedded CBOR...");

    while current_size < target_size {
        // Random chunk of noise
        let noise_len = rng.gen_range(500..5000); // 0.5KB to 5KB of noise
        let noise: Vec<u8> = (0..noise_len).map(|_| rng.gen()).collect();
        file.write_all(&noise)?;
        current_size += noise_len;

        // Valid CBOR payload
        let payload = &payloads[rng.gen_range(0..payloads.len())];
        let mut buffer = Vec::new();
        ciborium::ser::into_writer(payload, &mut buffer).unwrap();
        file.write_all(&buffer)?;
        current_size += buffer.len();

        // More noise
        if current_size >= target_size {
            break;
        }
    }

    println!("Done! Total size: {} bytes", current_size);
    Ok(())
}
