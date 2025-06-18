# Kontour - Modern Kubernetes Desktop Manager

A powerful, desktop-based Kubernetes management tool built with Rust and Dioxus. Kontour provides a modern, intuitive interface for managing your Kubernetes clusters with native performance and reliability.

## ✨ Features

- 🚀 **Performance**: Built with Rust for fast performance and reliability
- 🎯 **Resource Management**: Comprehensive management of Kubernetes resources:
  - Deployments, StatefulSets, and DaemonSets
  - Pods and Services
  - ConfigMaps and Secrets
  - Jobs and CronJobs
  - Ingresses and PVCs
  - Namespaces and Nodes
- 🎨 **Modern UI**: A clean & responsive interface built with Dioxus
- 🔒 **Multiple Cluster Support**: Easily switch between different Kubernetes contexts (coming soon!)
- 💻 **Desktop-First**: Native desktop application for macOS, Windows, and Linux

## 🚀 Getting Started

### Prerequisites

- Rust toolchain (latest stable version)
- A valid kubeconfig file
- Node.js and npm (for Tailwind CSS)

### Installation

1. Clone the repository

2. Install dependencies:
```bash
cargo build
npm install
```

3. Run the application:
```bash
dx serve
```

## 🛠 Development

The project uses:
- [Dioxus](https://dioxuslabs.com/) for the UI framework
- [kube-rs](https://kube.rs/) for Kubernetes API interactions

### Project Structure

```
kontour/
├── src/
│   ├── components/    # Reusable UI components
│   │   ├── pod_item.rs
│   │   ├── deployment_item.rs
│   │   └── ...
│   ├── views/        # Main application views
│   │   ├── pods.rs
│   │   ├── deployments.rs
│   │   └── ...
│   ├── main.rs       # Application entry point
│   └── utils.rs      # Utility functions
├── assets/           # Static assets and styling
└── examples/         # Example Kubernetes configurations
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
