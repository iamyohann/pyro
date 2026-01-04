# Pyro Package Manager

Pyro comes with a built-in package manager inspired by Go. It allows you to create projects and manage dependencies using Git repositories.

## Creating a Project

To initialize a new Pyro project:

```bash
pyro mod init my_project
```

This creates a `pyro.mod` file and a `src` directory with a `main.pyro` file.

## Installing Dependencies

To install a dependency from a Git repository:

```bash
pyro get github.com/username/repo
```

This clones the repository into `~/.pyro/pkg/github.com/username/repo`.

## Importing Packages

You can import local files or installed packages using the `import` statement:

```python
# Import a local file relative to the current file
import "lib.pyro"

# Import an installed package
import "github.com/username/repo/src/lib.pyro"

lib_function()
```

## Module Resolution

Pyro resolves imports in the following order:
1. Relative to the current file.
2. Inside `~/.pyro/pkg/`.

This allows for a simple and effective way to manage dependencies without complex project-local `node_modules` folders.
