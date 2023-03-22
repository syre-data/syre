# Rust

## Linting and formatting
Use [`rust-analyzer`](https://rust-analyzer.github.io/) for all linting and formatting.

## Functions
+ Function defintiions should be bottom dependent. this means that calling functions should always be defined above their dependent functions as best as possible.
+ **Return types:** Return types should be as explicit as possible.
  e.g. 
  .. codeblock:: rust

    struct MyStruct { ... }
    impl MyStruct {
      fn new() -> MyStruct { ... }  // returns `MyStruct` instead of `Self`
    }

### Documentation
Function documentation should start with a brief description of what the function
does, any side effects it causes, and its return value.
This brief description can then be followed by any of the following sections, in order.
Each of these sections should be an `h1`.
#. `Arguments`: Ordered list describing the arguments the function receives.
    `#. \`argument_name\`: Description of argument.`
#. `Returns`: Description of the return value.
#. `Errors`: Unordered list of errors that the function may return.
    `+ \[\`ErrorType\`\]: Description of what can cause this error to be returned.
#. `Side effects`: Side effects that the function causes.
#. `See also`: Links to related resources.

### `Unwrap`ping
Prefer `expect` over `unwrap` in order to provide an error message.
This helps track down bugs more easily, especially in client-side code.

## Tuples
Use an ordered list in the documentation to describe each field.

## Enums
Use documentation on each variant to decribe its meaning.

## Comments

### Tags
Comments can include tags for easy searching. All tags are preceeded by an ampersand (`@`) and
separated from their associated comment by a colon (`:`).

+ `@todo[<priority>]`: Indicates a pending task. Priorities indicate the urgency
of the task and can be between 0 and 5. Lower values indicate a higher urgency.
+ `@remove`: Indicates the associated code should be removed after testing.
This is useful for tracking intermediate code changes (such as print statements)
during development.
+ `@note`: Used to explain a subtlety of the following code block.
Can be used to explain why a certain design choice was made to future developers.
