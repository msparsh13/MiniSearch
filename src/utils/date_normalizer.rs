pub fn normalize_date(date: &str) -> Option<String> {
     let date = date.trim().replace('/', "-");
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return None;
    }

    // Detect format
    if parts[0].len() == 4 {
        // Already YYYY-MM-DD
        let y = parts[0];
        let m = parts[1];
        let d = parts[2];
        return Some(format!("{y}-{m}-{d}"));
    } else {
        // DD-MM-YYYY → convert
        let d = parts[0];
        let m = parts[1];
        let y = parts[2];
        return Some(format!("{y}-{m}-{d}"));
    }
}
