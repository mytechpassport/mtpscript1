# MTPScript

A deterministic programming language designed for serverless deployment with guaranteed reproducible builds and runtime isolation. Compiles to an intermediate representation and executes in a hardened, embeddable runtime engine.

## How to Compile the Language

### Prerequisites
- GCC compiler
- MySQL development libraries (`mysql_config`)
- OpenSSL 1.1+ development libraries
- cURL development libraries
- GNU Make

### Building from Source

```bash
# Clone the repository
git clone <repository>
cd mtpscript

# Build the compiler and runtime
make

# Optional: Run tests to verify build
make test
```

This creates:
- `mtpsc` - MTPScript compiler, type checker, and development tools
- `mtpjs` - High-performance runtime engine for executing compiled code

### Cross-Compilation

For Windows builds:
```bash
make CONFIG_WIN32=y
```

For 32-bit compatibility:
```bash
make CONFIG_WIN32=y CONFIG_X86_32=y
```

### Development Builds

For faster development builds with debug symbols:
```bash
make CONFIG_SMALL=  # Full optimization off
```

## How to Install as Daemon

### Systemd Service (Linux)

Create a systemd service file at `/etc/systemd/system/mtpscript.service`:

```ini
[Unit]
Description=MTPScript Runtime Daemon
After=network.target

[Service]
Type=simple
User=mtpscript
Group=mtpscript
ExecStart=/usr/local/bin/mtpjs -b /var/lib/mtpscript/app.msqs
Restart=always
RestartSec=5
Environment=NODE_ENV=production
LimitNOFILE=65536
MemoryLimit=512M

# Security hardening
NoNewPrivileges=yes
PrivateTmp=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/var/lib/mtpscript
ReadOnlyPaths=/usr/local/bin/mtpjs

[Install]
WantedBy=multi-user.target
```

Install and enable the service:
```bash
# Copy binaries
sudo cp mtpsc mtpjs /usr/local/bin/

# Create user and directories
sudo useradd -r -s /bin/false mtpscript
sudo mkdir -p /var/lib/mtpscript
sudo chown mtpscript:mtpscript /var/lib/mtpscript

# Copy your snapshot
sudo cp app.msqs /var/lib/mtpscript/

# Install and start service
sudo systemctl daemon-reload
sudo systemctl enable mtpscript
sudo systemctl start mtpscript

# Check status
sudo systemctl status mtpscript
```

### Docker Container

Create a `Dockerfile`:
```dockerfile
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache libcrypto1.1 libssl1.1

# Create app user
RUN adduser -D -s /bin/sh mtpscript

# Copy binaries and snapshot
COPY mtpjs /usr/local/bin/
COPY app.msqs /app/

# Set permissions
RUN chown mtpscript:mtpscript /app/app.msqs

# Switch to non-root user
USER mtpscript

# Expose port (if serving HTTP)
EXPOSE 8080

# Run the daemon
CMD ["/usr/local/bin/mtpjs", "-b", "/app/app.msqs"]
```

Build and run:
```bash
docker build -t mtpscript-app .
docker run -d -p 8080:8080 --name mtpscript-daemon mtpscript-app
```

### AWS Lambda Custom Runtime

For serverless daemon-like execution, use the Lambda deployment:

```bash
# Create Lambda deployment package
./mtpsc lambda-deploy app.mtps

# The resulting ZIP contains:
# - bootstrap (Lambda entry point)
# - mtpjs (runtime binary)
# - app.msqs (compiled snapshot)
```

## How to Call and See Commandline Commands

### MTPScript Compiler (`mtpsc`)

The main compiler tool with comprehensive command-line interface:

```bash
# Show help
./mtpsc --help

# Usage syntax
Usage: mtpsc <command> [options] <file>

Commands:
  compile <file>        Compile MTPScript to JavaScript intermediate representation
  run <file>           Compile and run MTPScript (combines compile + execute)
  check <file>         Type check MTPScript code without compilation
  openapi <file>       Generate OpenAPI 3.0 specification from type annotations
  snapshot <file>      Create signed .msqs snapshot for production deployment
  lambda-deploy <file> Create AWS Lambda deployment package
  infra-generate       Generate AWS infrastructure templates
  serve <file>         Start local web server daemon
  lsp                  Start Language Server Protocol server
  npm-audit <dir>      Generate audit manifest for unsafe adapters

Migration Commands:
  migrate <file.ts>       Convert TypeScript to MTPScript
  migrate --dir <dir>     Batch migration of directories
  migrate --check         Dry-run with compatibility report

Package Manager:
  add <package>[@ver]     Add git-pinned dependency
  remove <package>        Remove dependency
  update <package>        Update to latest signed tag
  list                    List all dependencies

Performance & Analysis:
  benchmark <file> [n]    Run performance benchmark (default 100 iterations)
  profile <file>          Profile gas consumption
```

### Runtime Engine (`mtpjs`)

High-performance execution engine:

```bash
# Show help
./mtpjs --help

Usage: mtpjs [options] [file [args]]

Core execution:
  -b, --allow-bytecode    Load and execute .msqs snapshot or bytecode file
  [file]                  Execute compiled JavaScript intermediate representation

Development tools:
  -h, --help              Show help
  -e, --eval EXPR         Evaluate JavaScript expression
  -i, --interactive       Start interactive REPL
  -I, --include file      Include additional JavaScript file

Resource controls:
  -d, --dump              Show memory usage statistics after execution
      --memory-limit n    Limit memory usage to 'n' bytes (default: 16MB)
  --no-column             Strip column info from debug output

Bytecode operations:
  -o FILE                 Compile JavaScript to bytecode file
  -m32                    Generate 32-bit compatible bytecode
```

### Example Command Usage

```bash
# Development workflow
./mtpsc check app.mtps                    # Type check only
./mtpsc compile app.mtps > app.js         # Compile to JS
./mtpjs app.js                            # Execute JS
./mtpsc run app.mtps                       # Compile and run in one step

# Production deployment
./mtpsc snapshot app.mtps                  # Create signed snapshot
./mtpjs -b app.msqs                        # Execute snapshot

# API documentation
./mtpsc openapi app.mtps > api.json        # Generate OpenAPI spec

# Performance analysis
./mtpsc benchmark app.mtps 1000            # Run 1000 iterations
./mtpsc profile app.mtps                    # Profile gas usage

# Package management
./mtpsc add lodash@4.17.21                 # Add dependency
./mtpsc list                               # Show dependencies
./mtpsc update lodash                      # Update dependency
```

## How to Deploy Serverless

### AWS Lambda

#### 1. Create Lambda Function

```bash
# Compile and create deployment package
./mtpsc lambda-deploy app.mtps

# This creates app-lambda.zip containing:
# - bootstrap (custom runtime bootstrap script)
# - mtpjs (runtime binary)
# - app.msqs (compiled snapshot)
```

#### 2. AWS CLI Deployment

```bash
# Create IAM role (if not exists)
aws iam create-role --role-name mtpscript-lambda-role \
  --assume-role-policy-document file://trust-policy.json

# Create Lambda function
aws lambda create-function --function-name my-mtpscript-app \
  --runtime provided.al2 \
  --role arn:aws:iam::account-id:role/mtpscript-lambda-role \
  --handler bootstrap \
  --zip-file fileb://app-lambda.zip \
  --architectures x86_64

# Update function code
aws lambda update-function-code --function-name my-mtpscript-app \
  --zip-file fileb://app-lambda.zip
```

#### 3. Infrastructure as Code

Generate Terraform configuration:
```bash
./mtpsc infra-generate terraform
```

This creates `infrastructure.tf`:

```hcl
resource "aws_lambda_function" "mtpscript_app" {
  function_name = "mtpscript-app"
  runtime       = "provided.al2"
  handler       = "bootstrap"
  architectures = ["x86_64"]

  filename         = "app-lambda.zip"
  source_code_hash = filebase64sha256("app-lambda.zip")

  environment {
    variables = {
      NODE_ENV = "production"
    }
  }
}

resource "aws_apigatewayv2_api" "mtpscript_api" {
  name          = "mtpscript-api"
  protocol_type = "HTTP"
}

resource "aws_apigatewayv2_integration" "mtpscript_integration" {
  api_id           = aws_apigatewayv2_api.mtpscript_api.id
  integration_type = "AWS_PROXY"

  connection_type    = "INTERNET"
  integration_method = "POST"
  integration_uri    = aws_lambda_function.mtpscript_app.invoke_arn
}
```

### Google Cloud Functions

```bash
# Generate GCP deployment
./mtpsc infra-generate gcp

# Deploy using gcloud CLI
gcloud functions deploy my-mtpscript-app \
  --runtime custom \
  --source . \
  --entry-point bootstrap \
  --trigger-http \
  --allow-unauthenticated
```

### Azure Functions

```bash
# Generate ARM templates
./mtpsc infra-generate azure

# Deploy using Azure CLI
az functionapp create --resource-group myResourceGroup \
  --consumption-plan-location eastus \
  --runtime custom \
  --name my-mtpscript-app \
  --storage-account myStorageAccount
```

### Multi-Cloud Deployment

For multi-cloud deployment, use the generated infrastructure templates:

```bash
# Generate all infrastructure templates
./mtpsc infra-generate all

# This creates:
# - terraform/ (Terraform modules for AWS, GCP, Azure)
# - arm-templates/ (Azure Resource Manager templates)
# - deployment-manager/ (GCP Deployment Manager templates)
```

## Language Keywords and Syntax

MTPScript is a functional programming language with effect tracking. Here are all language keywords and their usage:

### Core Keywords

#### Function Definition
```mtp
function functionName(param: Type): ReturnType {
    // function body
    return value;
}

// Example
function add(x: number, y: number): number {
    return x + y;
}
```

#### Variable Declaration
```mtp
const variableName = value;
const typedVar: string = "hello";
```

#### Control Flow
```mtp
// If expression (returns a value)
const result = if (condition) {
    "true branch"
} else {
    "false branch"
};

// Await for async operations
const data = await fetchData();
```

#### API Endpoints
```mtp
api GET "/users" function getUsers(): Json {
    return { users: [] };
}

api POST "/users" function createUser(name: string): Json {
    return { id: 123, name: name };
}
```

#### Effects (Side Effect Tracking)
```mtp
effect DbRead;
effect DbWrite;
effect HttpOut;
effect Log;

// Usage in functions
function saveUser(user: User): Result<string, string> {
    const result = DbWrite.execute("INSERT INTO users...", [user]);
    return Ok("User saved");
}
```

#### Pattern Matching
```mtp
match result {
    Ok(data) => processData(data),
    Err(error) => handleError(error)
}

match value {
    Some(x) => useValue(x),
    None => defaultValue()
}
```

#### Module System
```mtp
import { func1, func2 } from "package@version";
import * as utils from "./utils.mtps";

uses "database@1.2.3";
uses "http@2.0.0";
```

### HTTP Methods (for API definitions)
```mtp
api GET "/path" function handler(): Json { /* ... */ }
api POST "/path" function handler(): Json { /* ... */ }
api PUT "/path" function handler(): Json { /* ... */ }
api DELETE "/path" function handler(): Json { /* ... */ }
api PATCH "/path" function handler(): Json { /* ... */ }
```

### Type Annotations
```mtp
// Primitive types
const num: number = 42;
const decimal: Decimal = 3.14159;
const text: string = "hello";
const flag: boolean = true;

// Complex types
const user: { name: string, age: number } = { name: "John", age: 30 };
const users: [User] = [user1, user2];
const result: Result<string, string> = Ok("success");
const maybeUser: Option<User> = Some(user);
```

### Pipeline Operator
```mtp
// Left-associative pipeline (like F#)
let result = data
    |> validate
    |> transform
    |> save;
```

### Comments
```mtp
// Single line comment

/*
Multi-line
comment
*/
```

## JavaScript Syntax Comparison

MTPScript is designed to be familiar to JavaScript developers while being more restrictive and deterministic. Here's a side-by-side comparison:

### Functions

**JavaScript:**
```javascript
function add(x, y) {
  return x + y;
}

const multiply = (x, y) => x * y;
```

**MTPScript:**
```
function add(x: number, y: number): number {
    return x + y;
}

function multiply(x: number, y: number): number {
    return x * y;
}
```

### Variables and Types

**JavaScript:**
```javascript
let x = 42;
const name = "John";
var flag = true;
```

**MTPScript:**
```
const x = 42;  // Inferred as number
const name: string = "John";
const flag: boolean = true;
```

### Objects and Data Structures

**JavaScript:**
```javascript
const user = {
  name: "John",
  age: 30
};

const users = [
  user1,
  user2
];
```

**MTPScript:**
```
const user = {
    name: "John",
    age: 30
};

const users: [{ name: string, age: number }] = [
    user1,
    user2
];
```

### Control Flow

**JavaScript:**
```javascript
if (condition) {
  doSomething();
} else {
  doOther();
}

for (let i = 0; i < 10; i++) {
  console.log(i);
}
```

**MTPScript:**
```
const result = if (condition) {
    doSomething()
} else {
    doOther()
};

// No loops - use recursion or higher-order functions
```

### Error Handling

**JavaScript:**
```javascript
try {
  riskyOperation();
} catch (error) {
  console.error(error);
}

throw new Error("Failed");
```

**MTPScript:**
```
const result = match riskyOperation() {
    Ok(data) => process(data),
    Err(error) => handleError(error)
};

// Errors are values, not exceptions
return Err("Failed");
```

### Async/Await

**JavaScript:**
```javascript
async function fetchData() {
  const response = await fetch(url);
  return response.json();
}
```

**MTPScript:**
```
function fetchData(): Result<Json, string> {
    const response = await HttpOut.get(url);
    return response.body;
}
```

### Classes and OOP

**JavaScript:**
```javascript
class User {
  constructor(name) {
    this.name = name;
  }

  greet() {
    return `Hello, ${this.name}`;
  }
}
```

**MTPScript:**
```
// No classes - use functions and data structures
function createUser(name: string): User {
    return { name: name };
}

function greet(user: User): string {
    return "Hello, " + user.name;
}
```

### Modules

**JavaScript:**
```javascript
// ES6 modules
import { function } from 'module';
export const value = 42;

// CommonJS
const mod = require('module');
```

**MTPScript:**
```
// Git-hash pinned imports
import { function } from "module@1.2.3";
uses "module@1.2.3";

// No runtime require()
```

### Prohibited JavaScript Features

MTPScript **explicitly forbids** these JavaScript features to ensure determinism:

- `eval()` - Dynamic code execution
- `class` - Object-oriented programming
- `this` - Instance methods and state
- Loops (`for`, `while`, `do-while`) - Use recursion or map/filter
- `==` - Only strict equality (`===` equivalent)
- `var` - Only block-scoped declarations
- `new` - No constructors
- `prototype` - No prototype manipulation
- `async`/`await` - Only explicit effect tracking
- `try`/`catch`/`throw` - Only Result/Option types

## Folder Structure and Where Things Are

```
mtpscript/
├── build/                          # Build artifacts and generated files
│   ├── artifacts/                  # Final build outputs
│   ├── ci/                        # CI/CD scripts
│   ├── docker/                    # Docker configurations
│   ├── generated/                 # Auto-generated headers and files
│   │   ├── example_stdlib.h
│   │   └── mquickjs_atom.h
│   └── objects/                   # Compiled object files (.o)
├── core/                          # Core runtime engine
│   ├── crypto/                    # Cryptographic operations
│   │   ├── mquickjs_crypto.c/.h
│   ├── db/                        # Database integration
│   │   ├── mquickjs_db.c/.h
│   ├── effects/                   # Side effect tracking
│   │   ├── mquickjs_effects.c/.h
│   │   └── mquickjs_log.c/.h
│   ├── http/                      # HTTP client/server
│   │   ├── mquickjs_http.c/.h
│   └── runtime/                   # Core VM and execution
│       ├── mquickjs.c/.h          # Main QuickJS runtime
│       ├── mquickjs_errors.c/.h   # Error handling
│       ├── mtpjs_stdlib.c/.h      # Standard library
│       └── readline*.c/.h         # REPL and CLI
├── src/                           # Source code organization
│   ├── cli/                       # Command-line tools
│   │   └── mtpsc.c                # Main compiler CLI
│   ├── compiler/                  # Compiler pipeline
│   │   ├── lexer.c/.h            # Lexical analysis
│   │   ├── parser.c/.h           # Syntax parsing
│   │   ├── typechecker.c/.h      # Type checking
│   │   ├── codegen.c/.h          # Code generation
│   │   ├── bytecode.c/.h         # Bytecode emission
│   │   └── *.c/.h                # Other compiler modules
│   ├── main/                     # Main entry points
│   │   ├── mtpjs.c               # Runtime main
│   │   └── readline.c            # REPL main
│   ├── snapshot/                 # Snapshot creation
│   ├── stdlib/                   # Standard library
│   └── test/                     # Unit tests
├── tools/                        # Development tools
│   ├── bench/                    # Benchmarking tools
│   ├── build_info_generator.c    # Build metadata generator
│   └── example_stdlib            # Example stdlib builder
├── pkg/                          # Package management
│   ├── decimal/                  # Decimal arithmetic
│   ├── lsp/                      # Language server
│   └── readline/                 # Enhanced readline
├── extensions/                   # Editor integrations
│   ├── cursor/                   # Cursor editor support
│   └── vscode/                   # VS Code extension
├── tests/                        # Test suites
│   ├── fixtures/                 # Test MTPScript files
│   ├── integration/              # Integration tests
│   └── unit/                     # Unit tests
├── examples/                     # Example applications
│   ├── example.c                 # C API example
│   └── example_stdlib.c          # Stdlib example
├── compliance/                   # Compliance documentation
│   ├── iso27001-compliance.md
│   ├── pci-dss-compliance.md
│   ├── soc2-compliance.md
│   └── sox-compliance.md
├── docs/                         # Documentation
│   ├── api/                      # API documentation
│   └── requirements/             # Requirements and specs
├── marketing/                    # Marketing materials
├── vendor/                       # Third-party dependencies
├── Dockerfile                    # Container build
├── Makefile                      # Build system
├── mtp.lock                      # Dependency lockfile
├── README.md                     # This file
├── LICENSE                       # License
├── openapi-rules-v5.1.json       # OpenAPI validation rules
├── gas_costs.h                   # Gas metering costs
└── gas-v5.1.csv                  # Gas cost spreadsheet
```

### Key Directories Explained

#### `core/` - Runtime Engine
Contains the core QuickJS-based runtime that executes MTPScript code. This is where memory management, garbage collection, and bytecode execution happen.

#### `src/compiler/` - Compilation Pipeline
The complete compiler toolchain that transforms MTPScript source code into deterministic JavaScript, then bytecode, then signed snapshots.

#### `src/cli/` - Command Line Tools
Contains the main `mtpsc` compiler interface with all commands for compilation, deployment, package management, and development tools.

#### `build/` - Build System
Contains all build artifacts, generated files, and cross-compilation outputs. The Makefile coordinates building across this directory structure.

#### `tests/` - Quality Assurance
Comprehensive test suite including unit tests, integration tests, and performance benchmarks.

#### `examples/` - Getting Started
Simple examples showing how to use the MTPScript runtime from C code and how to build custom standard libraries.

#### `extensions/` - Developer Experience
Editor integrations for syntax highlighting, language server protocol support, and IDE features.

## License

MIT License - see LICENSE file for details.


