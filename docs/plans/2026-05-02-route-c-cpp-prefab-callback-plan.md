# Route C C++ / CMake / Prefab Revival Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Restore a supported C++ consumer path for `oboe-rust` that can be consumed by Android game projects using public headers, CMake, and Prefab, while preserving the original project’s integration experience and keeping the main implementation work in Rust.

**Architecture:** Reintroduce the historical C++ public surface and build files from the pre-deletion commit so downstream users keep the familiar `include/oboe`, CMake, and Prefab workflow, but route as much runtime behavior as possible through Rust-owned core/backends behind that unchanged API. The restored C++ lane is hardened around the exact production targets the user requested: low-latency callback-driven audio, stable buffer control, xrun/underrun observation, route/device change handling, and a realtime callback model close to native Oboe.

**Tech Stack:** C++17, Android NDK, CMake, Prefab metadata/AAR packaging, existing Rust/JNI crates, Android Java wrapper, GitHub Actions.

---

### Task 1: Restore Route C foundation assets

**Objective:** Bring back the public C++ headers, root CMake entrypoint, and Prefab metadata scaffold.

**Files:**
- Create/restore: `CMakeLists.txt`
- Create/restore: `include/oboe/**`
- Create/restore: `prefab/**`
- Create: `tools/check-route-c-foundation.sh`
- Modify: `README.md`
- Modify: `README.zh-CN.md`

**Step 1: Write failing foundation check**

Run:

```bash
bash tools/check-route-c-foundation.sh
```

Expected: FAIL because `CMakeLists.txt`, `include/oboe`, and `prefab` are currently absent.

**Step 2: Restore minimum public surface from historical commit**

Run:

```bash
git restore --source 7129feea -- CMakeLists.txt include prefab
```

**Step 3: Update top-level docs**

- README must no longer claim the C++ public headers / CMake / Prefab path was removed forever.
- README must clearly say the restored C++ lane is intended for Android game projects and production audio validation, while the Rust/Java wrapper still exists.

**Step 4: Re-run foundation check**

Run:

```bash
bash tools/check-route-c-foundation.sh
```

Expected: PASS.

**Step 5: Commit**

```bash
git add CMakeLists.txt include prefab README.md README.zh-CN.md tools/check-route-c-foundation.sh
git commit -m "feat: restore route-c c++ foundation surface"
```

### Task 2: Re-enable live legacy C++ build path

**Objective:** Make the restored C++ lane actually buildable again instead of dormant.

**Files:**
- Restore: `src/**`
- Restore: `tests/**`
- Restore: `samples/**`
- Restore: `apps/**` (or a reduced smoke subset)
- Restore: `prefab_build.sh`
- Modify: `tools/check-public-surface.sh`
- Modify: `.github/workflows/build-ci.yml`

**Step 1: Add a failing build smoke check**

Create a CI/local script that expects:

```bash
cmake -S . -B build-legacy-cpp -DOBOE_BUILD_LEGACY_CPP=ON
```

Expected: FAIL before `src/**` and related files are restored.

**Step 2: Restore the deleted C++ source tree from `7129feea`**

**Step 3: Run CMake configure for the legacy lane**

Expected: configure succeeds.

**Step 4: Adjust public-surface policy**

The current removal-oriented checks must be replaced with checks that preserve both Route C and Rust/JNI assets.

### Task 3: Validate realtime callback-driven audio contract

**Objective:** Guarantee the restored C++ lane still exposes and exercises the realtime features the user explicitly asked for: low-latency callback-driven audio, stable buffer control, xrun/underrun observation, route/device change handling, and a callback model close to native Oboe.

**Files:**
- `include/oboe/AudioStreamCallback.h`
- `include/oboe/AudioStreamBuilder.h`
- `src/aaudio/**`
- `src/opensles/**`
- `tests/**`
- `apps/OboeTester/**`

**Step 1: Add/restore tests covering**
- data callback registration
- error/disconnect callback behavior
- routing callback behavior
- AAudio low-latency callback path

**Step 2: Verify callback docs and samples**

**Step 3: Ensure callback path remains the preferred production API in README/docs**

### Task 4: Prefab and commercial-consumer packaging hardening

**Objective:** Make the C++ lane publishable/consumable by Android game teams.

**Files:**
- `prefab/**`
- `prefab_build.sh`
- packaging docs / release workflow files
- sample consumer project docs

**Step 1: Restore/validate prefab packaging script**

**Step 2: Add consumer instructions for**
- `find_package(oboe REQUIRED CONFIG)`
- `add_subdirectory(...)`
- Prefab / Maven / JitPack consumption

**Step 3: Add CI/release verification**
- package contains headers + ABI libs
- sample consumer project links and builds

### Task 5: Production-readiness audit for game audio

**Objective:** Close the gaps for low-latency, device adaptation, and commercial use.

**Files:**
- C++ API docs
- tests / device-smoke docs
- release checklist docs

**Checklist:**
- callback path documented as RT-safe and preferred
- latency tuning / route changes / disconnect handling clearly covered
- device/NDK/API-level constraints documented
- smoke test matrix for API 26+ and main ABIs
- migration guidance for game engines (Unreal/Unity/native C++)

---

## Immediate execution target

This session should complete **Task 1** and leave the repo with a restored Route C public foundation plus updated docs. Tasks 2–5 can then be executed incrementally.
