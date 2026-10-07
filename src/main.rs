use eframe::egui;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use walkdir::WalkDir;

#[derive(Clone, Debug)]
struct FileNode {
    name: String,
    path: PathBuf,
    size: u64,
    is_dir: bool,
    children: Vec<FileNode>,
}

#[derive(Clone, Debug)]
struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl FileNode {
    fn new(name: String, path: PathBuf, is_dir: bool) -> Self {
        Self {
            name,
            path,
            size: 0,
            is_dir,
            children: Vec::new(),
        }
    }

    fn total_size(&self) -> u64 {
        if self.children.is_empty() {
            self.size
        } else {
            self.children.iter().map(|c| c.total_size()).sum()
        }
    }
}

fn scan_directory(path: &Path, max_depth: usize) -> Option<FileNode> {
    let mut root = FileNode::new(
        path.file_name()?.to_string_lossy().to_string(),
        path.to_path_buf(),
        true,
    );

    let mut entries: Vec<(PathBuf, u64, bool)> = Vec::new();

    for entry in WalkDir::new(path)
        .max_depth(max_depth)
        .follow_links(false)
    {
        if let Ok(entry) = entry {
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            let size = if metadata.is_file() {
                metadata.len()
            } else {
                0
            };

            entries.push((
                entry.path().to_path_buf(),
                size,
                metadata.is_dir(),
            ));
        }
    }

    // Build tree structure
    for (entry_path, size, is_dir) in entries {
        if entry_path == path {
            continue;
        }

        if let Ok(relative) = entry_path.strip_prefix(path) {
            let components: Vec<_> = relative.components().collect();
            if components.len() == 1 {
                let name = entry_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let mut node = FileNode::new(name, entry_path, is_dir);
                node.size = size;
                root.children.push(node);
            }
        }
    }

    // Sort by size
    root.children.sort_by(|a, b| b.total_size().cmp(&a.total_size()));

    Some(root)
}

fn calculate_treemap(node: &FileNode, rect: Rect, min_area: f32) -> Vec<(Rect, FileNode)> {
    let mut result = Vec::new();

    if rect.width * rect.height < min_area {
        return result;
    }

    if node.children.is_empty() {
        result.push((rect, node.clone()));
        return result;
    }

    let total_size = node.total_size();
    if total_size == 0 {
        return result;
    }

    let mut current_x = rect.x;
    let mut current_y = rect.y;
    let horizontal = rect.width >= rect.height;

    for child in &node.children {
        let child_size = child.total_size();
        let ratio = child_size as f32 / total_size as f32;

        let child_rect = if horizontal {
            let width = rect.width * ratio;
            let r = Rect {
                x: current_x,
                y: current_y,
                width: width.max(1.0),
                height: rect.height,
            };
            current_x += width;
            r
        } else {
            let height = rect.height * ratio;
            let r = Rect {
                x: current_x,
                y: current_y,
                width: rect.width,
                height: height.max(1.0),
            };
            current_y += height;
            r
        };

        if child.is_dir && !child.children.is_empty() {
            result.extend(calculate_treemap(child, child_rect, min_area));
        } else {
            result.push((child_rect, child.clone()));
        }
    }

    result
}

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if size >= TB {
        format!("{:.2} TB", size as f64 / TB as f64)
    } else if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

fn get_color_for_depth(depth: usize, size_ratio: f32) -> egui::Color32 {
    let colors = [
        egui::Color32::from_rgb(100, 149, 237),  // Cornflower blue
        egui::Color32::from_rgb(72, 209, 204),   // Medium turquoise
        egui::Color32::from_rgb(144, 238, 144),  // Light green
        egui::Color32::from_rgb(255, 218, 185),  // Peach
        egui::Color32::from_rgb(221, 160, 221),  // Plum
        egui::Color32::from_rgb(255, 182, 193),  // Light pink
    ];

    let base_color = colors[depth % colors.len()];

    // Adjust brightness based on size
    let brightness = 0.7 + (size_ratio * 0.3);
    egui::Color32::from_rgb(
        (base_color.r() as f32 * brightness) as u8,
        (base_color.g() as f32 * brightness) as u8,
        (base_color.b() as f32 * brightness) as u8,
    )
}

struct DiskAnalyzerApp {
    root_node: Arc<Mutex<Option<FileNode>>>,
    current_path: String,
    scanning: bool,
    depth: usize,
    selected_node: Option<FileNode>,
}

impl Default for DiskAnalyzerApp {
    fn default() -> Self {
        Self {
            root_node: Arc::new(Mutex::new(None)),
            current_path: std::env::var("HOME").unwrap_or_else(|_| "/".to_string()),
            scanning: false,
            depth: 2,
            selected_node: None,
        }
    }
}

impl eframe::App for DiskAnalyzerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Cesta:");
                ui.text_edit_singleline(&mut self.current_path);

                ui.label("Hĺbka:");
                ui.add(egui::Slider::new(&mut self.depth, 1..=5));

                if ui.button("📂 Skenovať").clicked() && !self.scanning {
                    let path = PathBuf::from(self.current_path.clone());
                    let root_node = self.root_node.clone();
                    let depth = self.depth;
                    self.scanning = true;
                    self.selected_node = None;

                    thread::spawn(move || {
                        if let Some(node) = scan_directory(&path, depth) {
                            *root_node.lock().unwrap() = Some(node);
                        }
                    });
                }

                if self.scanning {
                    ui.spinner();
                    ui.label("Skenujem...");
                }
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            if let Some(node) = &self.selected_node {
                ui.horizontal(|ui| {
                    ui.label(format!("📄 {}", node.name));
                    ui.separator();
                    ui.label(format!("Veľkosť: {}", format_size(node.total_size())));
                    ui.separator();
                    ui.label(format!("Cesta: {}", node.path.display()));
                });
            } else {
                ui.label("Kliknite na blok pre zobrazenie detailov");
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let node_lock = self.root_node.lock().unwrap();

            if let Some(ref root) = *node_lock {
                if self.scanning {
                    self.scanning = false;
                }

                let available_size = ui.available_size();
                let rect = Rect {
                    x: 0.0,
                    y: 0.0,
                    width: available_size.x,
                    height: available_size.y,
                };

                let treemap = calculate_treemap(root, rect, 100.0);
                let total_size = root.total_size();

                let (response, painter) = ui.allocate_painter(available_size, egui::Sense::click());

                for (i, (rect, node)) in treemap.iter().enumerate() {
                    let size_ratio = node.total_size() as f32 / total_size as f32;
                    let color = get_color_for_depth(i / 10, size_ratio);

                    let rect_pos = response.rect.min + egui::vec2(rect.x, rect.y);
                    let rect_size = egui::vec2(rect.width, rect.height);
                    let egui_rect = egui::Rect::from_min_size(rect_pos, rect_size);

                    // Draw rectangle
                    painter.rect_filled(egui_rect, 2.0, color);
                    painter.rect_stroke(egui_rect, 2.0, egui::Stroke::new(1.0, egui::Color32::WHITE));

                    // Draw label if there's enough space
                    if rect.width > 60.0 && rect.height > 30.0 {
                        let text = format!("{}\n{}", node.name, format_size(node.total_size()));
                        painter.text(
                            egui_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            text,
                            egui::FontId::proportional(10.0),
                            egui::Color32::BLACK,
                        );
                    }

                    // Handle clicks
                    if response.clicked() {
                        if let Some(pointer_pos) = response.interact_pointer_pos() {
                            if egui_rect.contains(pointer_pos) {
                                self.selected_node = Some(node.clone());
                            }
                        }
                    }
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Zadajte cestu a kliknite na 'Skenovať'");
                });
            }
        });

        ctx.request_repaint();
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Disk Analyzer - Analýza obsadenosti úložiska"),
        ..Default::default()
    };

    eframe::run_native(
        "Disk Analyzer",
        options,
        Box::new(|_cc| Ok(Box::new(DiskAnalyzerApp::default()))),
    )
}
