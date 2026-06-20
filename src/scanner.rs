use crate::classify;
use crate::config::ScanConfig;
use crate::filters::{self, SkipDecision};
use crate::model::{
    ChildSummary, Classification, DirNode, ResultEntry, RootSource, ScanReport, ScanRoot, ScanStats,
};
use crate::windows;
use anyhow::Result;
use chrono::Utc;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

struct RootOutput {
    root: ScanRoot,
    node: DirNode,
    stats: ScanStats,
    warnings: Vec<String>,
}

#[derive(Clone)]
struct Candidate {
    path: PathBuf,
    display_name: String,
    size_bytes: u64,
    classification: Classification,
    root_label: String,
    root_reason: String,
    largest_children: Vec<ChildSummary>,
}

pub fn scan(config: &ScanConfig, roots: &[ScanRoot]) -> Result<ScanReport> {
    let started = Instant::now();

    let outputs: Vec<RootOutput> = roots
        .par_iter()
        .map(|root| scan_root(root.clone(), config))
        .collect();

    let mut stats = ScanStats::default();
    let mut warnings = Vec::new();
    let mut candidates = Vec::new();

    for output in &outputs {
        stats.merge(&output.stats);
        warnings.extend(output.warnings.clone());
        collect_candidates(&output.node, &output.root, config, 0, &mut candidates);
    }

    stats.elapsed_ms = started.elapsed().as_millis();
    stats.bytes_seen = outputs.iter().map(|o| o.node.size_bytes).sum();

    let mut entries = rank_candidates(candidates, config.top);
    for (index, entry) in entries.iter_mut().enumerate() {
        entry.id = index + 1;
    }

    Ok(ScanReport {
        generated_at_utc: Utc::now().to_rfc3339(),
        profile: config.profile,
        min_size_bytes: config.min_size_bytes,
        roots: roots.to_vec(),
        entries,
        stats,
        warnings,
    })
}

fn scan_root(root: ScanRoot, config: &ScanConfig) -> RootOutput {
    let started = Instant::now();
    let mut stats = ScanStats::default();
    let mut warnings = Vec::new();
    stats.roots_scanned = 1;

    let node = match scan_dir(&root.path, &root, config, &mut stats) {
        Ok(node) => node,
        Err(err) => {
            warnings.push(format!("{}: {}", root.path.display(), err));
            DirNode::empty(root.path.clone())
        }
    };

    stats.elapsed_ms = started.elapsed().as_millis();

    RootOutput {
        root,
        node,
        stats,
        warnings,
    }
}

fn scan_dir(
    path: &Path,
    root: &ScanRoot,
    config: &ScanConfig,
    stats: &mut ScanStats,
) -> Result<DirNode> {
    let mut node = DirNode::empty(path.to_path_buf());
    stats.dirs_scanned += 1;

    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(err) => {
            stats.access_errors += 1;
            return Err(err.into());
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => {
                stats.access_errors += 1;
                continue;
            }
        };

        let child_path = entry.path();
        let metadata = match if config.follow_links {
            fs::metadata(&child_path)
        } else {
            fs::symlink_metadata(&child_path)
        } {
            Ok(metadata) => metadata,
            Err(_) => {
                stats.access_errors += 1;
                continue;
            }
        };

        if metadata.is_dir() {
            match filters::should_skip_dir(&child_path, &metadata, root, config) {
                SkipDecision::Scan => {}
                SkipDecision::Skip(reason) => {
                    let _reason = reason;
                    stats.skipped_dirs += 1;
                    continue;
                }
            }

            match scan_dir(&child_path, root, config, stats) {
                Ok(child) => {
                    node.size_bytes = node.size_bytes.saturating_add(child.size_bytes);
                    node.file_count = node.file_count.saturating_add(child.file_count);
                    node.dir_count = node.dir_count.saturating_add(child.dir_count + 1);
                    node.children.push(child);
                }
                Err(_) => {
                    stats.access_errors += 1;
                }
            }
        } else if metadata.is_file() {
            let size = metadata.len();
            node.size_bytes = node.size_bytes.saturating_add(size);
            node.direct_file_bytes = node.direct_file_bytes.saturating_add(size);
            node.file_count = node.file_count.saturating_add(1);
            stats.files_scanned += 1;
        }
    }

    node.children
        .sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    Ok(node)
}

fn collect_candidates(
    node: &DirNode,
    root: &ScanRoot,
    config: &ScanConfig,
    depth: usize,
    out: &mut Vec<Candidate>,
) {
    if node.size_bytes >= config.min_size_bytes {
        let classification = classify::classify_path(&node.path, root);
        let shallow_enough = depth > 0 && depth <= config.candidate_depth;
        let targeted_root = depth == 0 && should_report_root(node, root);

        if shallow_enough || targeted_root || classification.special_candidate {
            let largest_children = node
                .largest_children(4)
                .into_iter()
                .map(|child| {
                    let child_class = classify::classify_path(&child.path, root);
                    ChildSummary {
                        name: child.name.clone(),
                        path: child.path.clone(),
                        size_bytes: child.size_bytes,
                        category: child_class.category,
                    }
                })
                .collect();

            out.push(Candidate {
                path: node.path.clone(),
                display_name: node.name.clone(),
                size_bytes: node.size_bytes,
                classification,
                root_label: root.label.clone(),
                root_reason: root.reason.clone(),
                largest_children,
            });
        }
    }

    for child in &node.children {
        collect_candidates(child, root, config, depth + 1, out);
    }
}

fn should_report_root(node: &DirNode, root: &ScanRoot) -> bool {
    match root.source {
        RootSource::SteamLibrary => false,
        RootSource::KnownFolder => false,
        RootSource::EpicManifest | RootSource::KnownModPath | RootSource::KnownUserAppPath => true,
        RootSource::KnownLauncherPath => node.children.len() <= 3,
        RootSource::Manual => true,
    }
}

fn rank_candidates(mut candidates: Vec<Candidate>, top: usize) -> Vec<ResultEntry> {
    candidates.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

    let mut entries: Vec<ResultEntry> = Vec::new();

    'candidate: for candidate in candidates {
        for selected in &entries {
            if windows::is_probably_same_or_child(&selected.path, &candidate.path)
                && selected.category == candidate.classification.category
                && candidate.size_bytes.saturating_mul(100)
                    >= selected.size_bytes.saturating_mul(85)
            {
                continue 'candidate;
            }
        }

        let reason = format!(
            "{}; root: {}",
            candidate.classification.reason, candidate.root_reason
        );
        entries.push(ResultEntry {
            id: 0,
            path: candidate.path,
            display_name: candidate.display_name,
            size_bytes: candidate.size_bytes,
            category: candidate.classification.category,
            confidence: candidate.classification.confidence,
            reason,
            advice: candidate.classification.advice,
            root_label: candidate.root_label,
            largest_children: candidate.largest_children,
        });

        if entries.len() >= top {
            break;
        }
    }

    entries
}
