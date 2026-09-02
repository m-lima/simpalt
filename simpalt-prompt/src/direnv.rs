pub fn is_active() -> Option<bool> {
    let diff = std::env::var("DIRENV_DIFF").ok()?;

    let mut decoded = decode(&diff)?;
    let parsed = simd_json::to_tape(&mut decoded).ok()?;

    detect_active(&parsed)
}

fn decode(diff: &str) -> Option<Vec<u8>> {
    let decoded =
        base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE, diff.trim()).ok()?;
    let mut zlib = flate2::read::ZlibDecoder::new(decoded.as_slice());
    let mut buffer = Vec::new();

    std::io::Read::read_to_end(&mut zlib, &mut buffer)
        .ok()
        .map(|_| buffer)
}

fn detect_active(tape: &simd_json::Tape<'_>) -> Option<bool> {
    tape.as_value().get("p")?.as_object().map(|p| !p.is_empty())
}
