//! What the page keeps of the app's saves (`api.rs`): `readFile` reads them back, and which
//! saves go to the user's files on the server («Мои файлы», `studio/web/studio.js`).

/// Saved files kept (the oldest go first).
pub const MAX_SAVED: usize = 16;

/// Extensions of the documents the page keeps on the server while it takes saves: the native
/// format and the vector formats it opens again (`.svg`, `.pdf`, `.ai`, `.eps`). Pictures and
/// everything else keep downloading.
pub const DOCUMENT_EXTS: &[&str] = &["vectorcraft", "drawcraft", "svg", "svgz", "pdf", "ai", "eps"];

/// Whether a save to `path` is a document the page keeps on the server.
pub fn is_document(path: &str) -> bool {
    match file_name(path).rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => DOCUMENT_EXTS.iter().any(|e| ext.eq_ignore_ascii_case(e)),
        _ => false,
    }
}

/// The last component of a `/` or `\` path.
pub fn file_name(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().filter(|n| !n.is_empty()).unwrap_or(path)
}

/// Files by name, newest last: saving a name again replaces it.
#[derive(Default)]
pub struct SavedFiles(Vec<(String, Vec<u8>)>);

impl SavedFiles {
    /// Keep `bytes` under the last component of `path`.
    pub fn keep(&mut self, path: &str, bytes: &[u8]) {
        let name = file_name(path).to_string();
        self.0.retain(|(n, _)| *n != name);
        self.0.push((name, bytes.to_vec()));
        let extra = self.0.len().saturating_sub(MAX_SAVED);
        self.0.drain(..extra);
    }

    /// A kept file by its name, or by a path ending in it.
    pub fn get(&self, name: &str) -> Option<&[u8]> {
        let wanted = file_name(name);
        self.0.iter().find(|(n, _)| n == wanted).map(|(_, b)| b.as_slice())
    }

    /// `(name, size)` of each kept file, oldest first.
    pub fn list(&self) -> impl Iterator<Item = (&str, usize)> {
        self.0.iter().map(|(n, b)| (n.as_str(), b.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_by_name_replaces_and_reads_back_by_path() {
        let mut s = SavedFiles::default();
        s.keep("exports/poster.png", b"one");
        s.keep(r"C:\out\poster.png", b"two");
        s.keep("notes.txt", b"three");
        assert_eq!(s.list().collect::<Vec<_>>(), vec![("poster.png", 3), ("notes.txt", 5)]);
        assert_eq!(s.get("poster.png"), Some(&b"two"[..]));
        assert_eq!(s.get("/exports/poster.png"), Some(&b"two"[..]));
        assert_eq!(s.get("missing.png"), None);
    }

    #[test]
    fn drops_the_oldest_past_the_cap() {
        let mut s = SavedFiles::default();
        for i in 0..MAX_SAVED + 3 {
            s.keep(&format!("f{i}.png"), &[0]);
        }
        assert_eq!(s.list().count(), MAX_SAVED);
        assert_eq!(s.get("f0.png"), None);
        assert!(s.get(&format!("f{}.png", MAX_SAVED + 2)).is_some());
    }

    #[test]
    fn documents_go_to_the_server_and_pictures_download() {
        for p in ["Логотип.vectorcraft", "old.DRAWCRAFT", "a/b/icon.svg", "icon.svgz", r"C:\x\print.pdf", "art.ai", "logo.eps"] {
            assert!(is_document(p), "{p}");
        }
        for p in ["poster.png", "poster.jpg", "notes.txt", ".svg", "svg", "", "dir/"] {
            assert!(!is_document(p), "{p}");
        }
    }

    #[test]
    fn a_trailing_separator_keeps_the_whole_name() {
        let mut s = SavedFiles::default();
        s.keep("odd/", b"x");
        assert_eq!(s.get("odd/"), Some(&b"x"[..]));
    }
}
