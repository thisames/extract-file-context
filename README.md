# 📦 Extract File Context

Ferramenta desktop para extrair o contexto completo de um projeto de código em um único arquivo de texto — ideal para enviar para LLMs (ChatGPT, Claude, Gemini, etc.).

![Rust](https://img.shields.io/badge/Rust-🦀-orange?style=flat-square)
![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Linux%20%7C%20Windows-blue?style=flat-square)
![License](https://img.shields.io/badge/License-MIT-green?style=flat-square)

## ✨ Funcionalidades

- 📂 **Árvore de arquivos** — Navegue e selecione arquivos com checkboxes
- 🔍 **Busca por nome** — Filtre arquivos em tempo real
- 🌐 **Filtro por linguagem** — Selecione apenas Python, Rust, TypeScript, etc.
- 📋 **Copiar para clipboard** — Um clique para copiar tudo
- 🗜️ **Copiar minificado** — Versão compacta para economizar tokens em LLMs
- 💾 **Salvar em arquivo** — Exporte em Markdown ou texto puro
- 🗂️ **Árvore do projeto** — Inclui estrutura visual no resultado
- 📊 **Contagem de linhas** — Mostra total de linhas extraídas
- ⚙️ **Configurável** — Pastas/extensões ignoradas, filtros personalizados
- 👁️ **Preview** — Visualize arquivos antes de extrair

## 📥 Download

Vá em [**Releases**](../../releases) e baixe o executável para seu sistema:

| Sistema | Arquivo |
|---------|---------|
| 🍎 macOS (Apple Silicon) | `extract-file-context-aarch64-apple-darwin.tar.gz` |
| 🍎 macOS (Intel) | `extract-file-context-x86_64-apple-darwin.tar.gz` |
| 🐧 Linux | `extract-file-context-x86_64-unknown-linux-gnu.tar.gz` |
| 🪟 Windows | `extract-file-context-x86_64-pc-windows-msvc.zip` |

> **macOS:** Se aparecer aviso de segurança, clique com botão direito → Abrir, ou execute: `xattr -d com.apple.quarantine extract-file-context`

## 🚀 Como usar

1. Abra o app
2. Clique em **📂 Escolher pasta…** ou cole o caminho do projeto
3. Selecione/desmarque arquivos na árvore
4. (Opcional) Ative o **filtro por linguagem** no painel direito
5. Clique em **🚀 Extrair arquivos selecionados**
6. Use **📋 Copiar tudo** ou **🗜️ Copiar mini** para colar na LLM

## 🛠️ Compilar do código-fonte

### Pré-requisitos

- [Rust](https://rustup.rs/) (1.75+)

### Build

```bash
git clone https://github.com/thisames/extract-file-context.git
cd extract-file-context
cargo run --release
```

O binário será gerado em `target/release/extract-file-context`.

## 📸 Screenshot

Ao abrir o app, você verá:

```
┌─────────────────────────────────────────────────────────────────┐
│ Pasta do projeto: [________________] ▶ Carregar  📂 Escolher   │
├──────────────────┬──────────────────┬───────────────────────────┤
│ Arquivos         │ Resultado        │ ⚙️ Configurações          │
│ ✅ Tudo ❌ Nada  │                  │ 📁 Pastas ignoradas       │
│ 🔍 [buscar...]   │                  │ 🚫 Extensões ignoradas    │
│                  │                  │ 🌐 Filtrar por linguagem  │
│ ☑ 📁 src         │  (preview ou     │ ⚙️ Opções de saída        │
│   ☑ 📄 main.rs   │   resultado)     │                           │
│   ☑ 📄 lib.rs    │                  │ 🚀 Extrair selecionados   │
│ ☑ 📄 Cargo.toml  │                  │                           │
├──────────────────┴──────────────────┴───────────────────────────┤
│ Status: 12 arquivo(s) selecionado(s)                            │
└─────────────────────────────────────────────────────────────────┘
```

## 📄 Licença

[MIT](LICENSE)

