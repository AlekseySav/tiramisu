# tiramusu

![yet another tmux session multiplexer](images/cast.gif)

## Installation

- Download and install the [latest release](https://github.com/AlekseySav/tiramisu/releases)

- Or install via ***homebrew***:
```
$ brew install alekseysav/tiramisu/tiramisu
```

## Configuration

Basic config

```toml
[[session]]
  root = "$HOME/src/(*)"
  name = "src/$1"
  [[session.window]]
    name = "$1"
    command = "nvim ."
  [[session.window]]
    name = "shell"
```
