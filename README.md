# 🌿 Mintas Programming Language

Mintas is a powerful, production-ready, universal polyglot programming language built from the ground up to be a "batteries-included" solution. It combines the ease of high-level scripting with the performance of systems programming.

## 📚 Documentation

Detailed guides and API references can be found here:
**Official Docs:** [NotBeastR/mintas-docs](https://github.com/NotBeastR/mintas-docs)

---

## 🚀 Quick Start

### Installation
You can install or remove Mintas using the provided shell script:

```bash
# Install Mintas
curl -sSL https://raw.githubusercontent.com/notbeastr/mintas-scr/main/install.sh | bash
```
```bash

# Uninstall Mintas
curl -sSL https://raw.githubusercontent.com/notbeastr/mintas-scr/main/install.sh | bash -s uninstall
```
Your First Script
Create a file named hello.as:
```bash

say("Hello, Mintas!")
```

Run it directly:

```bash

mintas run hello.as
Secure Compilation
Mintas features an advanced compiler that produces AES-256 Encrypted bytecode for secure distribution.


# Compiles hello.as to encrypted hello.ms
mintas compile hello.as

# Run the encrypted bytecode
mintas run hello.ms
```

✨ Features
Expressive Syntax: Readable, human-centric keywords like say, ask, is, so, and measure.

Advanced VM: High-performance stack-based Virtual Machine with AST -> Bytecode pipeline.

Batteries-Included Stdlib: Over 50+ built-in modules, including:

Web (dew): An Express-like HTTP framework with decorators.

Networking: Native support for HTTP, TCP/UDP, SSH, FTP, and SMTP.

Databases: Built-in drivers for SQLite, PostgreSQL, and Redis.

Modern Utils: Native JSON, CSV, UUID, and AI integration.

Polyglot FFI:

c_bytes: Call C dynamic libraries (.dll/.so) directly.

node2as: Seamless Node.js interoperability.

Concurrency: Built-in support for task, worker, and schedule.

Cross-Platform: Full support for Windows, Linux, and macOS.

🤝 Contributing
Contributions make the open-source community an amazing place to learn, inspire, and create. Any contributions you make are greatly appreciated.

Fork the Project

Create your Feature Branch (git checkout -b feature/AmazingFeature)

Commit your Changes (git commit -m 'Add some AmazingFeature')

Push to the Branch (git push origin feature/AmazingFeature)

Open a Pull Request
📄 License
Distributed under the Apache License. See LICENSE for more information.
