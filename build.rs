fn main() {
    println!("cargo:rerun-if-changed=loader/simpalt.nu");
    println!("cargo:rerun-if-changed=loader/simpalt.zsh");

    std::fs::write(
        concat!(env!("CARGO_MANIFEST_DIR"), "/simpalt.nu"),
        String::from_utf8(
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/loader/simpalt.nu")).unwrap(),
        )
        .unwrap()
        .replace("%%VERSION%%", env!("CARGO_PKG_VERSION")),
    )
    .unwrap();

    std::fs::write(
        concat!(env!("CARGO_MANIFEST_DIR"), "/simpalt.zsh"),
        String::from_utf8(
            std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/loader/simpalt.zsh")).unwrap(),
        )
        .unwrap()
        .replace("%%VERSION%%", env!("CARGO_PKG_VERSION")),
    )
    .unwrap();
}
