Here are some best practices about coding.

## Comments

- Comments should not exceed a few lines per code block.
- For functions or classes, it's okay to have more comments.
- If the name of the struct, function, or variable is clear, no need for comments.
- For longer code blocks, exceeding 5 to 7 lines of code, it's good to split them into 3 to 5 lines of code and have clear components and blank lines.
- It's nice to have some module comments at the top of the main module, or at least somewhere in the module tree.
- Region comments are a good way to split groups of functions, types, and such. Make sure to use them in large code files (> 100 lines of code).

## Coding

- All errors should return the crate or module type Result alias which maps to the corresponding moedule error. This allows to normalize the error taxonomy while providing flexibility.
- Modules should be per area of concern, and not try to do too many things. Otherwise, split them into submodules.
- Typically, files should not exceed 500 lines of code (excluding testing). This is not a hard rule, but splitting things early helps with further refactoring.
- Make sure that IDs that are strings are wrapped into their own types.

