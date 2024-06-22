use color_eyre::Result;
use std::path::Path;

pub async fn document_numbers() -> Result<()> {
    print!("Importing the document database numbers from the binary's static data...");
    db::import_document_database_numbers().await?;
    Ok(())
}

pub async fn tapes(path: &Path) -> Result<()> {
    print!("Importing the Tapes table from the NIST database...");
    db::import_nist_tapes_table_from_csv(path).await?;
    print!("done");
    Ok(())
}

pub async fn videos(path: &Path) -> Result<()> {
    print!("Importing the Videos table from the NIST database...");
    db::import_nist_videos_table_from_csv(path).await?;
    print!("done");
    Ok(())
}
