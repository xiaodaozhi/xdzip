//! Owned public-API harness. Fixed keys and passwords are test data only.
use aescry::{
    aescrypt::{Decryptor, Iterations},
    security::{EncryptKey, PublicIv, RawEncryptor, SessionIv, SessionKey},
    Version,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 6 {
        return Err("expected operation version profile input output".into());
    }
    let version = match args[2].as_str() {
        "0" => Version::V0,
        "1" => Version::V1,
        "2" => Version::V2,
        "3" => Version::V3,
        _ => return Err("version".into()),
    };
    let password = match args[3].as_str() {
        "unicode" => "owned-資料-🔑".to_owned(),
        "long" => "p".repeat(128) + "資料-🔑",
        _ => "owned-aes-fixture".to_owned(),
    };
    let iterations = match args[3].as_str() {
        "default" => 600_000,
        "one" => 1,
        _ => 2,
    };
    let input = std::fs::read(&args[4])?;
    let result = match args[1].as_str() {
        "encode" => {
            let encoded = RawEncryptor::new(EncryptKey::password(&password)?)
                .version(version)
                .iterations(Iterations::new(iterations)?)
                .deterministic(
                    PublicIv::try_from(&[0x17; 16][..])?,
                    SessionIv::from([0x35; 16]),
                    SessionKey::from([0x59; 32]),
                )
                .encrypt(&input)?;
            if Decryptor::new(&password)?.decrypt(&encoded)? != input {
                return Err("reference self-check".into());
            }
            encoded
        }
        "decode" => Decryptor::new(&password)?.decrypt(&input)?,
        _ => return Err("operation".into()),
    };
    std::fs::write(&args[5], result)?;
    Ok(())
}
