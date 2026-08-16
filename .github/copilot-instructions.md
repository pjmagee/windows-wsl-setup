Read and follow [AGENTS.md](../AGENTS.md). That file is the only playbook for bootstrapping and changing this WSL workstation.

You are often on a Windows 11 work laptop with no Linux toolchain yet. Detect Windows vs WSL first. On Windows, run `windows/bootstrap.ps1` (passwordless Ubuntu 26.04, Terminal `wsl`/`ubuntu` profiles, then `install.sh` inside the distro). Do not invent a second installer. At work, Copilot is the operator — do not tell the user to run Grok or Claude.
