---
name: Feature request
about: Propose a new API, an ergonomic improvement, or an additional config source.
title: "[feature] <short summary>"
labels: ["enhancement", "triage"]
assignees: []
---

<!--
Before opening this request, please skim CONTRIBUTING.md — specifically
the "Invariants that must not regress" section. Proposals that require
relaxing one of the invariants (adding a free function to the public
API, making the missing JSON file a hard error, adding `unwrap()` in
production code) will almost always be answered with "a new type of
loader, not a weakened existing one". That is still a valid discussion
— just be aware of the bar.
-->

## Problem

<!--
Describe the concrete use-case first, not the solution. What are you
trying to build, and which friction with the current crate stopped you?
"I need to load a YAML file alongside the JSON one" is a problem.
"Add a `with_yaml_file` method" is a solution — save it for the next
section.
-->

## Proposed change

<!--
If you already have an idea of the shape, sketch it. Signatures welcome.
If you don't, that's fine — say so and leave the design to the discussion.
-->

```rust
// Optional: what the new API might look like.
```

## Alternatives considered

<!--
What did you try first? Why didn't it work?
Examples: a wrapper type in your own crate, a helper function, a
different library, a manual workaround. This section is the one that
most often changes the outcome of the discussion.
-->

## Impact on the public contract

<!-- Tick every box that applies. -->

- [ ] Adds a new item (method / trait impl / type) — no existing API changes
- [ ] Adds a new Cargo feature (gated, off by default)
- [ ] Changes the signature of an existing public item (breaking)
- [ ] Relaxes one of the load-bearing invariants (JSON-only, non-blocking file, env-wins priority)
- [ ] Adds a new dependency — please name it:
- [ ] None of the above / not sure yet

## MSRV implication

<!-- Would this require a newer Rust version than the current MSRV (1.85)? -->

- [ ] No
- [ ] Yes — minimum required version:
- [ ] Not sure

## Additional context

<!-- Prior art in other crates, links to RFCs or issues, screenshots of compiler errors, etc. -->
