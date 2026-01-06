# Channels

This document outlines the design and plan for implementing Go-like channels in Pyro.

## Goal

Provide a mechanism for safe, concurrent communication between tasks (`go` routines), similar to Golang's channels.

## Design

### Syntax

We will introduce the `<-` operator, used for both sending and receiving.

-   **Send (Statement)**: `channel <- value`
    -   Sends `value` into `channel`.
    -   This is a statement, not an expression (cannot be used inside other expressions).
-   **Receive (Expression)**: `<- channel`
    -   Receives a value from `channel`.
    -   Blocks until a value is available.
    -   Can be used in expressions: `x = <- ch`.

### Semantics

-   **Channel Type**: A new `Value` variant `Channel`.
    -   Internally wraps a multi-producer, multi-consumer (MPMC) async channel.
    -   We will leverage `async-channel`.
-   **Creation**: `chan(capacity: int)` built-in function.
    -   `capacity > 0`: Buffered channel.
    -   `capacity = 0`: Defaults to capacity 1 (buffered) for now as `async-channel` requires size.

### Implementation Plan

#### 1. Dependencies
-   Add `async-channel` to `Cargo.toml`.

#### 2. Lexer & Parser
-   **Lexer**: Update to recognize `<-` as `Token::ArrowLeft`.
-   **AST**:
    -   Add `Expr::Receive(Box<Expr>)`.
    -   Add `Stmt::Send(Expr, Expr)`.
-   **Parser**:
    -   Update `parse_statement` to handle `expr <- value` syntax (similar to assignment).
    -   Update `parse_unary` (refactor `parse_atom`) to handle prefix `<-` for `Expr::Receive`.

#### 3. Interpreter
-   **Value**: Add `Channel(Sender<Value>, Receiver<Value>)`.
-   **Built-in**: Add `chan(capacity)` function to global scope.
-   **Execution**:
    -   `Expr::Receive`: `await` on the channel receiver.
    -   `Stmt::Send`: `await` on the channel sender.

#### 4. Verification
-   **Tests**:
    -   Simple send/recv within one thread.
    -   Ping-pong test between two `go` routines.
    -   Buffer capacity behavior.

## Task List

-   [x] Add `async-channel` dependency
-   [x] Update Lexer (`Token::ArrowLeft`)
-   [x] Update AST (`Expr::Receive`, `Stmt::Send`)
-   [x] Update Parser (Send statement, Receive expression)
-   [x] Update Interpreter (Channel primitives, `chan()` builtin)
-   [x] Write documentation
-   [x] Create comprehensive tests
