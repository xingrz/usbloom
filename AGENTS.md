# USBloom

USBloom is a desktop USB explorer for people developing and debugging USB
devices. It shows physical bus topology and readable device descriptors,
including configurations, interfaces, alternate settings, and endpoints.
macOS is the primary validation platform; Windows and Linux are build
targets whose runtime support must be reported honestly.

## Architecture

- Build the application in Rust with GPUI and GPUI Kit. Use Kit's matching
  GPUI re-exports and pin framework versions together through Cargo.lock.
- Call cyme as a Rust crate from background tasks. Keep acquisition, data
  normalization, and UI independent; never block the window on USB I/O.
- The application is GPL-3.0-or-later. Preserve dependency notices and
  provide corresponding source with distributed binaries.
- Register OS device notifications before the initial scan. Coalesce hotplug
  bursts and changes during acquisition; do not poll on a periodic timer.
- Scanning is observational. Do not reset devices, detach drivers, claim
  interfaces, or change configurations or alternate settings.
- A failed or partial read is not an empty device. Preserve unknown values
  and expose errors without manufacturing descriptor data.
- Keep snapshots versioned and validate imported data. Never commit real
  device serial numbers, host identifiers, or private hardware captures.

## Product and design

- Use English for code, documentation, and the primary interface.
- Favor a quiet, polished desktop interface with clear visual hierarchy.
  Avoid decorative dashboards, jargon-heavy copy, and redundant labels.
- Keep normal background operation quiet. Avoid permanent reassurance
  labels or status bars; show feedback when an action finishes, a problem
  occurs, or the viewing context changes. Use tooltips for secondary help.
- Render a selector only when there is a choice; a lone value is a label.
- Use the theme monospace font for addresses, IDs, serial numbers, and raw
  descriptor codes. Keep names, explanations, and field labels proportional.
- Keep data values selectable and provide a copy context menu. Field labels
  and navigation controls must remain outside text selection.
- Use Kit's native menus for text context actions, with its platform fallback
  where native menus are unavailable.
- Follow system appearance with coherent light and dark semantic colors.
- Reserve equal disclosure space for all tree rows at the same depth.
- Make common tasks discoverable: browse, search, refresh, copy, and save
  or open a snapshot. Explain fields where the explanation is useful.
- Keep selection, expansion, and scrolling stable across refreshes.
- Show raw values alongside decoded meanings without overwhelming the
  initial view. Unknown and vendor-specific values must remain inspectable.
- README.md is for users. Put development instructions in CONTRIBUTING.md
  and architectural detail in docs/. Keep this file useful to future agents.
- Record durable conclusions only. Do not refer to conversations, prompts,
  approvals, or the history of an agent's attempts in repository content.

## Workflow

- Read the relevant code and upstream API definitions before editing.
- Use the existing Git author and committer identity; do not change it.
- Commit complete, coherent, verified changes throughout development.
  Avoid both giant end-of-task commits and trivial fragmentary commits.
- Amend corrections into the nearby relevant unpublished commit. Use
  fixup and autosquash for earlier local commits when appropriate.
- Before the initial public release, unpublished history may be rewritten.
  Use `--force-with-lease` when a push requires rewriting remote history.
- Commit messages use English Conventional Commits, for example
  `feat(explorer): show alternate settings and endpoints`.
- Keep the subject brief. Include a short body explaining why. Hard-wrap
  every subject, body, and trailer line at 75 characters or fewer.
- Add a `Co-authored-by` trailer naming each model that actually coded the
  change, with its known version. Do not invent a model identity or credit
  a model that did not participate.
- Add meaningful tests for parsing, decoding, acquisition failures, and
  state transitions. Do not add tests that merely duplicate implementation.
- Run cargo fmt, clippy, relevant tests, and builds before committing.
  Validate meaningful UI changes in the real native application, using
  computer use or GPUI test support. Inspect screenshots for layout quality.
- Use GitHub Actions for macOS, Windows, and Linux build checks. A passing
  build is not evidence of hardware or GUI validation on that platform.
- Keep dependencies locked, CI permissions minimal, and downloaded
  executables pinned and checksum verified. Never commit credentials.
- Keep public-facing release and support claims consistent with evidence.
- Release tags must match Cargo's version. Build macOS distribution assets
  from the accompanying vendored source archive and stop at a draft release.
  See docs/releasing.md; Developer ID signing and notarization are deferred.
