# Local Recorder Assistant - System Architecture & Design Document

## 1. Project Vision & Architecture Overview
A privacy-first, fully offline desktop application built for Linux Mint (Ubuntu/Debian-based x86_64). The app automatically monitors USB voice recorder device mounts, transcribes audio to text with native CUDA acceleration, performs local speaker diarization (who spoke when), and generates structured summaries via local LLM APIs (Ollama/LM Studio).

### Key Architectural Decisions
- **Framework**: Tauri 2.0 (Rust backend + Web view) for zero-bloat UI, fast startup, and native system integration.
- **Frontend**: Vue 3 + TypeScript + Tailwind CSS + Pinia (Modern dark-mode minimalist media interface).
- **Internationalization (i18n)**: `vue-i18n` for dynamic runtime language switching (English / Simplified Chinese / German).
- **STT Engine**: Native Rust bindings to C++ via `whisper-rs` (`whisper.cpp` engine with CUDA enabled). No Python runtime required.
- **Diarization Engine**: Native `ort` (ONNX Runtime C++ Rust API) executing `Cam++` / `ResNet34` speaker embedding models directly on NVIDIA CUDA Execution Provider.
- **Summarization Integration**: Asynchronous HTTP client calling local OpenAI-compatible / Ollama endpoints (`http://localhost:11434` or `http://localhost:1234`).
- **Project Management System**: Embedded SQLite database (`rusqlite`) for session tagging, project workspaces, status tracking, search, and history management.
- **Target OS**: Linux Mint 21/22 x86_64 (NVIDIA RTX 4060 8GB VRAM, CUDA 12.x installed).

---

## 2. Hardware & VRAM Allocation Strategy
**Hardware Specs**: NVIDIA RTX 4060 (8GB VRAM), 64GB System RAM, Intel i5-13th Gen.

### VRAM Budgeting (Max 8.0 GB)
| Component | Engine / Model | Target VRAM | Lifecycle |
| :--- | :--- | :--- | :--- |
| System / Display | X11 / Wayland Desktop | ~0.5 GB | Always On |
| STT Engine | `whisper.cpp` (`large-v3-turbo-q5_0`) | ~1.4 GB | Loaded during STT, drop/unload optional |
| Diarization Engine | `Cam++` ONNX (CUDA EP) | ~0.3 GB | Transient execution |
| Local LLM | `gemma4:e4b` / `qwen2.5:14b` via Ollama | ~3.5 - 4.5 GB | Resident in VRAM (`keep_alive`) |
| **Total VRAM Peak** | | **~5.7 - 6.7 GB** | **Safe below 8.0 GB ceiling** |

---

## 3. Core Modules & Component Architecture

```
+---------------------------------------------------------------------------------------+
|                               Tauri Frontend (Vue 3)                                  |
|   +--------------------+   +-------------------+   +--------------------+             |
|   | USB Mount Detector |   | Audio Drag & Drop |   | Language Selector  |             |
|   +--------------------+   +-------------------+   +--------------------+             |
|   | Project Workspace  |   | Speaker Timeline  |   | Markdown Viewer    |             |
|   +--------------------+   +-------------------+   +--------------------+             |
|   | History / Search   |   | Settings Panel    |   | Model Downloader   |             |
+---------------------------------------------------------------------------------------+
| IPC (Tauri Commands)
v
+---------------------------------------------------------------------------------------+
|                                Tauri Rust Core App                                    |
|                                                                                       |
|  [USB Watcher Service] -----> Scans `/media/$USER/*` & `/run/media/*`                 |
|                                                                                       |
|  [Project DB Manager] ------> Embedded SQLite DB (`app.db`) for Workspace / Metadata  |
|                                                                                       |
|  [Pipeline Controller] --+--> 1. Audio Decoders (symphonia/hound)                     |
|                          |--> 2. STT Engine (whisper-rs + CUDA)                       |
|                          |--> 3. Speaker Diarization (ort + Cam++)                     |
|                          |--> 4. Text Merger & Formatter                              |
|                          `--> 5. LLM Summarizer (reqwest -> Ollama)                   |
|                                                                                       |
|  [Storage Engine] ---------> Saves structured `.md` and updates SQLite Project records |
+---------------------------------------------------------------------------------------+
```

### Module 1: USB Mount & File System Monitor (`src-tauri/src/services/usb_watcher.rs`)
- Continuously or periodically (every 3 seconds) inspects `/media/$USER/` and `/run/media/$USER/` for newly mounted removable block devices.
- Filters target files by extensions: `.mp3`, `.wav`, `.m4a`, `.aac`, `.flac`.
- Each file carries metadata: path, name, size, modified time, and probed audio duration
  (`symphonia` header probe, no full decode). The UI renders these in a fixed-height scroll list.
- Emits Tauri IPC Event `usb-device-attached` containing file paths and metadata.

### Module 2: Audio Preprocessing (`src-tauri/src/audio/decoder.rs`)
- Converts arbitrary audio streams into normalized 16kHz Mono IEEE Float32 PCM samples (required input format for Whisper).
- Uses native Rust audio decoding crates (`symphonia` / `hound`).

### Module 3: Native STT Pipeline (`src-tauri/src/stt/whisper.rs`)
- Wraps `whisper-rs` initialized with `WhisperContextParameters` targeting CUDA device 0.
- Model path: `~/.config/local-recorder/models/ggml-large-v3-turbo-q5_0.bin`.
- Supports forced or automatic language detection (`auto`, `zh`, `en`, `de`).
- Outputs time-stamped text segments: `[{ start_ms, end_ms, text, detected_language }]`.

### Module 4: Speaker Diarization Pipeline (`src-tauri/src/diarization/onnx.rs`)
1. **Voice Activity Detection (VAD)**: Uses energy-based sliding window or silero-vad ONNX to extract active speech intervals.
2. **Feature Extraction**: Feeds speech intervals into `Cam++.onnx` via `ort` using `CUDAExecutionProvider`.
3. **Clustering**: Calculates cosine similarity distance matrix across voice embedding vectors and applies Agglomerative Hierarchical Clustering (AHC) in Rust to label speakers as `Speaker 0`, `Speaker 1`, etc.
4. **Alignment**: Merges speaker boundaries with Whisper STT time intervals to produce structured transcript:
   `[00:01:12] Speaker 0: "Hello everyone, let's start the meeting."`
   - Persisted Whisper segments (`recordings.transcript_segments`) are reused when the frontend does
     not supply them, so diarization works across restarts. If no transcript exists, the speaker
     turns are returned without text (speaker-only timeline).

### Module 5: LLM Summarization Client (`src-tauri/src/llm/client.rs`)
- Sends HTTP POST requests to Ollama API (`http://127.0.0.1:11434/api/generate`) or custom OpenAI endpoints.
- Dynamically adapts system prompt according to the user's selected UI target language.
- The Settings panel stages provider/endpoint/model edits, requires confirmation before applying,
  tests provider reachability, and enumerates models from the running server
  (Ollama `/api/tags`, OpenAI-compatible `/models`).
- System Prompt Template (Localized):
  ```text
  You are an expert executive assistant. Summarize the following transcript in {TARGET_LANGUAGE}.
  Output structured Markdown containing:
  1. Executive Summary (3-5 sentences)
  2. Key Topics & Discussions (Grouped logically)
  3. Action Items per Speaker (With responsibilities and tasks)

  Transcript:
  {formatted_transcript_with_speakers}
  ```

### Module 6: Project Management Engine (`src-tauri/src/db/project_manager.rs`)
- Uses embedded `rusqlite` database (`~/.config/local-recorder/data/app.db`).
- SQLite Database Schema:
```sql
CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS recordings (
    id TEXT PRIMARY KEY,
    project_id TEXT,
    file_name TEXT NOT NULL,
    source_path TEXT NOT NULL,
    audio_duration_secs REAL,
    detected_language TEXT,
    status TEXT CHECK(status IN ('pending', 'transcribing', 'summarizing', 'completed', 'failed')),
    transcript_raw TEXT,
    summary_markdown TEXT,
    transcript_segments TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recording_id TEXT,
    tag_name TEXT NOT NULL,
    FOREIGN KEY(recording_id) REFERENCES recordings(id) ON DELETE CASCADE
);
```

- Backend Rust Command API:
- `create_project(name, description)`
- `list_projects()`
- `assign_recording_to_project(recording_id, project_id)`
- `search_recordings(query_keyword, project_id, tag_filter)`
- `delete_recording(recording_id)`

---

## 4. Multi-Language (i18n) & Localization Strategy

### Language Support
- **Interface Languages**: English (`en`), Simplified Chinese (`zh-CN`), German (`de`).
- **Audio Recognition Languages**: Auto-Detect (`auto`), Chinese (`zh`), English (`en`), German (`de`), French (`fr`), Spanish (`es`), Japanese (`ja`).

### i18n Architecture (`src/i18n/`)
- Frontend utilizes `vue-i18n` with local JSON translation bundles (`en.json`, `zh-CN.json`, `de.json`).
- Language selector dropdown located in the global navigation bar with persistent storage in `localStorage` & SQLite user settings.
- Transmitting target output language parameter to LLM API ensuring generated summaries match user preference.

---

## 5. Model Download & Configuration Management

Applications will **not** bundle large model files inside installation packages.

- **Configuration Directory**: `~/.config/local-recorder/`
- **Models Directory**: `<project>/models/` (downloaded locally; overridable via `LRA_MODELS_DIR`)
- **Outputs Directory**: `~/Documents/Recordings_Summary/`

### Downloadable Models Configuration Schema (`models.json`)

```json
{
  "stt_models": [
    {
      "id": "whisper-large-v3-turbo-q5",
      "name": "Whisper Large v3 Turbo (Recommended)",
      "size_mb": 1100,
      "url": "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo-q5_0.bin",
      "filename": "ggml-large-v3-turbo-q5_0.bin"
    }
  ],
  "diarization_models": [
    {
      "id": "camplusplus-onnx",
      "name": "Cam++ Speaker Embedding Model (3D-Speaker)",
      "size_mb": 27,
      "url": "https://huggingface.co/csukuangfj/speaker-embedding-models/resolve/main/3dspeaker_speech_campplus_sv_zh-cn_16k-common.onnx",
      "filename": "camplusplus.onnx"
    }
  ]
}
```

---

## 6. UI/UX Interface Requirements (Vue 3 + Tailwind)

### Main Layout
1. **Sidebar / Left Navigation**:
- **Language Selector Component**: Global dropdown (English / 简体中文 / Deutsch).
- **Project Tree View**: List workspaces (e.g., "Board Meetings", "Interview Series", "General Notes") with file counters.
- **Search & Filter Bar**: Keyword filter across raw transcripts, AI summaries, and custom tags.

2. **Top Header**:
- Device status indicator (`USB Recorder Connected: /media/user/RECORDER` / `No Device`).
- Real-time VRAM/GPU performance widget.

3. **Center Main Panel**:
- **Dropzone**: Drag-and-drop external audio files or click USB files to queue.
- **Processing Timeline**: Visual status `[1/4] Decoding -> [2/4] STT Transcribing -> [3/4] Speaker Clustering -> [4/4] LLM Summarizing`.
- **Dual Tab View**:
  - *Tab 1: Summary Report* (rendered Markdown preview via `marked` + `DOMPurify`, a raw Markdown
    toggle, and "Copy as Markdown").
  - *Tab 2: Interactive Transcript Timeline* (Audio player synchronized with highlighted speaker speech bubbles).

4. **Project Management Drawer**:
- Rename recordings, assign project tags, move between workspaces, or export batch `.md` / `.json` files.

---

## 7. Build & Dependency Specifications

### Linux Mint Build Requirements (`build-dependencies.sh`)

```bash
# System Native Packages
sudo apt update
sudo apt install -y build-essential cmake pkg-config libssl-dev \
    libclang-dev libasound2-dev libgtk-3-dev libwebkit2gtk-4.1-dev \
    patchelf nvidia-cuda-toolkit sqlite3 libsqlite3-dev
```

### Rust Cargo Dependencies (`src-tauri/Cargo.toml`)

```toml
[dependencies]
tauri = { version = "2.0", features = ["protocol-asset"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
whisper-rs = { version = "0.11", features = ["cuda"] }
ort = { version = "2.0", features = ["cuda"] }
symphonia = { version = "0.5", features = ["all"] }
hound = "3.5"
notify = "6.1"
walkdir = "2.4"
rusqlite = { version = "0.31", features = ["bundled"] }
uuid = { version = "1.8", features = ["v4", "serde"] }
```

---

## 8. Implementation Roadmap for Coding Agent

- [x] **Phase 1**: Scaffold Tauri 2.0 app with Vue 3, Tailwind CSS, Pinia, and `vue-i18n` setup (English / Chinese / German).
- [x] **Phase 2**: Setup SQLite database schema and implement Rust project management CRUD commands (`rusqlite`).
- [x] **Phase 3**: Implement native audio decoder and integrate `whisper-rs` with CUDA enabled.
- [x] **Phase 4**: Implement `ort` ONNX Runtime bindings for `Cam++` voice embedding and Rust-based speaker clustering.
- [x] **Phase 5**: Add USB device auto-detection under `/media/$USER/` and IPC file bridge.
- [x] **Phase 6**: Build LLM client connecting to Ollama (`http://localhost:11434`) with localized prompt generation.
- [x] **Phase 7**: Build frontend project management workspace, search/filter views, model downloader, and auto-export functionality.

---

## 9. Documentation, Versioning & Git Workflow

### Versioning
- The project follows **Semantic Versioning** (`MAJOR.MINOR.PATCH`).
- Every change bumps the **patch** version and updates the version in all three places so they stay
  in sync:
  1. `package.json` → `"version"`
  2. `src-tauri/Cargo.toml` → `[package] version`
  3. `src-tauri/tauri.conf.json` → `"version"`
- Breaking changes bump `MINOR` during the `0.x` series and must be called out in the README and this
  document.

### Git workflow
- **Remote**: `origin` → `git@github.com:yetian/stt_linux_app.git` (SSH).
- **Commits**: [Conventional Commits](https://www.conventionalcommits.org/) — `feat`, `fix`,
  `refactor`, `docs`, `chore`, `test` — with a concise subject and a body enumerating the notable
  changes (including the version bump).
- **Scope of a commit**: code + tests + version bump + README/design updates for the same change.
- **Never committed** (see `.gitignore`): build output (`dist/`, `target/`, `gen/schemas`),
  dependencies, downloaded model binaries (`models/*`), local/machine paths, and any credentials or
  secrets.

```bash
# implement change, update README/design if notable
# bump version in package.json, src-tauri/Cargo.toml, src-tauri/tauri.conf.json
git add -A
git commit -m "fix(scope): short summary"
git push origin HEAD
```

### Documentation rule
For every **notable or breaking** change, update:
- `README.md` — Features / Usage / Troubleshooting / Changelog / Backend Commands as applicable.
- `design.md` — the affected module section, and the changelog entry when architecture changes.
