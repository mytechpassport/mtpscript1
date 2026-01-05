# **Building the MTPScript Package Manager**

## 1. Package Format

### 1.1 Package Definition

Each package is defined by a `mtp.toml` file, which specifies the package's metadata and dependencies.

```toml
[package]
name = "example-package"
version = "0.1.0"
description = "An example MTPScript package"

[dependencies]
another-package = "github.com/another/another-package@abc123"
```

### 1.2 Package Structure

A typical package has the following structure:

```
example-package/
├── src/
│   └── main.mtp
├── host/
│   └── unsafe/
│       └── example.js
├── mtp.toml
└── package.json
```

## 2. Package Manager Commands

### 2.1 Initialize a New Project

```sh
mtp init
```

This command creates a new project directory with a basic structure and a `mtp.toml` file.

### 2.2 Add a Dependency

```sh
mtp add github.com/example/lib@abc123
```

This command adds a dependency to the `mtp.toml` file and vendors it at build time.

### 2.3 Build the Project

```sh
mtp build
```

This command compiles the project and vendors all dependencies.

### 2.4 Serve the Project

```sh
mtp serve --port 8080
```

This command starts the reference HTTP server for local development.

### 2.5 Deploy to Serverless

```sh
mtp deploy --target lambda
```

This command deploys the project to AWS Lambda.

## 3. Package Manager Implementation

### 3.1 Parsing `mtp.toml`

The package manager needs to parse the `mtp.toml` file to understand the project's dependencies.

```sh
# Example of parsing mtp.toml
cat mtp.toml | toml-cli parse
```

### 3.2 Fetching Dependencies

Dependencies are fetched from Git repositories and vendored at build time.

```sh
# Example of fetching a dependency
git clone https://github.com/example/lib.git
cd lib
git checkout abc123
```

### 3.3 Vendoring Dependencies

Dependencies are copied into the project's `vendor/` directory.

```sh
# Example of vendoring a dependency
cp -r lib vendor/example-lib
```

### 3.4 Building the Project

The project is compiled using the MTPScript compiler.

```sh
# Example of building the project
mtpc -o app.bin src/main.mtp
```

### 3.5 Running the HTTP Server

The reference HTTP server is started using the compiled bytecode.

```sh
# Example of running the HTTP server
mtp-runtime serve app.bin --port 8080
```

### 3.6 Deploying to Serverless

The project is deployed to AWS Lambda using a custom runtime.

```sh
# Example of deploying to AWS Lambda
aws lambda create-function --function-name my-mtpscript-function --runtime provided.al2 --handler index.handler --zip-file fileb://app.zip
```

## 4. npm Bridge

### 4.1 Creating Host Adapters

Host adapters are created for npm packages to bridge them into MTPScript.

```js
// host/unsafe/example.js
import { exampleFunction } from "example-npm-package";

export function callExampleFunction() {
  return exampleFunction();
}
```

### 4.2 Injecting Adapters

Adapters are injected as effects at runtime.

```sh
# Example of injecting an adapter
effects.unsafe = {
  callExampleFunction: require('./host/unsafe/example').callExampleFunction
}
```

## 5. Folder Structure

A typical project has the following folder structure:

```
my-mtpscript-project/
├── src/
│   └── main.mtp
├── host/
│   └── unsafe/
│       └── example.js
├── vendor/
│   └── example-lib/
├── mtp.toml
└── package.json
```

## 6. Example `mtp.toml`

```toml
[package]
name = "my-mtpscript-project"
version = "0.1.0"
description = "A sample MTPScript project"

[dependencies]
example-lib = "github.com/example/lib@abc123"
```

## 7. Example `package.json`

```json
{
  "name": "my-mtpscript-project",
  "version": "0.1.0",
  "dependencies": {
    "example-npm-package": "^1.0.0"
  }
}
```

## 8. Example `main.mtp`

```mtp
// Define an HTTP API
api GET /hello {
  respond "Hello, World!"
}

// Define a function that uses an npm package
function useExampleFunction(): string uses { unsafe } {
  return unsafe.callExampleFunction()
}
```

## 9. Example `host/unsafe/example.js`

```js
// host/unsafe/example.js
import { exampleFunction } from "example-npm-package";

export function callExampleFunction() {
  return exampleFunction();
}
```

## 10. Building the Package Manager

### 10.1 Parsing `mtp.toml`

```sh
# Example of parsing mtp.toml
cat mtp.toml | toml-cli parse
```

### 10.2 Fetching Dependencies

```sh
# Example of fetching a dependency
git clone https://github.com/example/lib.git
cd lib
git checkout abc123
```

### 10.3 Vendoring Dependencies

```sh
# Example of vendoring a dependency
cp -r lib vendor/example-lib
```

### 10.4 Building the Project

```sh
# Example of building the project
mtpc -o app.bin src/main.mtp
```

### 10.5 Running the HTTP Server

```sh
# Example of running the HTTP server
mtp-runtime serve app.bin --port 8080
```

### 10.6 Deploying to Serverless

```sh
# Example of deploying to AWS Lambda
aws lambda create-function --function-name my-mtpscript-function --runtime provided.al2 --handler index.handler --zip-file fileb://app.zip
```