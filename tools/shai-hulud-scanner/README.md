# Shai-Hulud Scanner

A comprehensive security tool to detect npm packages affected by the **Shai-Hulud** supply chain attack in your local environment.

## Features

- ✅ **Full PC scan by default** - Scans your entire home directory
- ✅ **Multiple package managers** - npm, yarn, pnpm, bun
- ✅ **Global packages detection** - Scans globally installed packages
- ✅ **Suspicious file detection** - Detects malicious binaries and scripts
- ✅ **All lockfile formats** - package-lock.json, yarn.lock, pnpm-lock.yaml, bun.lock
- ✅ **Offline mode** - Cache CSV for offline scanning
- ✅ **JSON output** - Machine-readable output for CI/CD integration

## Background

In November 2025, a self-propagating npm worm called "Shai-Hulud: The Second Coming" infected over 800 npm packages, including popular ones like Zapier, PostHog, and Postman. This tool helps you scan your local projects for any affected packages.

### References

- [Codebook - Shai-Huludの悪夢再び：800件のnpmパッケージが感染し、シークレットがGitHub上に流出](https://codebook.machinarecord.com/threatreport/silobreaker-cyber-alert/42718/)
- [Wiz Security IoC List](https://github.com/wiz-sec-public/wiz-research-iocs/blob/main/reports/shai-hulud-2-packages.csv)

## Installation

### Build from source

```bash
cd tools/shai-hulud-scanner
cargo build --release
```

The binary will be available at `target/release/shai-hulud-scanner`.

### Install globally (optional)

```bash
cargo install --path .
```

## Usage

### Full PC scan (default)

```bash
# Scan entire home directory (default behavior)
./shai-hulud-scanner

# This will:
# 1. Download the latest IoC list from Wiz Security
# 2. Scan all projects in your home directory
# 3. Check all globally installed packages (npm, yarn, pnpm, bun)
# 4. Detect suspicious files and binaries
```

### Scan specific directory

```bash
# Scan specific directory
./shai-hulud-scanner --scan-dir ~/projects

# Scan current directory only
./shai-hulud-scanner --current-dir
```

### Options

```
Options:
  -s, --scan-dir <SCAN_DIR>    Directory to scan (defaults to home directory)
      --current-dir            Scan current directory only (instead of home)
  -c, --csv-file <CSV_FILE>    Path to local CSV file
      --csv-url <CSV_URL>      URL to download CSV from [default: Wiz Security's GitHub]
  -o, --output <OUTPUT>        Output format: text, json [default: text]
      --verbose                Show all scanned packages
      --offline                Use cached CSV (run without --offline first)
      --skip-global            Skip global packages scan
      --skip-suspicious        Skip suspicious file detection
  -h, --help                   Print help
  -V, --version                Print version
```

### Examples

```bash
# Full scan with JSON output
./shai-hulud-scanner --output json

# Scan projects only (skip global packages)
./shai-hulud-scanner --skip-global

# Offline mode (use cached CSV)
./shai-hulud-scanner --offline

# Use local CSV file
./shai-hulud-scanner --csv-file ./custom-ioc-list.csv

# Quick scan of current project
./shai-hulud-scanner --current-dir --skip-global
```

## What it scans

### Package Files

| Package Manager | Lockfile | Supported |
|-----------------|----------|-----------|
| npm | package-lock.json | ✅ |
| npm | package.json | ✅ |
| yarn | yarn.lock | ✅ |
| pnpm | pnpm-lock.yaml | ✅ |
| bun | bun.lock | ✅ |
| bun | bun.lockb | ✅ |

### Global Packages

| Package Manager | Location | Supported |
|-----------------|----------|-----------|
| npm | `npm root -g` | ✅ |
| yarn | `yarn global dir` | ✅ |
| pnpm | `pnpm root -g` | ✅ |
| bun | `~/.bun/install/global/` | ✅ |

### Suspicious Files

The tool also scans for:
- Fake `bun` / `bunx` binaries
- TruffleHog binaries (used by Shai-Hulud)
- Suspicious postinstall scripts
- Scripts containing malicious patterns:
  - `npm whoami` / `npm publish`
  - `curl | eval` / `wget | eval`
  - `base64` decode operations
  - Suspicious child_process usage

### Skipped Directories

The following directories are automatically skipped for performance:
- `node_modules`, `.git`, `target`, `dist`, `build`
- `.next`, `out`, `.cache`, `.npm`, `.yarn`, `.pnpm-store`
- `Library`, `Applications`, `.Trash` (macOS system directories)
- `Pictures`, `Music`, `Movies`, `Downloads`

## Output

### Text output (default)

```
╔═══════════════════════════════════════════════════════════════╗
║         Shai-Hulud Supply Chain Attack Scanner               ║
║   Detecting affected npm packages in your local environment  ║
╚═══════════════════════════════════════════════════════════════╝

→ Downloading CSV from: https://raw.githubusercontent.com/...
✓ 798 affected packages loaded from CSV

→ Scanning directory: /Users/you
✓ Found 50000 package files in local projects

→ Scanning global packages...
✓ Found 25 global packages (npm/yarn/pnpm/bun)

→ Scanning for suspicious files...
✓ Checked for suspicious patterns

═══════════════════════════════════════════════════════════════
⚠ 2 potential issues detected!
═══════════════════════════════════════════════════════════════

CRITICAL FINDINGS (exact version match):
─────────────────────────────────────────

[CRITICAL] @asyncapi/parser @ 3.4.1 (Local)
   Location: /Users/you/projects/api/package.json
   File: package.json
   Affected versions: 3.4.1, 3.4.2
   ⚠ This exact version is known to be compromised!

WARNINGS (package name match, different version):
─────────────────────────────────────────────────

[WARNING] posthog-node @ 4.11.1 (npm (global))
   Location: /usr/local/lib/node_modules/posthog-node/package.json
   File: npm global
   Affected versions: 5.13.3, 5.11.3, 4.18.1

═══════════════════════════════════════════════════════════════
                          SUMMARY
═══════════════════════════════════════════════════════════════
● 1 Critical issues (exact version match)
● 1 Warnings (package name match)
● 1 issues in global packages

Scanned:
  → Local projects (package.json, lockfiles)
  → Global packages (npm, yarn, pnpm, bun)
  → Suspicious files and binaries

Recommended Actions:
  1. → Immediately review and remove/update critical packages
  2. → Rotate all secrets (API keys, tokens, SSH keys)
  3. → Check GitHub for suspicious repositories
```

### JSON output

```json
{
  "detections": [
    {
      "package": "@asyncapi/parser",
      "installed_version": "3.4.1",
      "location": "/Users/you/projects/api/package.json",
      "file_type": "package.json",
      "source": "Local",
      "affected_versions": ["3.4.1", "3.4.2"],
      "severity": "CRITICAL"
    }
  ],
  "suspicious_files": []
}
```

## Severity Levels

| Severity | Description |
|----------|-------------|
| **CRITICAL** | Exact version match with known compromised version |
| **WARNING** | Package name matches but installed version is different |
| **SUSPICIOUS** | Potentially malicious file or binary detected |

## Recommended Actions

If critical issues are found:

1. **Immediately remove or update** the affected packages
2. **Rotate all secrets** (API keys, tokens, SSH keys, etc.)
3. **Check GitHub** for suspicious repositories created under your account
4. **Review audit logs** in npm, GitHub, and cloud providers
5. **Run `npm audit`** or `bun audit` for additional checks
6. **Add `ignore-scripts=true`** to `.npmrc` or `bunfig.toml`

## Cache

The tool caches the downloaded CSV file for offline use:
- macOS: `~/Library/Caches/shai-hulud-scanner/affected-packages.csv`
- Linux: `~/.cache/shai-hulud-scanner/affected-packages.csv`
- Windows: `%LOCALAPPDATA%/shai-hulud-scanner/affected-packages.csv`

## CI/CD Integration

```yaml
# GitHub Actions example
- name: Scan for Shai-Hulud
  run: |
    ./shai-hulud-scanner --output json > scan-results.json
    if jq -e '.detections | length > 0' scan-results.json; then
      echo "⚠️ Affected packages detected!"
      exit 1
    fi
```

## License

MIT License
