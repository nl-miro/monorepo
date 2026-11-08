fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args_os()
        .nth(1)
        .ok_or("usage: download_ankerau <output-dir>")?;
    let run = retailer_sourcing::graphql::download_ankerau(std::path::Path::new(&output))?;
    println!("{}", run.display());
    Ok(())
}
