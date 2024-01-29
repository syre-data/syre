# Syre Developer Guide

Welcome to the Syre developer guide.

*How should I name my branch? What should I put in my commit message?*

These are the kind of questions this guide aims to answer.

## Table of contents

- [Developer Guide](#developer-guide)
  - [Table of contents](#table-of-contents)
  - [TODO](#todo)
  - [On branches](#on-branches)
  - [On commits](#on-commits)

## TODO

We use the following tags to mark TODOs in comments:

`TODO[l] Message.`

Priority is denoted by the letter in the brackets l, m, h, which
respectively stand for low, medium and high priority.

For example:

```rs
pub fn main() {
    //TODO[l] Message should not be hardcoded.
    println!("Hello, world!");
}
```

## On branches

We use the branch name convention `type/description` (e.g. `feat/updated_ui`, `fix/compile-error`).

## On commits

If a commit fixes an issue, it should be mentioned in the commit message. For example:

`Fixes Issue #354`
