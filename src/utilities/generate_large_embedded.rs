use ciborium::value::Value;
use clap::Parser;
use rand::Rng;
use std::fs::File;
use std::io::Write;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target file size (e.g., "500kb", "10mb", "1gb")
    #[arg(short, long, default_value = "500kb")]
    size: String,

    /// Number of embedded CBOR payloads (optional limit)
    #[arg(short, long)]
    count: Option<usize>,

    /// Output file path
    #[arg(short, long, default_value = "test/data/large_embedded.bin")]
    output: String,
}

fn parse_size(s: &str) -> u64 {
    let s = s.to_lowercase();
    let (num, multiplier) = if s.ends_with("kb") {
        (&s[..s.len() - 2], 1024)
    } else if s.ends_with("mb") {
        (&s[..s.len() - 2], 1024 * 1024)
    } else if s.ends_with("gb") {
        (&s[..s.len() - 2], 1024 * 1024 * 1024)
    } else if s.ends_with("b") {
        (&s[..s.len() - 1], 1)
    } else {
        match s.parse::<u64>() {
            Ok(n) => return n,
            Err(_) => (&s[..], 1), // Try parsing as raw number if no suffix
        }
    };

    num.parse::<u64>().expect("Invalid size format") * multiplier
}

fn generate_massive_payload() -> Value {
    let mut rng = rand::thread_rng();

    // 1. Massive Array of Maps (simulating a large log dump)
    let log_entries: Vec<Value> = (0..500)
        .map(|i| {
            Value::Map(vec![
                (
                    Value::Text("id".to_string()),
                    Value::Integer((i as i64).into()),
                ),
                (
                    Value::Text("severity".to_string()),
                    Value::Text(if i % 10 == 0 {
                        "ERROR".to_string()
                    } else {
                        "INFO".to_string()
                    }),
                ),
                (
                    Value::Text("msg".to_string()),
                    Value::Text(format!("Log entry #{} process_id={}", i, rng.gen::<u16>())),
                ),
                (
                    Value::Text("meta".to_string()),
                    Value::Array(vec![
                        Value::Integer(rng.gen::<u8>().into()),
                        Value::Integer(rng.gen::<u8>().into()),
                        Value::Integer(rng.gen::<u8>().into()),
                    ]),
                ),
            ])
        })
        .collect();

    // 2. Deeply nested configuration
    let mut deep_config = Value::Map(vec![(
        Value::Text("level_0".to_string()),
        Value::Bool(true),
    )]);
    for i in 0..50 {
        deep_config = Value::Map(vec![(Value::Text(format!("level_{}", i + 1)), deep_config)]);
    }

    // 3. Large Byte Array (simulating an image or firmware blob)
    let blob_size = 10 * 1024; // 10KB
    let blob: Vec<u8> = (0..blob_size).map(|_| rng.gen()).collect();

    Value::Map(vec![
        (
            Value::Text("type".to_string()),
            Value::Text("MASSIVE_DATA_DUMP".to_string()),
        ),
        (Value::Text("logs".to_string()), Value::Array(log_entries)),
        (Value::Text("deep_config".to_string()), deep_config),
        (Value::Text("firmware_blob".to_string()), Value::Bytes(blob)),
    ])
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    let target_size = parse_size(&args.size);

    println!(
        "Targeting {} bytes output to '{}'",
        target_size, args.output
    );

    let mut file = File::create(&args.output)?;
    let mut rng = rand::thread_rng();
    let mut current_size = 0;
    let mut payload_count = 0;

    // Define standard payloads
    let standard_payloads = [
        // 1. Mission Log Entry
        Value::Map(vec![
            (
                Value::Text("id".to_string()),
                Value::Text("LOG-2026-X88".to_string()),
            ),
            (
                Value::Text("timestamp".to_string()),
                Value::Float(1771234567.89),
            ),
            (
                Value::Text("coordinates".to_string()),
                Value::Array(vec![
                    Value::Float(42.1234),
                    Value::Float(-71.5678),
                    Value::Float(1200.5),
                ]),
            ),
            (
                Value::Text("readings".to_string()),
                Value::Map(vec![
                    (Value::Text("temp".to_string()), Value::Float(22.5)),
                    (Value::Text("humidity".to_string()), Value::Float(45.2)),
                    (Value::Text("pressure".to_string()), Value::Float(1013.2)),
                ]),
            ),
        ]),
        // 2. User Profile Chunk
        Value::Map(vec![
            (
                Value::Text("user_id".to_string()),
                Value::Integer(99887766.into()),
            ),
            (
                Value::Text("username".to_string()),
                Value::Text("cyber_nomad_2077".to_string()),
            ),
            (
                Value::Text("preferences".to_string()),
                Value::Map(vec![(
                    Value::Text("theme".to_string()),
                    Value::Text("dark_neon".to_string()),
                )]),
            ),
        ]),
        // 3. Firmware Header
        Value::Map(vec![
            (
                Value::Text("magic".to_string()),
                Value::Integer((0xCAFEBABEu32 as i64).into()),
            ),
            (
                Value::Text("version".to_string()),
                Value::Text("v3.14.15".to_string()),
            ),
            (
                Value::Text("checksum".to_string()),
                Value::Bytes(vec![0xAA, 0xBB, 0xCC, 0xDD]),
            ),
        ]),
        // 4. Encrypted Message
        Value::Tag(
            24,
            Box::new(Value::Bytes(vec![
                0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99,
                0xAA, 0xBB,
            ])),
        ),
    ];

    while current_size < target_size {
        // Stop if we hit the count limit
        if let Some(limit) = args.count {
            if payload_count >= limit {
                break;
            }
        }

        // Random chunk of noise
        let noise_len = rng.gen_range(100..1000);
        let noise: Vec<u8> = (0..noise_len).map(|_| rng.gen()).collect();
        file.write_all(&noise)?;
        current_size += noise_len as u64;

        // Determine if we should write a massive payload or a standard one
        // 1 in 20 chance for massive payload
        let payload = if rng.gen_ratio(1, 20) {
            generate_massive_payload()
        } else {
            standard_payloads[rng.gen_range(0..standard_payloads.len())].clone()
        };

        let mut buffer = Vec::new();
        ciborium::ser::into_writer(&payload, &mut buffer).unwrap();
        file.write_all(&buffer)?;
        current_size += buffer.len() as u64;
        payload_count += 1;

        if current_size >= target_size {
            break;
        }
    }

    println!(
        "Done! Generated {} payloads. Total size: {} bytes",
        payload_count, current_size
    );
    Ok(())
}
