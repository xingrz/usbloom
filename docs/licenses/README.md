# Supplemental dependency notices

These files supplement license texts omitted from published Cargo crates.
`index.json` associates each file with an exact dependency name and version,
a SHA256 digest, and an immutable upstream source URL. Shared license texts
are stored once under their digest.

Most entries are notices from the dependency repository at the crate's
recorded source revision. Entries marked `standard-license-text` use SPDX's
canonical text for the license declared in that crate's Cargo metadata
when no standalone upstream notice was available. For a dual-license
choice, these entries select Apache-2.0 when it is offered. Standard text
is not a substitute for package-specific copyright attribution: the
release inventory also retains declared authors and the source archive
preserves the original source headers and attribution files.

When updating dependencies, review their declared licenses, packaged
notices, source copyright headers, and upstream notice files. Add or
update the version mapping, verify the source URLs, and record the new
file hashes. Never invent copyright holders or dates. The release script
verifies these hashes and fails on dependencies without license text.

The generated THIRD_PARTY_NOTICES.txt includes all locked dependencies,
including build/test and other-platform sources. This deliberately exceeds
the set of code linked into the macOS binary. GPUI Kit's Lucide icon notice
is included from its published assets crate. Fonts are supplied by the OS,
not redistributed in the application.
