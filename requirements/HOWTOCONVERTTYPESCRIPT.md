# **Converting TypeScript to MTPScript: A Comprehensive Guide**

## 1. Understanding the Differences

### 1.1 Language Features

MTPScript is designed to be safer, simpler, and more deterministic than TypeScript. Here are some key differences:

- **No Classes or Inheritance**: MTPScript uses records and functions instead of classes.
- **No Dynamic Code Loading**: MTPScript does not support dynamic code loading.
- **No Shared Mutable State**: MTPScript ensures immutability.
- **No Floating-Point Math**: MTPScript uses deterministic arithmetic for financial-grade calculations.
- **Explicit Effects**: MTPScript requires explicit declaration of effects (capabilities).

### 1.2 Type System

MTPScript has a strong type system with primitive and composite types. It does not support `null` or `undefined`, using `Option` and `Result` types instead.

### 1.3 Syntax

MTPScript has a simpler syntax compared to TypeScript. For example, it uses explicit `let` statements and does not support implicit returns.

---

## 2. Manual Conversion Steps

### 2.1 Convert TypeScript Types to MTPScript Types

MTPScript uses the same type names as TypeScript for primitives (except for high-precision decimals).

#### TypeScript

```typescript
interface User {
  id: number;
  name: string;
  email: string;
}
```

#### MTPScript

```mtp
type User {
  id: number
  name: string
  email: string
}
```

### 2.2 Convert Classes to Records and Functions

#### TypeScript

```typescript
class User {
  constructor(public id: number, public name: string, public email: string) {}
}
```

#### MTPScript

```mtp
type User {
  id: number
  name: string
  email: string
}

function createUser(id: number, name: string, email: string): User {
  return { id, name, email }
}
```

### 2.3 Convert Exceptions to Result Types

#### TypeScript

```typescript
function divide(a: number, b: number): number {
  if (b === 0) {
    throw new Error("Division by zero");
  }
  return a / b;
}
```

#### MTPScript

```mtp
type Result<T, E> {
  | ok(value: T)
  | err(error: E)
}

function divide(a: number, b: number): Result<number, string> {
  if (b == 0) {
    return err("Division by zero")
  }
  return ok(a / b)
}
```

### 2.4 Convert Async/Await to Effects

#### TypeScript

```typescript
async function fetchUser(id: number): Promise<User> {
  const response = await fetch(`https://api.example.com/users/${id}`);
  return await response.json();
}
```

#### MTPScript

```mtp
effect http

function fetchUser(id: number): User uses { http } {
  const response: string = http.get(`https://api.example.com/users/${id}`)
  return json.parse(response)
}
```

### 2.5 Convert Interfaces to Type Aliases

#### TypeScript

```typescript
interface Payment {
  amount: number;
  currency: string;
}
```

#### MTPScript

```mtp
type Payment {
  amount: number
  currency: string
}
```

---

## 3. Writing a Converter Tool

### 3.1 Setting Up the Converter

You can write a converter tool in TypeScript or any other language you are comfortable with. Below is an example using TypeScript.

### 3.2 Parsing TypeScript Code

Use a TypeScript parser like `typescript-eslint-parser` to parse the TypeScript code.

```typescript
import * as ts from 'typescript';
import * as fs from 'fs';

function parseTsFile(filePath: string): ts.SourceFile {
  const content = fs.readFileSync(filePath, 'utf-8');
  return ts.createSourceFile(filePath, content, ts.ScriptTarget.Latest, true);
}
```

### 3.3 Converting TypeScript to MTPScript

Create functions to convert different TypeScript constructs to MTPScript.

#### Convert Interfaces

```typescript
function convertInterface(node: ts.InterfaceDeclaration): string {
  const properties = node.members.map((member) => {
    if (ts.isPropertySignature(member)) {
      return `${member.name.getText()}: ${convertType(member.type)}`;
    }
    return '';
  }).join('\n');

  return `type ${node.name.getText()} {\n${properties}\n}`;
}
```

#### Convert Types

```typescript
function convertType(node: ts.TypeNode): string {
  if (ts.isKeywordTypeNode(node)) {
    switch (node.kind) {
      case ts.SyntaxKind.NumberKeyword:
        return 'number';
      case ts.SyntaxKind.StringKeyword:
        return 'string';
      case ts.SyntaxKind.BooleanKeyword:
        return 'boolean';
      default:
        return 'Any';
    }
  }
  return node.getText();
}
```

#### Convert Functions

```typescript
function convertFunction(node: ts.FunctionDeclaration): string {
  const params = node.parameters.map((param) => {
    return `${param.name.getText()}: ${convertType(param.type)}`;
  }).join(', ');

  const returnType = node.type ? convertType(node.type) : 'Any';

  const body = node.body ? node.body.getText() : '';

  return `function ${node.name.getText()}(${params}): ${returnType} {\n${body}\n}`;
}
```

### 3.4 Writing the Converted Code

Write the converted MTPScript code to a file.

```typescript
function writeMtpFile(filePath: string, content: string): void {
  fs.writeFileSync(filePath, content, 'utf-8');
}
```

### 3.5 Putting It All Together

Combine the parsing and conversion functions to create the converter tool.

```typescript
function convertTsToMtp(inputFile: string, outputFile: string): void {
  const sourceFile = parseTsFile(inputFile);
  let mtpContent = '';

  ts.forEachChild(sourceFile, (node) => {
    if (ts.isInterfaceDeclaration(node)) {
      mtpContent += convertInterface(node) + '\n';
    } else if (ts.isFunctionDeclaration(node)) {
      mtpContent += convertFunction(node) + '\n';
    }
  });

  writeMtpFile(outputFile, mtpContent);
}

// Example usage
convertTsToMtp('input.ts', 'output.mtp');
```

---

## 4. Example Conversion

### 4.1 TypeScript Code

```typescript
// input.ts
interface User {
  id: number;
  name: string;
  email: string;
}

function createUser(id: number, name: string, email: string): User {
  return { id, name, email };
}

async function fetchUser(id: number): Promise<User> {
  const response = await fetch(`https://api.example.com/users/${id}`);
  return await response.json();
}
```

### 4.2 Converted MTPScript Code

```mtp
// output.mtp
type User {
  id: number
  name: string
  email: string
}

function createUser(id: number, name: string, email: string): User {
  return { id, name, email }
}

effect http

function fetchUser(id: number): User uses { http } {
  const response: string = http.get(`https://api.example.com/users/${id}`)
  return json.parse(response)
}
```

---

## 5. Conclusion

This guide provides a comprehensive overview of how to convert TypeScript to MTPScript and how to write a converter tool. By understanding the differences between the two languages and using the provided conversion functions, you can automate the process of converting TypeScript code to MTPScript. This ensures that your existing TypeScript APIs can be migrated to MTPScript with minimal effort.