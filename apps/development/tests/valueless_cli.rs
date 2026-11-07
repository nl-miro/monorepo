use std::{fs, path::Path, process::Command};

fn run(root: &Path, retailers: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_valueless-offer-details"))
        .arg(root)
        .args(retailers)
        .output()
        .unwrap()
}

#[test]
fn selection_reruns_and_failures_preserve_sources_and_continue() {
    let root = std::env::temp_dir().join(format!("valueless-cli-{}", std::process::id()));
    let source_dir = root.join("ankerau/offer_details/sources");
    let output = root.join("ankerau/offer_details/valueless/a");
    fs::create_dir_all(&source_dir).unwrap();
    fs::create_dir_all(root.join("ankervn/offer_details/sources")).unwrap();
    let html = "<html><head><script type=\"application/ld+json\">{\"name\":\"Product\"}</script></head><body>untouched</body></html>";
    fs::write(source_dir.join("a.html"), html).unwrap();
    fs::write(source_dir.join("skip.txt"), "ignored").unwrap();
    fs::write(
        root.join("ankervn/offer_details/sources/other.html"),
        "<html></html>",
    )
    .unwrap();
    let first = run(&root, &["ankerau"]);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert!(output.join("document.html").is_file());
    assert!(fs::read_dir(&output).unwrap().count() > 1);
    assert!(!root.join("ankervn/offer_details/valueless").exists());
    assert!(!output.parent().unwrap().join("skip").exists());
    assert_eq!(fs::read_to_string(source_dir.join("a.html")).unwrap(), html);

    fs::write(output.join("obsolete.html"), "stale").unwrap();
    assert!(run(&root, &["ankerau"]).status.success());
    assert!(!output.join("obsolete.html").exists());
    // An early read failure must remove its previous output and not stop b.
    fs::write(source_dir.join("a.html"), [0xff]).unwrap();
    fs::write(source_dir.join("b.html"), html).unwrap();
    let failed = run(&root, &["ankerau"]);
    assert_eq!(failed.status.code(), Some(1));
    assert!(!output.exists());
    assert!(output.parent().unwrap().join("b/document.html").exists());

    // Parser failures have the same cleanup behavior.
    fs::write(
        source_dir.join("a.html"),
        "<script type=\"application/ld+json\">{invalid}</script>",
    )
    .unwrap();
    fs::create_dir_all(&output).unwrap();
    fs::write(output.join("document.html"), "stale").unwrap();
    assert_eq!(run(&root, &[]).status.code(), Some(1));
    assert!(!output.exists());
    assert!(
        root.join("ankervn/offer_details/valueless/other/document.html")
            .exists()
    );
    assert_eq!(run(&root, &["not-a-retailer"]).status.code(), Some(2));
    fs::remove_dir_all(root).unwrap();
}
