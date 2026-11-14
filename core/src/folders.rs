//! Folder tree of a mailbox.
//!
//! Adapted from `spartanz51/tutabridge` (`crates/tuta/src/folder_system.rs`,
//! GPL-3.0), reworked for TMail: public API narrowed to the DTOs the
//! frontend needs, tests kept, and ordering documented. See `NOTICE.md`.

use std::cmp::Ordering;
use std::collections::HashMap;

use num_enum::TryFromPrimitive;
use tuta_sdk::entities::generated::tutanota::{Mail, MailSet};
use tuta_sdk::{GeneratedId, IdTupleGenerated};

/// Type of a `MailSet`, mirroring the server's `MailSetKind` enum.
#[derive(Copy, Clone, PartialEq, Eq, TryFromPrimitive, Debug, serde::Serialize)]
#[repr(u64)]
pub enum MailSetKind {
    Custom = 0,
    Inbox = 1,
    Sent = 2,
    Trash = 3,
    Archive = 4,
    Spam = 5,
    Draft = 6,
    All = 7,
    Label = 8,
    Imported = 9,
    Scheduled = 10,
    Unknown = 9999,
}

impl MailSetKind {
    #[must_use]
    pub fn is_visible_system(self) -> bool {
        matches!(
            self,
            Self::Inbox
                | Self::Sent
                | Self::Trash
                | Self::Archive
                | Self::Spam
                | Self::Draft
                | Self::Scheduled
        )
    }

    /// Stable machine name used by the UI and keyboard shortcuts.
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Self::Custom => "custom",
            Self::Inbox => "inbox",
            Self::Sent => "sent",
            Self::Trash => "trash",
            Self::Archive => "archive",
            Self::Spam => "spam",
            Self::Draft => "draft",
            Self::All => "all",
            Self::Label => "label",
            Self::Imported => "imported",
            Self::Scheduled => "scheduled",
            Self::Unknown => "unknown",
        }
    }
}

pub trait MailSetExt {
    fn kind(&self) -> MailSetKind;
    fn element_id(&self) -> Option<&GeneratedId>;
    fn is_visible_system(&self) -> bool;
}

impl MailSetExt for MailSet {
    fn kind(&self) -> MailSetKind {
        MailSetKind::try_from(self.folderType as u64).unwrap_or(MailSetKind::Unknown)
    }

    fn element_id(&self) -> Option<&GeneratedId> {
        self._id.as_ref().map(|id| &id.element_id)
    }

    fn is_visible_system(&self) -> bool {
        self.kind().is_visible_system()
    }
}

/// A folder flattened for the sidebar, with its nesting level.
#[derive(Clone, Debug, serde::Serialize)]
pub struct FolderNode {
    pub id: String,
    pub list_id: String,
    pub entries_list_id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub kind: &'static str,
    pub level: usize,
}

impl FolderNode {
    #[must_use]
    pub fn is_system(&self) -> bool {
        self.kind != "custom"
    }
}

/// Flat folder list plus the lookup maps the sync layer needs.
///
/// The server sends every `MailSet` in one flat list; parent/child links are
/// references by element id. This type rebuilds the trees once and exposes a
/// deterministic, system-first ordering for the UI.
pub struct FolderTree {
    nodes: Vec<FolderNode>,
    by_id: HashMap<String, usize>,
}

impl FolderTree {
    #[must_use]
    pub fn new(folders: Vec<MailSet>) -> Self {
        let mut children: HashMap<String, Vec<MailSet>> = HashMap::new();
        for folder in &folders {
            if let Some(parent) = &folder.parentFolder {
                children
                    .entry(parent.element_id.0.clone())
                    .or_default()
                    .push(folder.clone());
            }
        }
        for kids in children.values_mut() {
            kids.sort_by(compare_custom);
        }

        let mut system = Vec::new();
        let mut custom = Vec::new();
        for folder in folders {
            match folder.kind() {
                kind if kind.is_visible_system() => system.push(folder),
                MailSetKind::Custom if folder.parentFolder.is_none() => custom.push(folder),
                // Labels and imported sets are not folders in the sidebar.
                _ => {}
            }
        }
        system.sort_by(compare_system);
        custom.sort_by(compare_custom);

        let mut nodes = Vec::new();
        for folder in system {
            push_subtree(&children, folder, 0, &mut nodes);
        }
        for folder in custom {
            push_subtree(&children, folder, 0, &mut nodes);
        }
        let by_id = nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (node.id.clone(), index))
            .collect();
        Self { nodes, by_id }
    }

    #[must_use]
    pub fn nodes(&self) -> &[FolderNode] {
        &self.nodes
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<&FolderNode> {
        self.by_id.get(id).map(|index| &self.nodes[*index])
    }

    #[must_use]
    pub fn by_kind(&self, kind: MailSetKind) -> Option<&FolderNode> {
        self.nodes
            .iter()
            .find(|node| node.kind == kind.key() && node.is_system())
    }

    /// Inbox, or the first real folder if the account layout is unusual.
    #[must_use]
    pub fn default_folder(&self) -> Option<&FolderNode> {
        self.by_kind(MailSetKind::Inbox)
            .or_else(|| self.nodes.first())
    }

    /// Human-readable path, e.g. `Work/Projects`.
    #[must_use]
    pub fn path_of(&self, id: &str) -> String {
        let mut parts = Vec::new();
        let mut cursor = self.get(id);
        while let Some(node) = cursor {
            if !node.name.is_empty() {
                parts.push(node.name.clone());
            }
            cursor = node.parent_id.as_deref().and_then(|p| self.get(p));
        }
        parts.reverse();
        parts.join("/")
    }

    #[must_use]
    pub fn folder_of_mail(&self, mail: &Mail) -> Option<&FolderNode> {
        mail.sets.iter().find_map(|set| self.get(&set.element_id.0))
    }
}

fn push_subtree(
    children: &HashMap<String, Vec<MailSet>>,
    folder: MailSet,
    level: usize,
    out: &mut Vec<FolderNode>,
) {
    let Some(id) = folder.element_id().cloned() else {
        return;
    };
    let child_list = children.get(&id.0).cloned().unwrap_or_default();
    out.push(FolderNode {
        id: id.0.clone(),
        list_id: folder
            ._id
            .as_ref()
            .map_or_else(String::new, |i| i.list_id.0.clone()),
        entries_list_id: folder.entries.0.clone(),
        parent_id: folder.parentFolder.as_ref().map(|p| p.element_id.0.clone()),
        name: folder.name.clone(),
        kind: folder.kind().key(),
        level,
    });
    for child in child_list {
        push_subtree(children, child, level + 1, out);
    }
}

fn compare_system(a: &MailSet, b: &MailSet) -> Ordering {
    system_order(a.kind()).cmp(&system_order(b.kind()))
}

fn compare_custom(a: &MailSet, b: &MailSet) -> Ordering {
    a.name.to_lowercase().cmp(&b.name.to_lowercase())
}

fn system_order(kind: MailSetKind) -> u8 {
    match kind {
        MailSetKind::Inbox => 0,
        MailSetKind::Draft => 1,
        MailSetKind::Scheduled => 2,
        MailSetKind::Sent => 3,
        MailSetKind::Trash => 4,
        MailSetKind::Archive => 5,
        MailSetKind::Spam => 6,
        _ => 7,
    }
}

pub fn id_tuple(node: &FolderNode) -> IdTupleGenerated {
    IdTupleGenerated {
        list_id: GeneratedId(node.list_id.clone()),
        element_id: GeneratedId(node.id.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tuta_sdk::entities::generated::tutanota::MailSet;

    fn set(id: &str, kind: MailSetKind, name: &str, parent: Option<&str>) -> MailSet {
        MailSet {
            _id: Some(IdTupleGenerated {
                list_id: GeneratedId("folderlist".into()),
                element_id: GeneratedId(id.into()),
            }),
            _permissions: GeneratedId("perm".into()),
            _format: 0,
            _ownerEncSessionKey: None,
            _ownerGroup: None,
            _ownerKeyVersion: None,
            _kdfNonce: None,
            color: None,
            entries: GeneratedId(format!("entries-{id}")),
            folderType: kind as i64,
            name: name.into(),
            parentFolder: parent.map(|p| IdTupleGenerated {
                list_id: GeneratedId("folderlist".into()),
                element_id: GeneratedId(p.into()),
            }),
            _errors: Default::default(),
        }
    }

    #[test]
    fn system_first_then_custom_children() {
        let tree = FolderTree::new(vec![
            set("spam", MailSetKind::Spam, "", None),
            set("inbox", MailSetKind::Inbox, "", None),
            set("work", MailSetKind::Custom, "Work", None),
            set("proj", MailSetKind::Custom, "Projects", Some("work")),
            set("alpha", MailSetKind::Custom, "alpha", None),
        ]);
        let names: Vec<_> = tree.nodes().iter().map(|n| n.name.as_str()).collect();
        assert_eq!(names, ["", "", "alpha", "Work", "Projects"]);
        assert_eq!(tree.get("proj").unwrap().level, 1);
        assert_eq!(tree.path_of("proj"), "Work/Projects");
    }

    #[test]
    fn labels_and_imported_are_hidden() {
        let tree = FolderTree::new(vec![
            set("inbox", MailSetKind::Inbox, "", None),
            set("lbl", MailSetKind::Label, "Work", None),
            set("imp", MailSetKind::Imported, "Imported", None),
        ]);
        assert_eq!(tree.nodes().len(), 1);
        assert!(tree.get("lbl").is_none());
    }

    #[test]
    fn default_folder_prefers_inbox() {
        let tree = FolderTree::new(vec![
            set("spam", MailSetKind::Spam, "Spam", None),
            set("inbox", MailSetKind::Inbox, "Inbox", None),
        ]);
        assert_eq!(tree.default_folder().unwrap().id, "inbox");
    }
}
