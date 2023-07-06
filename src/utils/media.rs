use std::ffi::OsStr;
use std::path::Path;

// 判断是否是图片
pub fn is_image(content_type: &str) -> bool {
    let image_mimes = vec![
        "image/jpeg",
        "image/png",
        "image/gif",
        "image/bmp",
        "image/webp",
        "image/tiff",
        "image/x-icon",
        "image/svg+xml",
    ];
    for image_mime in image_mimes {
        if content_type.eq(image_mime) {
            return true;
        }
    }
    false
}

// 判断是否是视频
pub fn is_video(content_type: &str) -> bool {
    let video_mimes = vec![
        "video/mp4",
        "video/ogg",
        "video/webm",
        "video/3gpp",
        "video/3gpp2",
        "video/avi",
        "video/mpeg",
        "video/quicktime",
        "video/x-flv",
        "video/x-matroska",
        "video/x-ms-wmv",
        "video/x-msvideo",
    ];
    for video_mime in video_mimes {
        if content_type.eq(video_mime) {
            return true;
        }
    }
    false
}

// 判断是否是图片或视频
pub fn is_media(content_type: &str) -> bool {
    if is_image(content_type) || is_video(content_type) {
        return true;
    }
    false
}

// 获取文件的 MIME 类型
pub fn get_content_type(file_path: &str) -> Option<String> {
    let file_extension = match Path::new(file_path).extension().and_then(OsStr::to_str) {
        Some(extension) => extension,
        None => {
            return None;
        }
    };
    let content_types = vec![
        // image
        ("jpg", "image/jpeg"),
        ("jpeg", "image/jpeg"),
        ("png", "image/png"),
        ("gif", "image/gif"),
        ("bmp", "image/bmp"),
        ("webp", "image/webp"),
        ("tiff", "image/tiff"),
        ("ico", "image/x-icon"),
        ("svg", "image/svg+xml"),
        // video
        ("mp4", "video/mp4"),
        ("ogg", "video/ogg"),
        ("webm", "video/webm"),
        ("3gp", "video/3gpp"),
        ("3g2", "video/3gpp2"),
        ("avi", "video/avi"),
        ("mpeg", "video/mpeg"),
        ("mov", "video/quicktime"),
        ("flv", "video/x-flv"),
        ("mkv", "video/x-matroska"),
        ("wmv", "video/x-ms-wmv"),
        ("avi", "video/x-msvideo"),
    ];
    for content_type in content_types {
        if file_extension.eq(content_type.0) {
            return Some(content_type.1.to_string());
        }
    }
    None
}

// 参数:视频的路径, 使用ffmpeg获取视频的缩略图
pub async fn get_video_thumbnail(video_path: &str) -> Result<String, String> {
    let frame_thumbnail_path = format!("{}.thumbnail.jpg", video_path);

    let output = tokio::process::Command::new("ffmpeg")
        .arg("-i")
        .arg(video_path)
        .arg("-y")
        .arg("-f")
        .arg("image2")
        .arg("-vf")
        .arg("thumbnail,scale=1280:-1")
        .arg("-vframes")
        .arg("1")
        .arg("-ss")
        .arg("1")
        .arg("-t")
        .arg("0.001")
        .arg(frame_thumbnail_path.as_str())
        .output()
        .await;
    match output {
        Ok(output) => {
            if output.status.success() {
                Ok(frame_thumbnail_path)
            } else {
                Err("failed to execute process".to_string())
            }
        }
        Err(err) => Err(err.to_string()),
    }
}

// 参数:图片的路径, 使用image库获取图片的缩略图
pub async fn get_image_thumbnail(image_path: &str) -> Result<String, String> {
    let img = image::io::Reader::open(image_path)
        .unwrap()
        .decode()
        .unwrap();
    let ori_width = img.width();
    let ori_height = img.height();
    let aspect_ratio = ori_width as f32 / ori_height as f32;
    let thumb_width = 1280;
    let thumb_height = (thumb_width as f32 / aspect_ratio) as u32;
    let thumb = img.thumbnail(thumb_width, thumb_height);
    let thumb_path = format!("{}.thumbnail.jpg", image_path);
    match thumb.save(thumb_path.as_str()) {
        Ok(_) => Ok(thumb_path),
        Err(err) => Err(err.to_string()),
    }
}
