use color_eyre::Result;

pub async fn ls() -> Result<()> {
    let videos = db::get_nist_videos().await?;
    for video in videos.iter() {
        video.print_row();
    }
    Ok(())
}
