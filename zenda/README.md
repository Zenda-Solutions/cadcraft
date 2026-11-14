# Zenda build of CADCraft for Work

Based on upstream v0.20.1. Feature `zenda` adds a bounded `work.events` queue
(drained by Work over the authenticated control channel), the orange Theo mascot
button at the right end of the tool bar, and Share through Work (the toolbar and
menu `share` command queues `work.share` with path, name and unsaved state; the
engine command and the document format are unchanged). Upstream builds keep
stock behavior.

Strip: no startup or manual upstream update checks (Help › Check for Updates says
Work keeps the app up to date), no Discord or getartcraft.com links in About.
The macOS native menu still has upstream's Discord item; Work ships Windows only.

Brand (`docs/brand/LICENSE-brand.txt`): the build carries no ArtCraft Marks. About
says «Basada en CADCraft, del equipo de ArtCraft» and links to this fork.

Theo's image is the existing Zenda Work mascot (`app/ui/theo/theo-icon-64.png`,
SHA-256 `87770f80…bbeb4`). Button: #EA6C1E with black text, keyboard focus and
localized AccessKit name (es/en/pt-BR). New strings are in `es.tsv` and `pt_br.tsv`.

Packaging: `pwsh zenda/package.ps1 -Z 1`; workflow `Zenda` publishes immutable
`zenda-v0.20.1-z<n>` GitHub releases. Verify size and SHA-256 before editing
Work's manifest.

## Checks

`cargo test --config 'profile.dev.package.cadcraft-ui-egui.opt-level=0' -p cadcraft-ui-egui --lib --features zenda zenda`
and the same without the feature. Set `ZENDA_THEO_PREVIEW` to a PNG path for an
offscreen button preview.
