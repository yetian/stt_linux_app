# Local Recorder Assistant

A privacy-first, fully offline desktop app for Linux that watches USB voice recorders,
transcribes audio with CUDA-accelerated Whisper, performs local speaker diarization,
and produces structured Markdown summaries through local LLM servers (Ollama / LM Studio).

Everything runs on your machine. No audio, transcript, or summary ever leaves the computer.

## Features

- **USB recorder watcher** — polls `/media/$USER` and `/run/media/$USER` every 3 seconds and
  surfaces new `.mp3` / `.wav` / `.m4a` / `.aac` / `.flac` files.
- **Speech-to-text** — native `whisper.cpp` via `whisper-rs` with CUDA acceleration.
- **Speaker diarization** — energy VAD, 80-bin mel filterbank, Cam++ ONNX embeddings
  (ONNX Runtime CUDA EP), cosine agglomerative clustering, transcript alignment.
- **Local summarization** — Ollama (`/api/generate`) or OpenAI-compatible (`/chat/completions`),
  with prompts localized to the selected output language.
- **Project workspace** — embedded SQLite for projects, recordings, statuses, and tags.
- **Model manager** — downloads models into a local `models/` folder with live progress.
- **Export** — writes `<recording>.md` (summary + transcript) to `~/Documents/Recordings_Summary`.
- **i18n** — English, Simplified Chinese, German UI, switchable at runtime.

## Tech Stack

| Layer | Technology |
| --- | --- |
| Shell | Tauri 2 (Rust backend, WebView UI) |
| Frontend | Vue 3 + TypeScript + Vite + Tailwind CSS v4 + Pinia + vue-i18n |
| STT | `whisper-rs` (whisper.cpp, CUDA) |
| Diarization | `ort` (ONNX Runtime, CUDA) + `rustfft` |
| Audio | `symphonia` |
| Database | `rusqlite` (bundled SQLite) |
| Summaries | `reqwest` → Ollama / OpenAI-compatible |
| Downloads | `reqwest` + `tokio` streaming |

## Requirements

- Linux Mint 21/22 (or Ubuntu/Debian x86_64)
- NVIDIA GPU with a working driver and CUDA toolkit (`nvcc`) for GPU acceleration
- Rust (stable) and Node.js 18+ / npm

### System packages

```bash
sudo apt update
sudo apt install -y build-essential cmake pkg-config libssl-dev \
    libclang-dev libasound2-dev libgtk-3-dev libwebkit2gtk-4.1-dev \
    patchelf nvidia-cuda-toolkit sqlite3 libsqlite3-dev
```

`libclang` is required to build `whisper-rs` bindings. `cmake` + `nvcc` + `patchelf` are required
for the CUDA build.

## Getting Started

```bash
npm install
npm run tauri dev
```

On first launch the app creates its SQLite database and starts the USB watcher. It contains no
models — download them from the **Models** view, or build the release bundle and ship it.

### Production build

```bash
npm run tauri build
```

## Models

Models are never bundled. They are downloaded into the local `models/` folder inside this project.
The location is resolved as follows (in order):

1. `LRA_MODELS_DIR` environment variable, if set
2. `<project>/models/` (default)

The catalog lives in [`src-tauri/models.json`](src-tauri/models.json) and includes:

- **Speech-to-text**: `ggml-large-v3-turbo-q5_0.bin` (recommended), `ggml-medium-q5_0.bin`, `ggml-small-q5_1.bin`
- **Diarization**: `camplusplus.onnx` (Cam++ speaker embeddings)

Add or edit entries there to expose new models in the UI. Downloaded binaries are git-ignored.

## Configuration and Paths

| Purpose | Path |
| --- | --- |
| Config / data | `~/.config/local-recorder/` |
| SQLite database | `~/.config/local-recorder/data/app.db` |
| Models | `<project>/models/` (or `$LRA_MODELS_DIR`) |
| Exported summaries | `~/Documents/Recordings_Summary/` |

Local LLM endpoints (configurable in **Settings**):

- Ollama: `http://127.0.0.1:11434`
- OpenAI-compatible (LM Studio): `http://127.0.0.1:1234/v1`

## Usage

1. Open **Models** and download a Whisper model and the Cam++ diarization model.
2. Add a recording:
   - Connect the recorder; the **On Device** list appears in the sidebar — click a file to add it, or
   - Drag an audio file onto the window, or
   - Assign it to a project from the sidebar.
3. Select the recording in the sidebar and run the pipeline in the workspace:
   `Transcribe` → `Identify speakers` → `Summarize` → `Export .md`.
4. Search across transcripts, summaries, and tags from the sidebar search box.

## Project Layout

```
stt_app/
├── design.md                     # Architecture and design document
├── models/                       # Downloaded model files (git-ignored)
├── src/                          # Vue 3 frontend
│   ├── api/                      # Tauri command wrappers
│   ├── components/               # UI components
│   ├── i18n/                     # vue-i18n setup + en / zh-CN / de bundles
│   ├── stores/                   # Pinia stores (settings, device, projects, pipeline, models, ui)
│   ├── views/                    # Workspace, Models, Settings
│   └── types.ts                  # Shared frontend types
└── src-tauri/
    ├── models.json               # Downloadable model catalog
    └── src/
        ├── audio/decoder.rs      # Decode to 16 kHz mono f32
        ├── stt/whisper.rs        # whisper-rs CUDA wrapper
        ├── diarization/          # VAD, fbank, ONNX embeddings, clustering, alignment
        ├── llm/client.rs         # Ollama / OpenAI client + localized prompts
        ├── db/project_manager.rs # SQLite schema and CRUD
        ├── services/             # USB watcher, model downloader
        ├── commands.rs           # Tauri commands + shared state
        └── paths.rs              # Config / models / output directories
```

## Backend Commands

| Command | Description |
| --- | --- |
| `get_app_paths` | Resolve config, data, models, and output directories |
| `get_connected_device` | Current recorder mount and audio files |
| `create_project` / `list_projects` / `delete_project` | Project management |
| `add_recording` / `list_recordings` / `delete_recording` | Recording management |
| `search_recordings` | Keyword / project / tag search |
| `transcribe_recording` | Decode + Whisper transcription (saves transcript) |
| `diarize_recording` | Embed, cluster, align speakers (saves speaker transcript) |
| `summarize_recording` | LLM summary (saves summary + marks completed) |
| `list_models` / `download_model` / `delete_model` | Model manager |
| `export_recording` | Write Markdown to the outputs directory |

## Troubleshooting

- **`failed to load whisper model`** — download a model from the Models view; confirm `models/` is
  writable or set `LRA_MODELS_DIR`.
- **Build fails resolving `libclang` / bindgen** — install `libclang-dev`; optionally set
  `WHISPER_DONT_GENERATE_BINDINGS=1`.
- **CUDA out of memory / provider errors** — free VRAM by unloading other LLMs, or lower the
  resident LLM size. Peak usage is budgeted around 6.7 GB in `design.md`.
- **No recorder detected** — verify the mount appears under `/media/$USER` or `/run/media/$USER`
  and contains a supported audio extension.

## License

No license specified.
