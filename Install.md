---
Here is the installation guide for **Igneous-md**.


### 🛠️ Installation Guide for Igneous-md

This guide covers installing the required system dependencies (Rust, GTK4, webkit-gtk) and building the application from source.

#### **Part A: Installing Dependencies for Building (Ubuntu, Debian, Arch, FreeBSD)**

Before compiling Igneous-md, you need to install Rust and the required libraries. Use the commands for your specific system.

**1. Install Rust (All Systems)**
The project requires **Rust 1.89+**. The recommended way to install and manage Rust is via `rustup`.
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Follow the on-screen instructions, then restart your shell or run:
source ~/.cargo/env
# Verify the installation:
rustc --version
# Ensure it shows version 1.89 or higher (e.g., 1.89.0)
```

**2A. Install System Dependencies (GTK4 and webkit-gtk)**

*   **Ubuntu / Debian**
    ```bash
    sudo apt update
    sudo apt install libgtk-4-dev libwebkitgtk-6.0-dev build-essential
    ```
    *(Note: The package `libwebkitgtk-6.0-dev` corresponds to the required webkit-gtk 2.3x+ libraries.)*

*   **Arch Linux**
    ```bash
    sudo pacman -Syu gtk4 webkitgtk-6.0 base-devel
    ```

*   **FreeBSD**
    ```bash
    # Install as root packages
    pkg install rust gtk4 webkit2-gtk
    ```
    *(Note: The package name `webkit2-gtk` on FreeBSD provides the necessary webkit-gtk libraries.)*

#### **Part 2B: Building and Installing from Source Code**

Once the dependencies from Part A are installed, you can build Igneous-md from source on any system, including **Gentoo** and **NetBSD**.

**For Gentoo Linux (using `emerge`)**
1.  **Ensure Rust is installed:** `emerge dev-lang/rust`
2.  **Ensure Dependencies are installed:** You need to have `gtk4` and `net-libs/webkit-gtk` (version 2.3x+) available. You may need to unmask or accept keywords for these packages if they are not stable.
    ```zsh
    # Example: Add to /etc/portage/package.accept_keywords
    echo "dev-libs/gtk4 ~amd64" >> /etc/portage/package.accept_keywords/igneous-md
    echo "net-libs/webkit-gtk ~amd64" >> /etc/portage/package.accept_keywords/igneous-md
    emerge --ask --deep --newuse --backtrack=100 dev-libs/gtk4 net-libs/webkit-gtk
    ```

**For NetBSD (using pkgsrc)**
1.  **Ensure Rust is installed:** `pkg_add rust`
2.  **Ensure Dependencies are installed:** You will need to install `gtk4` and `webkit-gtk` from pkgsrc.
    ```zsh
    cd /usr/pkgsrc/x11/gtk4  
    make configure
    make build # takes a long time -> have a cup of coffee
    make install
    make clean
    cd /usr/pkgsrc/www/webkit-gtk 
    make configure
    make build
    make install
    make clean
    ```

**General Build Steps (for all systems, including Gentoo and NetBSD)**
1.  **Clone the repository:**
    ```zsh
    git clone https://github.com/DOD-101/igneous-md.git
    cd igneous-md
    ```
2.  **Build the project with Cargo (Rust's package manager):**
    ```zsh
    # Build the main application and the viewer in release mode
    cargo build --release
    ```
3.  **Install the binaries (optional):**
    The compiled binaries will be located in `./target/release/`. You can copy them to a directory in your `PATH`, such as `/usr/local/bin`.
    ```zsh
    # Example: Install for your user only
    cp ./target/release/igneous-md ~/.local/bin/
    cp ./target/release/igneous-md-viewer ~/.local/bin/

    # Or system-wide (may require sudo)
    sudo cp ./target/release/igneous-md /usr/local/bin/
    sudo cp ./target/release/igneous-md-viewer /usr/local/bin/
    ```

### ✅ Verification
After following these steps, you should be able to run the viewer:
```bash
igneous-md-viewer
```
Or view a markdown file:
```bash
igneous-md view path/to/your/file.md
```


