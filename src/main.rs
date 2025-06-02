use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;
use std::env;
use std::time::SystemTime;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};

/// Cache for user and group name lookups
struct NameCache {
	users: HashMap<u32, String>,
	groups: HashMap<u32, String>,
}

impl NameCache {
	/// Create a new NameCache and populate it by reading system files
	fn new() -> Self {
		let mut cache = NameCache {
			users: HashMap::new(),
			groups: HashMap::new(),
		};
		
		// Load user names from /etc/passwd
		if let Ok(file) = fs::File::open("/etc/passwd") {
			let reader = BufReader::new(file);
			for line in reader.lines() {
				if let Ok(line) = line {
					let parts: Vec<&str> = line.split(':').collect();
					if parts.len() >= 3 {
						if let Ok(uid) = parts[2].parse::<u32>() {
							cache.users.insert(uid, parts[0].to_string());
						}
					}
				}
			}
		}
		
		// Load group names from /etc/group
		if let Ok(file) = fs::File::open("/etc/group") {
			let reader = BufReader::new(file);
			for line in reader.lines() {
				if let Ok(line) = line {
					let parts: Vec<&str> = line.split(':').collect();
					if parts.len() >= 3 {
						if let Ok(gid) = parts[2].parse::<u32>() {
							cache.groups.insert(gid, parts[0].to_string());
						}
					}
				}
			}
		}
		
		cache
	}
	
	/// Get user name from UID, fallback to UID string if not found
	fn get_user_name(&self, uid: u32) -> String {
		self.users.get(&uid).cloned().unwrap_or_else(|| uid.to_string())
	}
	
	/// Get group name from GID, fallback to GID string if not found
	fn get_group_name(&self, gid: u32) -> String {
		self.groups.get(&gid).cloned().unwrap_or_else(|| gid.to_string())
	}
}

/// Represents a file system entry with display information
#[derive(Debug)]
struct FileEntry {
	name: String,
	permissions: String,
	owner: String,
	group: String,
	modified_text: String,
	icon: &'static str,
	is_dir: bool,
}

impl FileEntry {
	/// Create a new FileEntry from a directory entry
	fn new(entry: &fs::DirEntry, name_cache: &NameCache) -> std::io::Result<Self> {
		let metadata = entry.metadata()?;
		let file_name = entry.file_name().to_string_lossy().to_string();
		
		// Get permissions in octal format
		let mode = metadata.permissions().mode();
		let permissions = format!("{:o}", mode & 0o777);
		
		// Get owner and group IDs and resolve to names
		let owner_uid = metadata.uid();
		let group_gid = metadata.gid();
		let owner = name_cache.get_user_name(owner_uid);
		let group = name_cache.get_group_name(group_gid);
		
		// Get modification time and format as fuzzy duration
		let modified_text = match metadata.modified() {
			Ok(modified_time) => format_duration_since(modified_time),
			Err(_) => "unknown".to_string(),
		};
		
		let is_dir = metadata.is_dir();
		let icon = get_file_icon(&file_name, is_dir);
		
		Ok(FileEntry {
			name: file_name,
			permissions,
			owner,
			group,
			modified_text,
			icon,
			is_dir,
		})
	}
	
	/// Format this entry for display with proper column alignment
	fn format_display(&self, max_perms_len: usize, max_owner_len: usize, max_group_len: usize, max_modified_len: usize) -> String {
		// Use ANSI escape codes for colors
		let (name_color, reset) = if self.is_dir {
			("\x1b[34;1m", "\x1b[0m") // Blue bold for directories
		} else {
			("", "") // No color for files
		};
		
		// Ensure consistent icon column width with padding
		format!(
			"{}  \x1b[33m{:>width_perms$}\x1b[0m \x1b[32m{:>width_owner$}\x1b[0m \x1b[36m{:>width_group$}\x1b[0m \x1b[35m{:>width_modified$}\x1b[0m {}{}{}",
			self.icon,
			self.permissions,
			self.owner,
			self.group,
			self.modified_text,
			name_color,
			self.name,
			reset,
			width_perms = max_perms_len,
			width_owner = max_owner_len,
			width_group = max_group_len,
			width_modified = max_modified_len
		)
	}
}

/// Format duration since a given time into human-readable fuzzy text
fn format_duration_since(modified_time: SystemTime) -> String {
	let now = SystemTime::now();
	
	let duration = match now.duration_since(modified_time) {
		Ok(d) => d,
		Err(_) => return "future".to_string(), // File modified in the future?
	};
	
	let seconds = duration.as_secs();
	
	match seconds {
		0..=59 => {
			if seconds == 0 { "now".to_string() }
			else if seconds == 1 { "1 second".to_string() }
			else { format!("{} seconds", seconds) }
		},
		60..=3599 => {
			let minutes = seconds / 60;
			if minutes == 1 { "1 minute".to_string() }
			else { format!("{} minutes", minutes) }
		},
		3600..=86399 => {
			let hours = seconds / 3600;
			if hours == 1 { "1 hour".to_string() }
			else { format!("{} hours", hours) }
		},
		86400..=604799 => {
			let days = seconds / 86400;
			if days == 1 { "1 day".to_string() }
			else { format!("{} days", days) }
		},
		604800..=2629743 => {
			let weeks = seconds / 604800;
			if weeks == 1 { "1 week".to_string() }
			else { format!("{} weeks", weeks) }
		},
		2629744..=31556925 => {
			let months = seconds / 2629744; // Approximate month
			if months == 1 { "1 month".to_string() }
			else { format!("{} months", months) }
		},
		_ => {
			let years = seconds / 31556926; // Approximate year
			if years == 1 { "1 year".to_string() }
			else { format!("{} years", years) }
		}
	}
}

/// Get an appropriate icon for the file type
fn get_file_icon(filename: &str, is_dir: bool) -> &'static str {
	if is_dir {
		return "📁";  // nf-cod-folder or folder emoji
	}
	
	// Get file extension
	let extension = Path::new(filename)
		.extension()
		.and_then(|ext| ext.to_str())
		.unwrap_or("")
		.to_lowercase();
	
	match extension.as_str() {
		"rs" => "🦀",          // nf-dev-rust / Rust crab
		"py" => "🐍",          // nf-dev-python / Python snake
		"js" => "󰌞",          // nf-dev-javascript
		"ts" => "󰛦",          // nf-dev-typescript
		"html" | "htm" => "󰌝",// nf-dev-html5
		"css" => "󰌜",         // nf-dev-css3
		"json" => "󰘦",        // nf-mdi-code_json
		"md" | "markdown" => "󰍔", // nf-dev-markdown
		"txt" => "󰈙",         // nf-fa-file_text_o
		"pdf" => "󰈦",         // nf-fa-file_pdf_o
		"zip" | "tar" | "gz" | "rar" => "🗜️", // nf-fa-file_archive_o
		"jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" => "🖼️", // nf-fa-file_image_o
		"mp3" | "wav" | "flac" | "ogg" => "🎵", // nf-fa-file_audio_o
		"mp4" | "mkv" | "avi" | "mov" => "🎬", // nf-fa-file_video_o
		"exe" | "bin" => "⚙️",  // nf-mdi-application
		"toml" | "yaml" | "yml" | "ini" | "conf" => "⚙️", // nf-mdi-settings
		"c" | "h" => "󰙱",      // nf-custom-c
		"cpp" | "cc" | "cxx" | "hpp" => "󰙲", // nf-custom-cpp
		"java" => "󰬷",        // nf-dev-java
		"php" => "󰌟",         // nf-dev-php
		"rb" => "󰴭",          // nf-dev-ruby
		"go" => "󰟓",          // nf-dev-go or "🐹" for gopher
		"sh" | "bash" | "zsh" => "󰆍", // nf-dev-terminal
		"sql" => "󰆼",         // nf-dev-database
		"xml" => "󰗀",         // nf-mdi-xml
		"log" => "󰌱",         // nf-fa-file_text_o
		"lock" => "󰌾",        // nf-fa-lock
		"dockerfile" => "🐳",  // Docker whale
		"vue" => "󰡄",         // nf-mdi-vuejs
		"react" | "jsx" | "tsx" => "󰜈", // nf-dev-react
		"git" => "󰊢",         // nf-dev-git
		"node" => "󰎙",        // nf-dev-nodejs_small
		"npm" => "󰎙",         // nf-dev-nodejs_small
		"yarn" => "󰬷",        // nf-seti-yarn
		"docker" => "🐳",     // Docker whale
		_ if filename.starts_with('.') => "󰘓", // nf-fa-eye_slash (hidden)
		_ => "📄",             // nf-fa-file_o or generic file emoji
	}
}

/// Main function - lists current directory contents with aligned columns
fn main() -> std::io::Result<()> {
	let current_dir = env::current_dir()?;
	let entries = fs::read_dir(&current_dir)?;
	
	// Create name cache for user/group resolution
	let name_cache = NameCache::new();
	
	// Collect and sort entries
	let mut file_entries = Vec::new();
	for entry in entries {
		let entry = entry?;
		match FileEntry::new(&entry, &name_cache) {
			Ok(file_entry) => file_entries.push(file_entry),
			Err(_) => continue, // Skip entries we can't read
		}
	}
	
	// Sort: directories first, then by name (case-insensitive)
	file_entries.sort_by(|a, b| {
		match (a.is_dir, b.is_dir) {
			(true, false) => std::cmp::Ordering::Less,
			(false, true) => std::cmp::Ordering::Greater,
			_ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
		}
	});
	
	if file_entries.is_empty() {
		println!(" Empty directory");
		return Ok(());
	}
	
	// Display header
	println!(" {} ({} items)", current_dir.display(), file_entries.len());
	println!();
	
	// Calculate column widths for perfect alignment
	let max_perms_len = file_entries.iter()
		.map(|entry| entry.permissions.len())
		.max()
		.unwrap_or(0);
		
	let max_owner_len = file_entries.iter()
		.map(|entry| entry.owner.len())
		.max()
		.unwrap_or(0);
		
	let max_group_len = file_entries.iter()
		.map(|entry| entry.group.len())
		.max()
		.unwrap_or(0);
		
	let max_modified_len = file_entries.iter()
		.map(|entry| entry.modified_text.len())
		.max()
		.unwrap_or(0);
	
	// Display entries one per line with aligned columns
	for entry in &file_entries {
		println!("{}", entry.format_display(max_perms_len, max_owner_len, max_group_len, max_modified_len));
	}
	
	Ok(())
} 