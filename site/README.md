# The documentation site

Status: `built`, 2026-09-06. The decision is
[ADR-0012](../docs/08-decisions/adr-0012-docs-site-fumadocs.md); the
pipeline is [`06-cicd/05-docs-deploy.md`](../docs/06-cicd/05-docs-deploy.md).

Fumadocs on Next.js, exported as static files. `npm run dev` serves it,
`npm run build` writes `out/`, and `cargo xtask check-site` does the
build and checks that every generated page was rendered.

## What is written by hand and what is generated

| path | what it is | written by |
|---|---|---|
| `content/docs/index.mdx`, `install.mdx` | the guides | by hand |
| `content/docs/reference/**` | one page per entry point of the C ABI, plus the structs and the enums | `cargo xtask gen ffi` |
| `app/`, `components/`, `lib/` | the application: layouts, the search dialog, the MDX components | by hand, from the Fumadocs template |

Never edit anything under `content/docs/reference/`: change the boundary
crates and run `cargo xtask gen ffi`. `cargo xtask check-ffi`, which runs
on every push, regenerates the whole tree in memory and fails on any
difference — and on any file in that tree the generator does not produce,
so a page for a removed entry point cannot linger.

## The components a page may use

`components/mdx.tsx` registers `Tabs`, `Tab` and `Callout` globally, so a
generated page is Markdown with tabs in it and carries no imports. A page
that needs a component the list does not have gets it added there rather
than imported thirty-six times.

## Writing a guide

Add an `.mdx` file under `content/docs/` and list it in the `meta.json`
beside it. Frontmatter is `title` and `description`. Prose is British
spelling, like the rest of the project, and every example is one that
runs: the four facts on the home page are the four the C smoke test, the
Node consumer and the Dart consumer all print.

## Building

```sh
npm ci          # the lock file is checked in
npm run dev     # http://localhost:3000
npm run build   # writes out/
```

The site is deployed from a tag by
[`docs.yml`](../.github/workflows/docs.yml), so what is published is the
site of a release rather than of `main`.
