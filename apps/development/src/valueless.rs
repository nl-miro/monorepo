//! Offline diagnostics for saved offer documents.

use std::{env, fs, io, path::Path};

use bodul_shared::retailer::RetailerCode;
use retailer_data_extractor::{ValueLessResult, valueless_for};

/// Run the command, returning false if any source failed.
pub fn run() -> Result<bool, String> {
    let mut args = env::args().skip(1);
    let data = args
        .next()
        .ok_or("usage: valueless-offer-details <data_dir> [retailer ...]")?;
    let wanted: Vec<_> = args.collect();
    let retailers = crate::offer_details::retailers("sources", Path::new(&data), &wanted)?;
    if retailers.is_empty() {
        return Err(format!("no offer_details/sources folders in {data}"));
    }
    let mut all_ok = true;
    for (name, code, dir) in retailers {
        let entries = match fs::read_dir(dir.join("sources"))
            .and_then(|entries| entries.collect::<io::Result<Vec<_>>>())
        {
            Ok(entries) => entries,
            Err(error) => {
                eprintln!("{name}: {error}");
                all_ok = false;
                continue;
            }
        };
        let mut sources: Vec<_> = entries
            .into_iter()
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "html"))
            .collect();
        sources.sort();
        let mut written = 0;
        for source in &sources {
            let output = dir
                .join("valueless")
                .join(source.file_stem().expect("source filename"));
            match process_page(code, source, &output) {
                Ok(()) => written += 1,
                Err(error) => {
                    eprintln!("{}: {error}", source.display());
                    all_ok = false;
                }
            }
        }
        println!(
            "{name}: {} pages, {written} valueless, {} failed",
            sources.len(),
            sources.len() - written
        );
    }

    Ok(all_ok)
}

fn remove_output(output: &Path) -> io::Result<()> {
    match fs::symlink_metadata(output) {
        Ok(metadata) if metadata.is_dir() => fs::remove_dir_all(output),
        Ok(_) => fs::remove_file(output),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn process_page(code: RetailerCode, source: &Path, output: &Path) -> Result<(), String> {
    remove_output(output).map_err(|error| format!("remove stale output: {error}"))?;
    let result = (|| {
        let html = fs::read_to_string(source).map_err(|error| format!("read: {error}"))?;
        let result = valueless_for(code, &html).map_err(|error| format!("valueless: {error}"))?;

        write_result(&result, output).map_err(|error| format!("write: {error}"))
    })();
    if let Err(error) = result {
        remove_output(output).map_err(|cleanup| format!("{error}; cleanup failed: {cleanup}"))?;

        return Err(error);
    }

    Ok(())
}

fn write_result(result: &ValueLessResult, output: &Path) -> io::Result<()> {
    fs::create_dir_all(output)?;
    let extension = if result.struct_type == "next_js_app_router" {
        "json"
    } else {
        "html"
    };
    fs::write(output.join(format!("document.{extension}")), &result.html)?;
    for (index, child) in result.children.iter().enumerate() {
        // Indexes preserve order and distinguish duplicate or empty names. Only
        // safe ASCII characters enter paths; names cannot escape the output tree.
        let name: String = child
            .name
            .chars()
            .take(80)
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        write_result(child, &output.join(format!("{index:04}-{name}")))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_nested_children_and_json_without_name_collisions() {
        let dir = env::temp_dir().join(format!("valueless-tree-{}", std::process::id()));
        remove_output(&dir).unwrap();
        let result = ValueLessResult::document(
            "",
            "<html></html>",
            vec![
                ValueLessResult::segment(
                    "../same",
                    "first",
                    vec![ValueLessResult::next_js_app_router(
                        "payload",
                        "{\"price\":\"integer\"}",
                        vec![],
                    )],
                ),
                ValueLessResult::segment("../same", "second", vec![]),
            ],
        );
        write_result(&result, &dir).unwrap();
        assert_eq!(
            fs::read_to_string(dir.join("0000-___same/document.html")).unwrap(),
            "first"
        );
        assert_eq!(
            fs::read_to_string(dir.join("0001-___same/document.html")).unwrap(),
            "second"
        );
        assert_eq!(
            fs::read_to_string(dir.join("0000-___same/0000-payload/document.json")).unwrap(),
            "{\"price\":\"integer\"}"
        );
        remove_output(&dir).unwrap();
    }
}
