use kohaku_tornadocash::note::Note;
use serde_json::json;

fn main() {
    // Deliberately public fixtures. Never use these values with funds.
    let original = Note::new([1; 31], [2; 31], "ETH".into(), "1".into(), 31337);
    let mut changed = original.clone();
    changed.secret[0] ^= 1;

    // Call the SDK directly, independently of the exported WASM function.
    println!(
        "{}",
        json!({
            "original": {
                "note": original.to_string(),
                "commitment": format!("{:#066x}", original.commitment()),
            },
            "changed": {
                "note": changed.to_string(),
                "commitment": format!("{:#066x}", changed.commitment()),
            },
        })
    );
}
