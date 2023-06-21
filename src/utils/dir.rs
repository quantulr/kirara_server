pub async fn create_dir(path: &str) -> Result<(), std::io::Error> {
    // 使用 Tokio 的异步文件系统操作检查目录是否存在
    let metadata = tokio::fs::metadata(path).await;

    // 检查目录是否存在
    if let Ok(metadata) = metadata {
        if !metadata.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "文件名与目录名冲突！",
            ));
        }
    } else {
        // 目录不存在，创建目录
        if let Err(_err) = tokio::fs::create_dir_all(path).await {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "目录创建失败",
            ));
        }
    }
    Ok(())
}
