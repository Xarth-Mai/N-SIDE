# Parry collision queries

- Package: [`parry3d` 0.30.2](https://crates.io/crates/parry3d/0.30.2), pinned in `game/Cargo.toml` and `game/Cargo.lock`
- Author: Sébastien Crozet
- Repository: [dimforge/parry](https://github.com/dimforge/parry)
- License: Apache-2.0; complete upstream [LICENSE](LICENSE), including `Copyright 2020 Sébastien Crozet`
- Registry archive SHA-256: `01b00bf3ea4e0961a3f44aeb666b7dc3b0e87027b629561bd8283a7f6b2e2ed1`; the cached `.crate` archive matches the lockfile checksum
- Package VCS metadata: `git.sha1 = 1be4b1a7cd0a090bd7efb1207b7bc0d453f4132e`, `git.dirty = true`, `path_in_vcs = crates/parry3d`

N:SIDE calls the released crate's `TriMesh`, `CompositeShapeRef`, capsule and sphere sweep APIs from `game/src/world/collision.rs`. The integration derives static collision geometry from the same generated meshes used by the renderer; no Parry implementation source is copied or patched locally

The published crate declares Apache-2.0 in `Cargo.toml` but does not include a LICENSE file. The retained license was downloaded without modification from [LICENSE at the recorded VCS revision](https://raw.githubusercontent.com/dimforge/parry/1be4b1a7cd0a090bd7efb1207b7bc0d453f4132e/LICENSE); its SHA-256 is `ceacfa4d7fa67df64ab09a56fe248c50a3bfc9bc00374d69bd74ad763b14b89c`

The package's `dirty: true` marker means the recorded Git revision does not establish byte-for-byte equality with the published source. The registry archive checksum identifies the dependency actually built; the Git revision above identifies the license source and the publisher's recorded base

On 2026-09-28, the complete recursive Git tree for that revision contained 541 entries with `truncated: false` and no NOTICE-named file; the registry archive also contained none. No upstream NOTICE is invented. Distributions that contain the linked library should include this LICENSE and attribution; other dependencies retain their own terms
