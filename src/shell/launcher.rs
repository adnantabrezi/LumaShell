use super::*;

pub fn collect_apps(dir: PathBuf, items: &mut Vec<(String, String)>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_apps(path, items);
        } else if matches!(
            path.extension().and_then(|x| x.to_str()),
            Some("lnk") | Some("url") | Some("exe")
        ) {
            let name = path
                .file_stem()
                .and_then(|x| x.to_str())
                .unwrap_or("App")
                .replace('_', " ");
            items.push((name, path.to_string_lossy().into_owned()));
        }
    }
}

pub fn build_app_index() -> Vec<(String, String)> {
    let mut items = Vec::new();
    if let Some(appdata) = env::var_os("APPDATA") {
        collect_apps(
            PathBuf::from(appdata).join("Microsoft/Windows/Start Menu/Programs"),
            &mut items,
        );
    }
    if let Some(program_data) = env::var_os("ProgramData") {
        collect_apps(
            PathBuf::from(program_data).join("Microsoft/Windows/Start Menu/Programs"),
            &mut items,
        );
    }
    for (name, target) in [
        ("Wi-Fi settings", "ms-settings:network-wifi"),
        ("Sound settings", "ms-settings:sound"),
        ("Display settings", "ms-settings:display"),
        ("Bluetooth settings", "ms-settings:bluetooth"),
        ("Notifications settings", "ms-settings:notifications"),
        ("Personalization settings", "ms-settings:personalization"),
        ("Windows Update", "ms-settings:windowsupdate"),
    ] {
        items.push((name.to_string(), target.to_string()));
    }
    items.sort_by_key(|(name, _)| name.to_lowercase());
    items.dedup_by(|a, b| a.0.eq_ignore_ascii_case(&b.0));
    items
}

pub fn collect_matching_files(
    dir: &Path,
    query: &str,
    depth: u8,
    budget: &mut usize,
    items: &mut Vec<(String, String)>,
) {
    if depth > 3 || *budget == 0 {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if *budget == 0 {
            break;
        }
        *budget -= 1;
        let path = entry.path();
        if path.is_dir() {
            collect_matching_files(&path, query, depth + 1, budget, items);
        } else if path
            .file_name()
            .and_then(|value| value.to_str())
            .map(|name| name.to_lowercase().contains(query))
            .unwrap_or(false)
        {
            let name = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("File");
            items.push((
                format!("File · {}", name),
                path.to_string_lossy().into_owned(),
            ));
            if items.len() >= 20 {
                *budget = 0;
            }
        }
    }
}

pub fn search_user_files(query: &str) -> Vec<(String, String)> {
    let Some(profile) = env::var_os("USERPROFILE") else {
        return Vec::new();
    };
    let profile = PathBuf::from(profile);
    let mut items = Vec::new();
    let mut budget = 1500usize;
    for folder in ["Desktop", "Documents", "Downloads"] {
        collect_matching_files(&profile.join(folder), query, 0, &mut budget, &mut items);
        if budget == 0 {
            break;
        }
    }
    items
}

pub fn recent_path() -> PathBuf {
    config_path().with_file_name("recent.txt")
}
pub fn favorites_path() -> PathBuf {
    config_path().with_file_name("favorites.txt")
}
pub fn favorite_targets() -> Vec<String> {
    fs::read_to_string(favorites_path())
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}
pub fn is_favorite(target: &str) -> bool {
    favorite_targets()
        .iter()
        .any(|item| item.eq_ignore_ascii_case(target))
}
pub fn toggle_favorite(target: &str) {
    let mut items = favorite_targets();
    if let Some(index) = items
        .iter()
        .position(|item| item.eq_ignore_ascii_case(target))
    {
        items.remove(index);
    } else {
        items.insert(0, target.to_string());
        items.truncate(32);
    }
    if let Some(parent) = favorites_path().parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(favorites_path(), items.join("\n"));
}
pub fn record_recent(target: &str) {
    let path = recent_path();
    let mut items = fs::read_to_string(&path)
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|item| !item.is_empty() && !item.eq_ignore_ascii_case(target))
        .map(str::to_string)
        .collect::<Vec<_>>();
    items.insert(0, target.to_string());
    items.truncate(12);
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(path, items.join("\n"));
}

pub unsafe fn launcher_query() -> String {
    let len = GetWindowTextLengthW(LAUNCHER_EDIT);
    if len <= 0 {
        return String::new();
    }
    let mut buffer = vec![0u16; len as usize + 1];
    let copied = GetWindowTextW(LAUNCHER_EDIT, &mut buffer);
    String::from_utf16_lossy(&buffer[..copied as usize]).to_lowercase()
}

pub unsafe fn launcher_entries(query: &str) -> Vec<(String, String)> {
    let Some(index) = &APP_INDEX else {
        return Vec::new();
    };
    let mut result = Vec::new();
    if query.is_empty() {
        for target in favorite_targets() {
            let found = index
                .iter()
                .find(|item| item.1.eq_ignore_ascii_case(&target));
            if let Some(item) = found.filter(|item| {
                !result
                    .iter()
                    .any(|existing: &(String, String)| existing.1 == item.1)
            }) {
                result.push(item.clone());
            }
        }
        for target in fs::read_to_string(recent_path())
            .unwrap_or_default()
            .lines()
            .map(str::trim)
        {
            let found = index
                .iter()
                .find(|item| item.1.eq_ignore_ascii_case(target));
            if let Some(item) = found.filter(|item| {
                !result
                    .iter()
                    .any(|existing: &(String, String)| existing.1 == item.1)
            }) {
                result.push(item.clone());
            }
        }
    }
    for item in index
        .iter()
        .filter(|item| query.is_empty() || item.0.to_lowercase().contains(query))
    {
        if !result
            .iter()
            .any(|existing: &(String, String)| existing.1 == item.1)
        {
            result.push(item.clone());
        }
    }
    if query.len() >= 2 {
        for item in search_user_files(query) {
            if !result
                .iter()
                .any(|existing: &(String, String)| existing.1 == item.1)
            {
                result.push(item);
            }
        }
    }
    result
}
