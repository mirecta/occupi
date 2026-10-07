use eframe::egui;
use std::f32::consts::PI;
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
struct Segment {
    node: FileNode,
    start_angle: f32,
    end_angle: f32,
    inner_radius: f32,
    outer_radius: f32,
    depth: usize,
}

struct Callout {
    line_start: egui::Pos2,
    line_end: egui::Pos2,
    text: String,
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

fn calculate_dir_size(path: &Path) -> u64 {
    let mut total = 0;
    for entry in WalkDir::new(path).follow_links(false).max_depth(20) {
        if let Ok(entry) = entry {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total += metadata.len();
                }
            }
        }
    }
    total
}

fn scan_directory_level(path: &Path, depth: usize) -> Option<FileNode> {
    if depth > 2 {
        return None;
    }
    
    let metadata = std::fs::metadata(path).ok()?;
    
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    
    let mut node = FileNode::new(name, path.to_path_buf(), metadata.is_dir());
    
    if !metadata.is_dir() {
        node.size = metadata.len();
        return Some(node);
    }
    
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let child_path = entry.path();
                let child_name = child_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                
                if let Ok(child_metadata) = std::fs::metadata(&child_path) {
                    if child_metadata.is_file() {
                        let mut child_node = FileNode::new(
                            child_name,
                            child_path,
                            false,
                        );
                        child_node.size = child_metadata.len();
                        node.children.push(child_node);
                    } else if child_metadata.is_dir() {
                        if let Some(mut subnode) = scan_directory_level(&child_path, depth + 1) {
                            if subnode.children.is_empty() {
                                subnode.size = calculate_dir_size(&child_path);
                            }
                            node.children.push(subnode);
                        }
                    }
                }
            }
        }
    }
    
    node.children.sort_by(|a, b| b.total_size().cmp(&a.total_size()));
    
    Some(node)
}

fn scan_directory(path: &Path, _max_depth: usize) -> Option<FileNode> {
    scan_directory_level(path, 0)
}

fn calculate_sunburst(node: &FileNode, max_radius: f32) -> Vec<Segment> {
    let mut segments = Vec::new();
    calculate_sunburst_recursive(node, 0.0, 360.0, 80.0, max_radius, 0, &mut segments);
    segments
}

fn calculate_sunburst_recursive(
    node: &FileNode,
    start_angle: f32,
    end_angle: f32,
    inner_radius: f32,
    max_radius: f32,
    depth: usize,
    segments: &mut Vec<Segment>,
) {
    if node.children.is_empty() || depth >= 5 {
        return;
    }
    
    let total_size = node.total_size();
    if total_size == 0 {
        return;
    }
    
    let available_radius = max_radius - inner_radius;
    let levels_remaining = 5 - depth;
    let ring_width = available_radius / levels_remaining as f32;
    let outer_radius = inner_radius + ring_width;
    
    let mut current_angle = start_angle;
    let angle_range = end_angle - start_angle;
    
    for child in &node.children {
        let child_size = child.total_size();
        let ratio = child_size as f64 / total_size as f64;
        let angle_size = angle_range * ratio as f32;
        let child_end_angle = current_angle + angle_size;
        
        if angle_size > 0.2 {
            segments.push(Segment {
                node: child.clone(),
                start_angle: current_angle,
                end_angle: child_end_angle,
                inner_radius,
                outer_radius,
                depth,
            });
            
            if child.is_dir && !child.children.is_empty() && depth < 4 {
                calculate_sunburst_recursive(
                    child,
                    current_angle,
                    child_end_angle,
                    outer_radius,
                    max_radius,
                    depth + 1,
                    segments,
                );
            }
        }
        
        current_angle = child_end_angle;
    }
}

fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if size >= TB {
        format!("{:.1} TB", size as f64 / TB as f64)
    } else if size >= GB {
        format!("{:.1} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.1} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.1} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

fn get_color_for_angle(angle: f32, depth: usize) -> egui::Color32 {
    let hue = (angle / 360.0) % 1.0;
    let saturation = 0.75 - (depth as f32 * 0.08);
    let value = 0.88 - (depth as f32 * 0.08);
    let (r, g, b) = hsv_to_rgb(hue, saturation, value);
    egui::Color32::from_rgb(r, g, b)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let h = h * 6.0;
    let i = h.floor() as i32;
    let f = h - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    
    let (r, g, b) = match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    
    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

fn draw_segment(
    painter: &egui::Painter,
    center: egui::Pos2,
    segment: &Segment,
    highlighted: bool,
    max_radius: f32,
) -> Option<Callout> {
    let start_rad = (segment.start_angle - 90.0) * PI / 180.0;
    let end_rad = (segment.end_angle - 90.0) * PI / 180.0;
    let angle_diff = segment.end_angle - segment.start_angle;
    
    let n_points = ((angle_diff * 2.0).max(10.0)) as usize;
    let mut points = Vec::new();
    
    for i in 0..=n_points {
        let t = i as f32 / n_points as f32;
        let angle = start_rad + (end_rad - start_rad) * t;
        points.push(egui::Pos2::new(
            center.x + segment.outer_radius * angle.cos(),
            center.y + segment.outer_radius * angle.sin(),
        ));
    }
    
    for i in (0..=n_points).rev() {
        let t = i as f32 / n_points as f32;
        let angle = start_rad + (end_rad - start_rad) * t;
        points.push(egui::Pos2::new(
            center.x + segment.inner_radius * angle.cos(),
            center.y + segment.inner_radius * angle.sin(),
        ));
    }
    
    let base_color = get_color_for_angle(segment.start_angle, segment.depth);
    let color = if highlighted {
        egui::Color32::from_rgb(
            base_color.r().saturating_add(40),
            base_color.g().saturating_add(40),
            base_color.b().saturating_add(40),
        )
    } else {
        base_color
    };
    
    let shadow_offset = 1.5;
    let shadow_points: Vec<_> = points.iter()
        .map(|p| egui::Pos2::new(p.x + shadow_offset, p.y + shadow_offset))
        .collect();
    painter.add(egui::Shape::convex_polygon(
        shadow_points,
        egui::Color32::from_black_alpha(25),
        egui::Stroke::NONE,
    ));
    
    painter.add(egui::Shape::convex_polygon(
        points,
        color,
        egui::Stroke::new(
            if highlighted { 2.0 } else { 1.0 },
            if highlighted {
                egui::Color32::WHITE
            } else {
                egui::Color32::from_gray(30)
            },
        ),
    ));
    
    let mid_angle = ((segment.start_angle + segment.end_angle) / 2.0 - 90.0) * PI / 180.0;
    
    if angle_diff > 15.0 && (segment.outer_radius - segment.inner_radius) > 30.0 {
        let mid_radius = (segment.inner_radius + segment.outer_radius) / 2.0;
        let text_pos = egui::Pos2::new(
            center.x + mid_radius * mid_angle.cos(),
            center.y + mid_radius * mid_angle.sin(),
        );
        
        painter.text(
            text_pos,
            egui::Align2::CENTER_CENTER,
            &segment.node.name,
            egui::FontId::proportional(10.0),
            egui::Color32::BLACK,
        );
        None
    } else if angle_diff > 2.0 && segment.depth == 0 {
        let line_start = egui::Pos2::new(
            center.x + segment.outer_radius * mid_angle.cos(),
            center.y + segment.outer_radius * mid_angle.sin(),
        );
        let extension = 35.0;
        let line_end = egui::Pos2::new(
            center.x + (max_radius + extension) * mid_angle.cos(),
            center.y + (max_radius + extension) * mid_angle.sin(),
        );
        
        Some(Callout {
            line_start,
            line_end,
            text: segment.node.name.clone(),
        })
    } else {
        None
    }
}

struct DiskAnalyzerApp {
    root_node: Arc<Mutex<Option<FileNode>>>,
    current_path: String,
    scanning: bool,
    selected_node: Option<FileNode>,
    current_view_node: Option<FileNode>,
    navigation_stack: Vec<FileNode>,
    hovered_segment: Option<usize>,
}

impl Default for DiskAnalyzerApp {
    fn default() -> Self {
        Self {
            root_node: Arc::new(Mutex::new(None)),
            current_path: std::env::var("HOME").unwrap_or_else(|_| "/".to_string()),
            scanning: false,
            selected_node: None,
            current_view_node: None,
            navigation_stack: Vec::new(),
            hovered_segment: None,
        }
    }
}

impl eframe::App for DiskAnalyzerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Cesta:");
                ui.text_edit_singleline(&mut self.current_path);

                if ui.button("📂 Skenovať").clicked() && !self.scanning {
                    let path = PathBuf::from(self.current_path.clone());
                    let root_node = self.root_node.clone();
                    self.scanning = true;
                    self.selected_node = None;
                    self.current_view_node = None;
                    self.navigation_stack.clear();

                    thread::spawn(move || {
                        if let Some(node) = scan_directory(&path, 0) {
                            *root_node.lock().unwrap() = Some(node);
                        }
                    });
                }

                if self.scanning {
                    ui.spinner();
                    ui.label("Skenujem...");
                }
                
                ui.separator();
                
                if !self.navigation_stack.is_empty() {
                    if ui.button("⬅ Späť").clicked() {
                        self.current_view_node = self.navigation_stack.pop();
                        self.selected_node = None;
                    }
                }
                
                if ui.button("🏠 Koreň").clicked() {
                    self.current_view_node = None;
                    self.navigation_stack.clear();
                    self.selected_node = None;
                }
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if let Some(view_node) = &self.current_view_node {
                    ui.label(format!("📁 {}", view_node.path.display()));
                    ui.separator();
                    ui.label(format!("{}", format_size(view_node.total_size())));
                } else if let Some(root) = &*self.root_node.lock().unwrap() {
                    ui.label(format!("📁 {}", root.path.display()));
                    ui.separator();
                    ui.label(format!("{}", format_size(root.total_size())));
                }
                
                if let Some(node) = &self.selected_node {
                    ui.separator();
                    ui.label("|");
                    ui.separator();
                    ui.label(format!("Vybrané: {} ({})", node.name, format_size(node.total_size())));
                    if node.is_dir {
                        ui.label("| Dvojklik = vstup");
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let node_lock = self.root_node.lock().unwrap();

            let display_node = if let Some(ref view_node) = self.current_view_node {
                Some(view_node.clone())
            } else {
                node_lock.clone()
            };

            if let Some(ref current) = display_node {
                if self.scanning {
                    self.scanning = false;
                }

                let available_size = ui.available_size();
                let center = ui.available_rect_before_wrap().center();
                let max_radius = (available_size.x.min(available_size.y) / 2.0 - 60.0).max(220.0);

                let segments = calculate_sunburst(current, max_radius);

                let (response, painter) = ui.allocate_painter(
                    available_size,
                    egui::Sense::click()
                );

                self.hovered_segment = None;
                let mut callouts = Vec::new();
                
                for (i, segment) in segments.iter().enumerate() {
                    let highlighted = if let Some(pointer_pos) = response.hover_pos() {
                        let dx = pointer_pos.x - center.x;
                        let dy = pointer_pos.y - center.y;
                        let dist = (dx * dx + dy * dy).sqrt();
                        let mut angle = dy.atan2(dx).to_degrees() + 90.0;
                        if angle < 0.0 {
                            angle += 360.0;
                        }
                        
                        let is_hovered = dist >= segment.inner_radius 
                            && dist <= segment.outer_radius
                            && angle >= segment.start_angle 
                            && angle <= segment.end_angle;
                        
                        if is_hovered {
                            self.hovered_segment = Some(i);
                        }
                        
                        is_hovered
                    } else {
                        false
                    };
                    
                    if let Some(callout) = draw_segment(&painter, center, segment, highlighted, max_radius) {
                        callouts.push(callout);
                    }
                }
                
                // Draw callouts on top
                for callout in callouts {
                    painter.line_segment(
                        [callout.line_start, callout.line_end],
                        egui::Stroke::new(1.0, egui::Color32::from_gray(180)),
                    );
                    
                    painter.text(
                        callout.line_end,
                        egui::Align2::LEFT_CENTER,
                        &callout.text,
                        egui::FontId::proportional(9.0),
                        egui::Color32::WHITE,
                    );
                }
                
                if let Some(hovered_idx) = self.hovered_segment {
                    if let Some(segment) = segments.get(hovered_idx) {
                        if response.clicked() {
                            self.selected_node = Some(segment.node.clone());
                        }
                        
                        if response.double_clicked() && segment.node.is_dir {
                            if let Some(ref current_display) = display_node {
                                self.navigation_stack.push(current_display.clone());
                            }
                            self.current_view_node = Some(segment.node.clone());
                            self.selected_node = None;
                        }
                    }
                }
                
                painter.circle_filled(center, 80.0, egui::Color32::from_rgb(45, 45, 45));
                painter.circle_stroke(center, 80.0, egui::Stroke::new(2.0, egui::Color32::from_gray(25)));
                painter.text(
                    center,
                    egui::Align2::CENTER_CENTER,
                    format_size(current.total_size()),
                    egui::FontId::proportional(18.0),
                    egui::Color32::WHITE,
                );
            } else {
                ui.centered_and_justified(|ui| {
                    ui.heading("Zadajte cestu a kliknite na 'Skenovať'");
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
            .with_title("Disk Analyzer - Sunburst"),
        ..Default::default()
    };

    eframe::run_native(
        "Disk Analyzer",
        options,
        Box::new(|_cc| Ok(Box::new(DiskAnalyzerApp::default()))),
    )
}
