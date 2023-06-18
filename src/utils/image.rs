use std::ffi::OsStr;
use std::path::Path;

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
        if content_type.to_owned().eq(image_mime) {
            return true;
        }
    }
    false
}

pub fn get_content_type(file_path: &str) -> Option<String> {
    let file_extension = Path::new(file_path)
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("jpg");
    let content_types = vec![
        ("jpg", "image/jpeg"),
        ("jpeg", "image/jpeg"),
        ("png", "image/png"),
        ("gif", "image/gif"),
        ("bmp", "image/bmp"),
        ("webp", "image/webp"),
        ("tiff", "image/tiff"),
        ("ico", "image/x-icon"),
        ("svg", "image/svg+xml"),
    ];
    for content_type in content_types {
        if file_extension.eq(content_type.0) {
            return Some(content_type.1.to_string());
        }
    }
    None
}
