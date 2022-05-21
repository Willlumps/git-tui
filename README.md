# gitbuddy
gitbuddy is a simple terminal UI written in Rust using [tui-rs](https://github.com/fdehau/tui-rs) to perform basic git commands inspired by [lazygit](https://github.com/jesseduffield/lazygit). gitbuddy is still very much a work in progress and can only currently do the simplest of tasks/commands.

# Keymaps
## Basic Navigation
* `C-q`:  Quit the application
* `1-4`:  Navigate the four main component panes
* `j`: Scroll down through a list or window (`C-j` for scrolling through branches)
* `k`: Scroll up through a list or window (`C-k` for scrolling through branches)

## Files Component
* `c`: Commit
* `s`: Stage file under cursor
* `u`: Unstage file under cursor
* `a`: Stage all files
* `A`: Unstage all files
* `p`: Push to remote

## Commit Window
* `Esc`: Cancel commit
* `Enter`: Commit with entered message
