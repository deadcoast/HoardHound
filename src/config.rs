use crate::model::ScanProfile;

#[derive(Debug, Clone)]
pub struct ScanConfig {
    pub profile: ScanProfile,
    pub top: usize,
    pub min_size_bytes: u64,
    pub follow_links: bool,
    pub include_program_files: bool,
    pub show_skipped: bool,
    pub candidate_depth: usize,
}

impl ScanConfig {
    pub fn conservative_default(profile: ScanProfile, top: usize, min_size_bytes: u64) -> Self {
        Self {
            profile,
            top,
            min_size_bytes,
            follow_links: false,
            include_program_files: false,
            show_skipped: false,
            candidate_depth: 2,
        }
    }
}
