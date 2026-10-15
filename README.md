# Baltimore Municipal Intelligence Engine (BMIE)

A high-performance data analytics tool built in **Rust** that looks at Baltimore police misconduct settlements and crime incident records, processes them super fast using multi-threading, and shows the insights through both a terminal CLI and a clean web dashboard.

---

## What This Project Does

City leaders need a clear way to see how police misconduct and settlement payouts break down across different districts. This app takes two official Open Baltimore datasets (Gun Trace Task Force settlements and BPD GIS incidents), links them together safely, and answers 5 big questions:
1. **District Costs:** Which police districts cost the city the most money in settlements?
2. **Unit Hotspots:** Which police units and shifts have the highest concentration of matched incidents and payouts?
3. **Timelines:** How long does it actually take from when misconduct is reported to when disciplinary action happens?
4. **Repeat Officers:** Which officer IDs show up multiple times, and how much do they cost compared to the department average?
5. **District Risk Index:** A calculated score for each district combining financial liability and incident volume to help leaders allocate resources better.

---

## How It's Built (The Tech Stack)

* **Rust:** The whole core engine is written in Rust so it's crazy fast, memory-safe, and doesn't crash or lag.
* **Concurrency (Tokio & Rayon):** Instead of processing data one row at a time, it uses **Tokio** to handle background tasks and **Rayon** to split the heavy lifting across multiple CPU cores at once.
* **Clean Code Patterns:** 
  * **Pipeline:** Data flows smoothly step-by-step from raw files $\rightarrow$ parsing $\rightarrow$ matching $\rightarrow$ analysis.
  * **Builder:** Lets you easily filter results down to a specific district (like Western or Central).
  * **Strategy:** Keeps the math for each of the 5 research questions neatly separated into its own modular block.
* **Visual Dashboards:** Gives you cool ASCII charts right in your terminal when you run commands, plus a companion `index.html` web page for a clean visual view.

---

## Project Layout

```text
baltimore_intelligence/
├── Cargo.toml          # Project config and package dependencies
├── run.sh              # Quick-start script to build and run the app
├── index.html          # Visual web dashboard for city leaders
├── data/               # The normalized CSV files
│   ├── gttf.csv
│   └── incidents.csv
└── src/
    ├── main.rs         # Starting point (bridges async Tokio with Rayon)
    ├── models.rs       # Data structures for settlements and incidents
    ├── concurrency.rs  # Shared multi-threading helpers
    ├── ingestion.rs    # Loads and parses CSVs in parallel
    ├── pipeline.rs     # Matches records together safely without copying heavy data
    ├── analysis.rs     # The logic and math for all 5 research questions
    └── ui.rs           # Renders the terminal output and ASCII charts


## macOS Compilation, Execution, & Verification Guide

Follow these exact steps to compile, run, and verify the application on a macOS environment (tested on Apple Silicon and Intel Macs).

### Step 1: Install Rust (if not already installed)
Open your macOS **Terminal** and run the official `rustup` installer script:
```bash
curl --proto '=https' --tlsv1.2 -sSf [https://sh.rustup.rs](https://sh.rustup.rs) | sh
Bash
source "$HOME/.cargo/env"
Verification: Run cargo --version to confirm the Rust compiler is active.

Step 2: Navigate to the Project Directory
Open your terminal and change into the project folder:

Bash
cd /path/to/baltimore_intelligence
Step 3: Compile the Project
Build the optimized release binary using Cargo:

Bash
cargo build --release
Verification: Look for the Finished release [optimized] target(s) message. The compiled executable will be located at target/release/bmie.

Step 4: Execute the Application
You can run the application directly through Cargo or via the automation script:

Run all default analytics:

Bash
cargo run --release
Run with district filtering (e.g., Western District):

Bash
cargo run --release -- --district Western
Run via the shell script:

Bash
./run.sh
Step 5: Verify the Output
Terminal Verification: Confirm that the application outputs the header (Baltimore Municipal Intelligence Engine), displays the record counts (46 settlements, 278,239 incidents), and renders all 5 research question sections with ASCII bar charts (█).

Web Dashboard Verification: Open the companion visual dashboard in your default browser:

Bash
open index.html
