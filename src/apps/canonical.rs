use std::collections::HashMap;
use std::path::PathBuf;

use mime::Mime;
use once_cell::sync::Lazy;
use xdg_mime::SharedMimeInfo;

use crate::common::Handler;
use crate::Result;

use super::{MimeApps, APPS};

fn unalias_mime(db: &SharedMimeInfo, mime: &Mime) -> Mime {
    // unalias_mime_type() performs a linear scan over the list of aliases.
    // It should use a hashmap.
    if let Some(canonical) = db.unalias_mime_type(mime) {
        return canonical;
    } else {
        return mime.clone();
    }
}

fn unalias_mime_map<V>(
    db: &SharedMimeInfo,
    mime_map: HashMap<Mime, V>,
) -> HashMap<Mime, V> {
    use std::collections::hash_map::Entry;

    // map[canonical mime] (current best match, its associations)
    let mut canonical_map =
        HashMap::<Mime, (Mime, V)>::with_capacity(mime_map.len());

    /*
    Handlr's MimeApps type stores its data in HashMap<Mime, VecDeque<Handler>>,
    Unfortunately this randomizes the order of the keys (MIME types),
    when it would be useful to keep preserve the order of
    different aliases of the same MIME type.

    In the shared MIME database, many MIME types (wav, flac, mpeg, ogg, and more)
    have one or more MIME aliases equivalent to a canonical MIME type.
    Consequently, if a mimeapps.list file contains associations for
    multiple canonical/aliased MIME types,
    and you want to lookup the file associations for the canonical or aliased MIME type,
    it's unclear which MIME type's file associations should be picked,
    and different libraries (Qt vs. GLib vs. handlr/etc.) will pick different ones.

    GLib resolves a MIME type with aliases by picking the first matching alias
    in mimeapps.list (which we can't replicate because HashMap randomizes order),
    rather than preferring the canonical MIME type.
    However, GLib's dependence on mimeapps.list's order causes issues in practice,
    so it's not worth replicating.

    Our algorithm prefers canonical MIME types over non-canonical,
    but if no canonical MIME type is present, we need to pick among the aliases.
    How do we pick? (I didn't check what approach Qt takes.)

    We can't use the original order in mimeapps.list, since HashMap shuffles the items.
    So we need a way to deterministically pick an alias from a
    randomly-ordered HashMap iterator.
    I chose to use the alphabetically first alias present in mimeapps.list.

    Sorting the hashmap iterator alphabetically takes O(n log n) time,
    so I prefer to not do that.
    I instead came up with an clever (and hopefully correct) linear-time algorithm
    for picking the alphabetically first alias if no canonical MIME type is present,
    without sorting the iterator.

    ...except it's pointless because for each item in the HashMap,
    we call unalias_mime(), which calls SharedMimeInfo::unalias_mime_type(),
    which linearly searches through all 300-ish MIME aliases present on the system
    (performing a string comparison for each).
    Running O(n) linear searches will probably dwarf the O(n log n) time
    taken to sort the HashMap iterator.

    How do you count the number of MIME aliases on a system?
    The easy approach is to count the lines in /usr/share/mime/aliases
    (298 on my machine).
    The complex way is to grep for '<alias' in either /usr/share/mime/packages/ (299),
    or /usr/share/mime/ minus the packages folder (299).
    I don't know why the numbers don't line up.
    */
    for (mime, v) in mime_map.into_iter() {
        let canonical_mime = unalias_mime(db, &mime);
        if mime == canonical_mime {
            // canonical mime wins. overwrite and discard the old value.
            canonical_map.insert(canonical_mime.clone(), (canonical_mime, v));
        } else {
            // mime is an alias.
            let entry = canonical_map.entry(canonical_mime);

            // if slot empty, insert.
            // if slot comes from canonical, abort.
            // if slot comes from alias, overwrite if we're alphabetically first.
            match entry {
                Entry::Vacant(e) => {
                    e.insert((mime, v));
                }
                Entry::Occupied(mut e) => {
                    let canonical_mime = e.key();
                    let old_mime = &e.get().0;
                    if old_mime == canonical_mime {
                        // if slot contains canonical mime, do nothing.
                    } else if mime < *old_mime {
                        // if we are alphabetically before alias, overwrite and discard the old value.
                        e.insert((mime, v));
                    } else {
                        // if we are alphabetically after alias, do nothing.
                    }
                }
            }
        }
    }

    canonical_map
        .into_iter()
        .map(|(canonical, (_actual, v))| (canonical, v))
        .collect()
}

pub struct CanonicalMimeApps {
    db: SharedMimeInfo,
    mimeapps: MimeApps,
}

impl From<MimeApps> for CanonicalMimeApps {
    fn from(mimeapps: MimeApps) -> CanonicalMimeApps {
        let db = SharedMimeInfo::new();

        let added_associations =
            unalias_mime_map(&db, mimeapps.added_associations);
        let default_apps = unalias_mime_map(&db, mimeapps.default_apps);
        let system_apps = mimeapps.system_apps;

        CanonicalMimeApps {
            db,
            mimeapps: MimeApps {
                added_associations,
                default_apps,
                system_apps,
            },
        }
    }
}

impl CanonicalMimeApps {
    fn unalias(&self, mime: &Mime) -> Mime {
        unalias_mime(&self.db, mime)
    }

    pub fn add_handler(&mut self, mime: Mime, handler: Handler) {
        self.mimeapps.add_handler(self.unalias(&mime), handler)
    }

    pub fn set_handler(&mut self, mime: Mime, handler: Handler) {
        self.mimeapps.set_handler(self.unalias(&mime), handler)
    }

    pub fn remove_handler(&mut self, mime: &Mime) -> Result<()> {
        // I suppose that if adding audio/x-flac (alias) adds audio/flac (canonical) instead,
        // then removing audio/x-flac should remove audio/flac instead.
        //
        // There's no reason to remove audio/x-flac but not audio/flac,
        // because canonicalization already does so.
        // The trouble is that the user might do so anyway, not knowing it's not needed.
        self.mimeapps.remove_handler(&self.unalias(mime))
    }

    pub fn get_handler(&self, mime: &Mime) -> Result<Handler> {
        self.mimeapps.get_handler(&self.unalias(mime))
    }

    pub fn show_handler(&self, mime: &Mime, output_json: bool) -> Result<()> {
        self.mimeapps.show_handler(&self.unalias(mime), output_json)
    }

    pub fn path() -> Result<PathBuf> {
        MimeApps::path()
    }

    pub fn read() -> Result<Self> {
        Ok(Self::from(MimeApps::read()?))
    }

    pub fn save(&self) -> Result<()> {
        self.mimeapps.save()
    }

    pub fn print(&self, detailed: bool) -> Result<()> {
        self.mimeapps.print(detailed)
    }

    pub fn list_handlers() -> Result<()> {
        MimeApps::list_handlers()
    }
}

pub static CANONICAL: Lazy<CanonicalMimeApps> =
    Lazy::new(|| CanonicalMimeApps::from(APPS.clone()));
