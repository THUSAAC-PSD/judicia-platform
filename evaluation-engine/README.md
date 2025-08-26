# Evaluation Engine

The evaluation engine is responsible for compiling and executing submitted code securely. It provides sandboxed execution environments to ensure submissions cannot harm the host system.

## Purpose

- **Code Compilation**: Compile submitted source code in various programming languages
- **Secure Execution**: Run compiled programs in isolated sandbox environments
- **Resource Management**: Enforce time and memory limits during execution
- **Result Collection**: Gather execution results, output, and resource usage
- **Worker Coordination**: Manage distributed evaluation across multiple worker nodes

## Key Components

### Executor
Handles the compilation and execution of submissions. Supports multiple programming languages with configurable compile and run commands.

### Coordinator
Manages the distribution of evaluation jobs across available worker nodes. Provides load balancing and fault tolerance.

### Sandbox
Implements secure execution environments using system-level sandboxing technologies to prevent malicious code from affecting the host system.

### Job Queue
Manages the queue of pending evaluations using RabbitMQ for reliable message delivery and processing.

## Supported Languages

The engine supports multiple programming languages including:
- C++17 with g++ compiler
- Python 3
- Java 11
- JavaScript (Node.js)

Additional languages can be added through configuration.

## Security

All code execution happens in isolated sandboxes with:
- Restricted file system access
- Network isolation
- Resource limits (CPU, memory, time)
- System call filtering

## Configuration

Configured through environment variables for resource limits, sandbox settings, and queue connections.