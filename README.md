
# Recall

### Capture anything. Retrieve it instantly.

Recall is a lightweight, AI-powered command-line knowledge capture and retrieval tool. Save text snippets directly from your terminal and retrieve relevant information using natural-language queries.

No need to create files, remember filenames, or search through directories. Just capture and recall.

<p align="center">
  <a href="https://github.com/Raman-Khiva/recall/releases"><img src="https://img.shields.io/github/v/release/Raman-Khiva/recall?style=flat-square" alt="Latest Release"></a>
  <a href="https://github.com/Raman-Khiva/recall/blob/main/LICENSE"><img src="https://img.shields.io/github/license/Raman-Khiva/recall?style=flat-square" alt="License"></a>
  <img src="https://img.shields.io/badge/platform-Linux%20x86__64-blue?style=flat-square" alt="Platform">
  <img src="https://img.shields.io/badge/built%20with-Rust-orange?style=flat-square" alt="Built with Rust">
</p>

---

## Why Recall?

Developers constantly encounter information worth remembering:

- Complex commands and configuration snippets
- Deployment instructions and infrastructure notes
- Error messages and their solutions
- API references and useful code snippets
- Database queries and debugging notes
- Temporary information that might be useful later

Traditional approaches require organizing files, remembering filenames, and manually searching through them.

Recall provides a simpler workflow:

**Save now. Retrieve whenever you need it.**

## Features

- **Quick capture:** Save text directly from your terminal.
- **Natural-language retrieval:** Search for previously saved information using queries.
- **Persistent storage:** Keep your snippets in a local SQLite database.
- **AI-powered retrieval:** Uses the Groq API for intelligent retrieval.
- **Lightweight CLI:** No browser, GUI, or separate application required.
- **Simple installation:** Install a precompiled binary without setting up Rust or Cargo.

## Installation

### Linux x86_64

Install Recall with a single command:

```bash
curl -fsSL https://raw.githubusercontent.com/Raman-Khiva/recall/main/install.sh | bash
```

The installer automatically:

1. Detects your operating system and architecture.
2. Downloads the appropriate release archive.
3. Verifies the SHA-256 checksum.
4. Extracts and installs the executable.
5. Configures your shell's PATH when necessary.
6. Checks whether a Groq API key is configured.

By default, Recall installs to:

```text
~/.local/bin/rc
```

You can customize the installation directory:

```bash
RECALL_INSTALL_DIR="$HOME/bin" \
curl -fsSL https://raw.githubusercontent.com/Raman-Khiva/recall/main/install.sh | bash
```

**Requirements**

- Linux x86_64
- `curl`, `tar`, `sha256sum`, and standard Unix utilities
- Bash to execute the installation script
- A Groq API key

> The installer currently supports Linux x86_64. Other operating systems and architectures are not yet supported.

## Configuration

Recall uses the Groq API for AI-powered retrieval.

### 1. Get an API key

Create an API key from the [Groq Console](https://console.groq.com/keys).

### 2. Configure your environment

Set the `GROQ_API_KEY` environment variable:

**Bash / Zsh**

```bash
export GROQ_API_KEY="your_api_key"
```

To persist it across terminal sessions, add the export to your shell configuration file.

**Fish**

```fish
set -Ux GROQ_API_KEY "your_api_key"
```

Verify that the variable is configured without printing the secret:

```bash
test -n "$GROQ_API_KEY" && echo "API key configured"
```

Keep your API key private. Never commit it to your repository or share it publicly.

## Usage

### Save a snippet

Use `-s` to save text:

```bash
rc -s "Your text to save"
```

Example:

```bash
rc -s "To inspect Kubernetes pods across all namespaces, use kubectl get pods -A"
```

### Retrieve information

Use `-g` to retrieve relevant information:

```bash
rc -g "Your search query"
```

Example:

```bash
rc -g "How do I list pods across all Kubernetes namespaces?"
```

Recall searches your saved knowledge and returns relevant information.

### Check version

```bash
rc --version
```

### Get help

```bash
rc --help
```

## Example Workflow

```bash
# Capture useful information
rc -s "Docker Compose can restart services using docker compose restart"

rc -s "To inspect listening TCP ports, use ss -tulnp"

rc -s "Kubernetes service types include ClusterIP, NodePort and LoadBalancer"

# Retrieve it later
rc -g "How can I restart Docker Compose services?"

rc -g "How do I inspect listening ports on Linux?"

rc -g "What are the different Kubernetes service types?"
```

No need to remember which snippet you saved or where you stored it.

## How It Works

Recall combines a lightweight local storage layer with AI-powered retrieval.

```text
                 User
                  |
                  v
             Recall CLI
                  |
          +-------+-------+
          |               |
          v               v
      Save (-s)       Retrieve (-g)
          |               |
          v               v
       SQLite         Groq API
       Database           |
                          v
                    Retrieval result
```

**SQLite** provides persistent local storage for captured information.

**Groq API** powers the AI-assisted retrieval workflow.

## Technology Stack

| Component | Technology |
|---|---|
| Programming language | Rust |
| Database | SQLite |
| AI inference | Groq API |
| Distribution | Precompiled Linux binary |

## Uninstallation

Remove the Recall executable:

```bash
rm -f ~/.local/bin/rc
```

If you installed Recall into a custom directory, remove the executable from that location instead.

Your SQLite database and saved snippets are separate from the executable. Removing the binary does not necessarily remove your saved data.

## Roadmap

- [x] CLI-based text capture
- [x] SQLite persistence
- [x] AI-powered retrieval
- [x] Linux x86_64 release
- [x] Automated installation script
- [ ] Linux ARM64 support
- [ ] macOS support
- [ ] Improved retrieval and ranking
- [ ] Interactive terminal interface
- [ ] Clipboard integration

## Contributing

Contributions, bug reports, and feature suggestions are welcome.

1. Fork the repository.
2. Create a feature branch.
3. Make your changes.
4. Submit a pull request.

For bugs and feature requests, open an issue on [GitHub](https://github.com/Raman-Khiva/recall/issues).

## License

See [LICENSE](LICENSE) for licensing information.

---

<p align="center">
  <strong>Recall — Your terminal's external memory.</strong>
  <br>
  Capture less effort. Remember more.
</p>
