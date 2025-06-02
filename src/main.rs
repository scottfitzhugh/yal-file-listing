use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;
use std::env;

/// Represents a file system entry with display information
#[derive(Debug)]
struct FileEntry {
	name: String,
	permissions: String,
	owner: u32,
	group: u32,
	icon: &'static str,
	is_dir: bool,
}

impl FileEntry {
	/// Create a new FileEntry from a directory entry
	fn new(entry: &fs::DirEntry) -> std::io::Result<Self> {
		let metadata = entry.metadata()?;
		let file_name = entry.file_name().to_string_lossy().to_string();
		
		// Get permissions in octal format
		let mode = metadata.permissions().mode();
		let permissions = format!("{:o}", mode & 0o777);
		
		// Get owner and group IDs
		let owner = metadata.uid();
		let group = metadata.gid();
		
		let is_dir = metadata.is_dir();
		let icon = get_file_icon(&file_name, is_dir);
		
		Ok(FileEntry {
			name: file_name,
			permissions,
			owner,
			group,
			icon,
			is_dir,
		})
	}
	
	/// Format this entry for display with proper column alignment
	fn format_display(&self, max_perms_len: usize, max_owner_len: usize, max_group_len: usize) -> String {
		// Use ANSI escape codes for colors
		let (name_color, reset) = if self.is_dir {
			("\x1b[34;1m", "\x1b[0m") // Blue bold for directories
		} else {
			("", "") // No color for files
		};
		
		format!(
			"{} \x1b[33m{:>width_perms$}\x1b[0m \x1b[32m{:>width_owner$}\x1b[0m \x1b[36m{:>width_group$}\x1b[0m {}{}{}",
			self.icon,
			self.permissions,
			self.owner,
			self.group,
			name_color,
			self.name,
			reset,
			width_perms = max_perms_len,
			width_owner = max_owner_len,
			width_group = max_group_len
		)
	}
}

/// Get an appropriate icon for the file type
fn get_file_icon(filename: &str, is_dir: bool) -> &'static str {
	if is_dir {
		return "📁";
	}
	
	// Get file extension
	let extension = Path::new(filename)
		.extension()
		.and_then(|ext| ext.to_str())
		.unwrap_or("")
		.to_lowercase();
	
	match extension.as_str() {
		"rs" => "🦀",
		"py" => "🐍",
		"js" | "ts" => "⚡",
		"html" | "htm" => "🌐",
		"css" => "🎨",
		"json" => "📊",
		"md" | "markdown" => "📝",
		"txt" => "📄",
		"pdf" => "📕",
		"zip" | "tar" | "gz" | "rar" => "📦",
		"jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" => "🖼️",
		"mp3" | "wav" | "flac" | "ogg" => "🎵",
		"mp4" | "mkv" | "avi" | "mov" => "🎬",
		"exe" | "bin" => "⚙️",
		"toml" | "yaml" | "yml" | "ini" | "conf" => "⚙️",
		_ if filename.starts_with('.') => "👻",
		_ => "📄",
	}
}

/// Main function - lists current directory contents with aligned columns
fn main() -> std::io::Result<()> {
	let current_dir = env::current_dir()?;
	let entries = fs::read_dir(&current_dir)?;
	
	// Collect and sort entries
	let mut file_entries = Vec::new();
	for entry in entries {
		let entry = entry?;
		match FileEntry::new(&entry) {
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
		println!("📭 Empty directory");
		return Ok(());
	}
	
	// Display header
	println!("📂 {} ({} items)", current_dir.display(), file_entries.len());
	println!();
	
	// Calculate column widths for perfect alignment
	let max_perms_len = file_entries.iter()
		.map(|entry| entry.permissions.len())
		.max()
		.unwrap_or(0);
		
	let max_owner_len = file_entries.iter()
		.map(|entry| entry.owner.to_string().len())
		.max()
		.unwrap_or(0);
		
	let max_group_len = file_entries.iter()
		.map(|entry| entry.group.to_string().len())
		.max()
		.unwrap_or(0);
	
	// Display entries one per line with aligned columns
	for entry in &file_entries {
		println!("{}", entry.format_display(max_perms_len, max_owner_len, max_group_len));
	}
	
	Ok(())
} 