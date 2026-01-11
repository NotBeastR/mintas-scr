# Mintas Programming Language
<p align="center" style="font-size: 300px; margin: 20px 0;">
  🌿
</p>
Mintas is a powerful, production-ready, universal polyglot programming language built from the ground up to be a "batteries-included" solution for any project. It combines the ease of scripting languages with the power of system programming.

## 📚 Documentation

**Official Documentation:** [https://github.com/NotBeastR/mintas-docs/index.html](https://github.com/NotBeastR/mintas-docs/index.html)
# Install
bash install.sh

# Uninstall
bash install.sh uninstall
## ✨ Features

- **Expressive Syntax**: Clean and readable syntax with unique keywords like `say`, `ask`, `is`, `so`, and `measure`.
- **Advanced Bytecode Compiler**: Stack-based VM with `AST -> Bytecode` compilation.
- **Secure Execution**: **AES-256 Encrypted** bytecode (`.ms` files) for secure distribution.
- **Batteries-Included Stdlib**: 52+ built-in modules including:
  - **Web Framework** (`dew`): Express-like HTTP server with decorators.
  - **Networking**: HTTP, TCP/UDP, SSH, FTP, SMTP.
  - **Databases**: SQLite, PostgreSQL, Redis.
  - **System**: File system, process management, OS interactions.
  - **Modern Utils**: JSON, CSV, DateTime, Cryptography, UUID, AI integration.
- **Polyglot FFI**:
  - `c_bytes`: Load and call C dynamic libraries (`.dll`/`.so`).
  - `node2as`: Seamless Node.js interop.
- **Async & Concurrency**: Built-in `task`, `worker`, and `schedule` support.
- **Cross-Platform**: Runs on Windows, Linux, and macOS.

### Running Your First Script

Create a file named `hello.as`:

```mintas
say("Hello, Mintas!")
```

Run it:
```bash
mintas run hello.as
```

### Compiling Code

Securely compile your script to encrypted bytecode:

```bash
mintas compile hello.as
# Outputs hello.ms (AES-256 Encrypted)

mintas run hello.ms
```
## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT License.
