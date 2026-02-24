/**
 * MTPScript Code Generator
 * Specification §5.1, §5.3
 *
 * Converts MTPScript AST to JavaScript
 *
 * Copyright (c) 2025 My Tech Passport Inc.
 * Author: Ryan Wong
 */

import {
  Program,
  Declaration,
  FunctionDeclaration,
  ApiDeclaration,
  Statement,
  Expression,
  Type,
  Parameter,
  MatchExpression,
  MatchArm,
  Pattern,
  IfExpression,
  BlockExpression,
  LambdaExpression,
  PipeExpression,
  VariableDeclaration,
  ExpressionStatement,
  UsesBlock,
  OptionLiteral,
  ResultLiteral,
  ListLiteral,
  MapLiteral,
  RecordLiteral,
  FunctionCall,
  MethodCall,
  Variable
} from './ast.js';

export class CodeGenerator {
  private indentLevel = 0;
  private output: string[] = [];

  private indent(): string {
    return '  '.repeat(this.indentLevel);
  }

  private emit(line: string = ''): void {
    this.output.push(this.indent() + line);
  }

  private emitBlock(start: string, end: string, body: () => void): void {
    this.emit(start);
    this.indentLevel++;
    body();
    this.indentLevel--;
    this.emit(end);
  }

  public generateProgram(program: Program): string {
    this.output = [];
    this.indentLevel = 0;

    // Add standard library imports
    this.emit('"use strict";');
    this.emit();

    // Generate module wrapper
    this.emitBlock('(function() {', '})();', () => {
      // Generate declarations
      for (const decl of program.declarations) {
        this.generateDeclaration(decl);
      }

      // Export main function if it exists
      this.emit();
      this.emit('if (typeof exports !== "undefined") {');
      this.emit('  exports.handler = main;');
      this.emit('}');
    });

    return this.output.join('\n');
  }

  private generateDeclaration(decl: Declaration): void {
    switch (decl.kind) {
      case 'FunctionDeclaration':
        this.generateFunctionDeclaration(decl as FunctionDeclaration);
        break;
      case 'ApiDeclaration':
        this.generateApiDeclaration(decl as ApiDeclaration);
        break;
      default:
        // Handle other declaration types
        this.emit(`// TODO: ${decl.kind}`);
    }
  }

  private generateFunctionDeclaration(decl: FunctionDeclaration): void {
    const params = decl.parameters.map(p => p.name).join(', ');
    const effects = decl.effects.map(e => e.kind.replace('Effect', '')).join(', ');

    this.emit();
    this.emit(`// ${decl.name}: ${effects ? `uses { ${effects} }` : 'pure'}`);
    this.emit(`function ${decl.name}(${params}) {`);

    this.indentLevel++;
    this.generateBlockExpression(decl.body);
    this.indentLevel--;

    this.emit('}');
  }

  private generateApiDeclaration(decl: ApiDeclaration): void {
    const pathParams = decl.pathParameters.join(', ');
    const queryParams = decl.queryParameters.map(p => p.name).join(', ');
    const allParams = [pathParams, queryParams].filter(p => p).join(', ');
    const effects = decl.effects.map(e => e.kind.replace('Effect', '')).join(', ');

    this.emit();
    this.emit(`// API ${decl.method} ${decl.path}: uses { ${effects} }`);
    this.emit(`function api_${decl.method.toLowerCase()}_${decl.path.replace(/[^a-zA-Z0-9]/g, '_')}(${allParams}) {`);

    this.indentLevel++;
    this.generateBlockExpression(decl.body);
    this.indentLevel--;

    this.emit('}');
  }

  private generateStatement(stmt: Statement): void {
    switch (stmt.kind) {
      case 'VariableDeclaration':
        this.generateVariableDeclaration(stmt as VariableDeclaration);
        break;
      case 'ExpressionStatement':
        this.generateExpressionStatement(stmt as ExpressionStatement);
        break;
      case 'ReturnStatement':
        this.generateReturnStatement(stmt as any);
        break;
      case 'UsesBlock':
        this.generateUsesBlock(stmt as UsesBlock);
        break;
      default:
        this.emit(`// TODO: ${(stmt as any).kind}`);
    }
  }

  private generateVariableDeclaration(decl: VariableDeclaration): void {
    const keyword = decl.isConst ? 'const' : 'let';
    const typeAnnotation = decl.type ? ` /*: ${this.typeToString(decl.type)} */` : '';
    this.emit(`${keyword} ${decl.name}${typeAnnotation} = ${this.generateExpression(decl.initializer)};`);
  }

  private generateExpressionStatement(stmt: ExpressionStatement): void {
    this.emit(`${this.generateExpression(stmt.expression)};`);
  }

  private generateReturnStatement(stmt: any): void {
    if (stmt.expression) {
      this.emit(`return ${this.generateExpression(stmt.expression)};`);
    } else {
      this.emit('return;');
    }
  }

  private generateUsesBlock(block: UsesBlock): void {
    const effects = block.effects.map(e => e.kind.replace('Effect', '')).join(', ');
    this.emit(`// uses { ${effects} }`);
    this.emit('{');

    this.indentLevel++;
    this.generateBlockExpression(block.body);
    this.indentLevel--;

    this.emit('}');
  }

  private generateExpression(expr: Expression): string {
    switch (expr.kind) {
      case 'IntLiteral':
        return (expr as any).value.toString();

      case 'StringLiteral':
        return JSON.stringify((expr as any).value);

      case 'BoolLiteral':
        return (expr as any).value ? 'true' : 'false';

      case 'DecimalLiteral':
        return `Decimal.from(${JSON.stringify((expr as any).value)}).value`;

      case 'Variable':
        return (expr as any).name;

      case 'OptionLiteral':
        const opt = expr as OptionLiteral;
        if (opt.tag === 'Some') {
          return `{tag: "Some", value: ${this.generateExpression(opt.value!)}}`;
        } else {
          return `{tag: "None"}`;
        }

      case 'ResultLiteral':
        const res = expr as ResultLiteral;
        if (res.tag === 'Ok') {
          return `{tag: "Ok", value: ${this.generateExpression(res.value)}}`;
        } else {
          return `{tag: "Err", error: ${this.generateExpression(res.value)}}`;
        }

      case 'ListLiteral':
        const list = expr as ListLiteral;
        const elements = list.elements.map(e => this.generateExpression(e)).join(', ');
        return `[${elements}]`;

      case 'MapLiteral':
        const map = expr as MapLiteral;
        const entries = map.entries.map(([k, v]) =>
          `[${this.generateExpression(k)}, ${this.generateExpression(v)}]`
        ).join(', ');
        return `new Map([${entries}])`;

      case 'RecordLiteral':
        const record = expr as RecordLiteral;
        const fields = Object.entries(record.fields).map(([k, v]) =>
          `${k}: ${this.generateExpression(v)}`
        ).join(', ');
        return `{${fields}}`;

      case 'BinaryExpression':
        const binary = expr as any;
        const left = this.generateExpression(binary.left);
        const right = this.generateExpression(binary.right);
        return `${left} ${this.operatorToString(binary.operator)} ${right}`;

      case 'FunctionCall':
        const call = expr as FunctionCall;
        const args = call.arguments.map(a => this.generateExpression(a)).join(', ');
        return `${this.generateExpression(call.function)}(${args})`;

      case 'MethodCall':
        const method = expr as MethodCall;
        const methodArgs = method.arguments.map(a => this.generateExpression(a)).join(', ');
        return `${this.generateExpression(method.object)}.${method.method}(${methodArgs})`;

      case 'IfExpression':
        const ifExpr = expr as IfExpression;
        const condition = this.generateExpression(ifExpr.condition);
        const thenBranch = this.generateExpression(ifExpr.thenBranch);
        const elseBranch = this.generateExpression(ifExpr.elseBranch);
        return `(${condition} ? ${thenBranch} : ${elseBranch})`;

      case 'MatchExpression':
        return this.generateMatchExpression(expr as MatchExpression);

      case 'BlockExpression':
        return this.generateInlineBlockExpression(expr as BlockExpression);

      case 'LambdaExpression':
        const lambda = expr as LambdaExpression;
        const lambdaParams = lambda.parameters.map(p => p.name).join(', ');
        const lambdaBody = this.generateExpression(lambda.body);
        return `(${lambdaParams}) => ${lambdaBody}`;

      case 'PipeExpression':
        const pipe = expr as PipeExpression;
        // Left-associative: a |> b |> c ≡ (a |> b) |> c
        return this.generatePipeExpression(pipe);

      default:
        return `/* TODO: ${expr.kind} */`;
    }
  }

  private generateMatchExpression(match: MatchExpression): string {
    const expr = this.generateExpression(match.expression);

    // Convert to chained ternary expressions
    let result = '';

    for (let i = 0; i < match.arms.length; i++) {
      const arm = match.arms[i];
      const condition = this.generatePatternMatch(expr, arm.pattern);
      const body = this.generateExpression(arm.body);

      if (i === 0) {
        result = `(${condition} ? ${body} : `;
      } else if (i === match.arms.length - 1) {
        result += `${body}`;
      } else {
        result += `(${condition} ? ${body} : `;
      }
    }

    // Close all the ternaries
    for (let i = 1; i < match.arms.length; i++) {
      result += ')';
    }

    return result;
  }

  private generatePatternMatch(expr: string, pattern: Pattern): string {
    // Simplified pattern matching - would need expansion for full destructuring
    switch (pattern.kind) {
      case 'WildcardPattern':
        return 'true';

      case 'VariablePattern':
        return 'true'; // Variable binding handled separately

      case 'LiteralPattern':
        const literal = (pattern as any).value;
        return `${expr} === ${this.generateExpression(literal)}`;

      case 'OptionPattern':
        const optPattern = pattern as any;
        if (optPattern.tag === 'Some') {
          return `${expr}.tag === "Some"`;
        } else {
          return `${expr}.tag === "None"`;
        }

      case 'ResultPattern':
        const resPattern = pattern as any;
        if (resPattern.tag === 'Ok') {
          return `${expr}.tag === "Ok"`;
        } else {
          return `${expr}.tag === "Err"`;
        }

      default:
        return 'true'; // TODO: Implement full pattern matching
    }
  }

  private generateInlineBlockExpression(block: BlockExpression): string {
    // For inline blocks, use IIFE
    let result = '(() => { ';

    for (const stmt of block.statements) {
      if (stmt.kind === 'VariableDeclaration') {
        const decl = stmt as VariableDeclaration;
        result += `const ${decl.name} = ${this.generateExpression(decl.initializer)}; `;
      } else if (stmt.kind === 'ExpressionStatement') {
        const exprStmt = stmt as ExpressionStatement;
        result += `${this.generateExpression(exprStmt.expression)}; `;
      }
    }

    if (block.result) {
      result += `return ${this.generateExpression(block.result)}; `;
    }

    result += '})()';
    return result;
  }

  private generateBlockExpression(block: BlockExpression): void {
    for (const stmt of block.statements) {
      this.generateStatement(stmt);
    }

    if (block.result) {
      this.emit(`return ${this.generateExpression(block.result)};`);
    }
  }

  private generatePipeExpression(pipe: PipeExpression): string {
    // Convert a |> f to f(a)
    // This assumes f is a function that takes one argument
    const left = this.generateExpression(pipe.left);
    const right = this.generateExpression(pipe.right);

    // For simple cases, assume right is a function call
    return `${right}(${left})`;
  }

  private operatorToString(operator: string): string {
    switch (operator) {
      case 'PLUS': return '+';
      case 'MINUS': return '-';
      case 'STAR': return '*';
      case 'SLASH': return '/';
      case 'PERCENT': return '%';
      case 'EQUALS': return '===';
      case 'NOT_EQUALS': return '!==';
      case 'LT': return '<';
      case 'LE': return '<=';
      case 'GT': return '>';
      case 'GE': return '>=';
      case 'AND': return '&&';
      case 'OR': return '||';
      default: return operator;
    }
  }

  private typeToString(type: Type): string {
    switch (type.kind) {
      case 'IntType': return 'Int';
      case 'StringType': return 'String';
      case 'BoolType': return 'Bool';
      case 'DecimalType': return 'Decimal';
      case 'OptionType': return `Option<${this.typeToString((type as any).elementType)}>`;
      case 'ResultType': return `Result<${this.typeToString((type as any).okType)}, ${this.typeToString((type as any).errorType)}>`;
      case 'ListType': return `List<${this.typeToString((type as any).elementType)}>`;
      case 'MapType': return `Map<${this.typeToString((type as any).keyType)}, ${this.typeToString((type as any).valueType)}>`;
      case 'FunctionType': return 'Function';
      case 'UnitType': return 'Unit';
      case 'NeverType': return 'Never';
      case 'JsonType': return 'Json';
      default: return type.kind;
    }
  }
}

// Convenience function
export function generateCode(program: Program): string {
  const generator = new CodeGenerator();
  return generator.generateProgram(program);
}
