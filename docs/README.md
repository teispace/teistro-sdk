# The documentation map

Everything about Teistro SDK lives here, and it is organised so that a reader
finds the one page that answers their question rather than reading in order.
Numbered directories read top to bottom as the project itself progresses: the
vision comes first, research grounds it, architecture shapes it, design and
implementation realise it, testing and CI/CD guard it, and the roadmap orders
it. Decisions and guidelines sit at the end because they are referenced from
everywhere.

Three files are living documents and are updated in every working session:

| file | what it is |
|---|---|
| [`STATUS.md`](STATUS.md) | where the work stands: done, in progress, next, and a session log |
| [`QUESTIONS.md`](QUESTIONS.md) | open questions for the maintainer, each with options and a recommendation, and the decisions taken |
| [`08-decisions/`](08-decisions/README.md) | architecture decision records; a question becomes an ADR when it is decided |

## Directories

| directory | answers | status |
|---|---|---|
| [`00-vision/`](00-vision/) | what Teistro SDK is, the principles it holds to, and the scope of the first version | drafted |
| [`01-research/`](01-research/README.md) | the astrology feature universe across traditions, the competitive landscape, the baseline engine, and the platform research (language, bindings, ephemeris abstraction, localization, calendars, modularity, performance, security, testing, docs, CI/CD, licensing) | drafted |
| [`02-architecture/`](02-architecture/00-overview.md) | the shape: layers, modules, ports, data model, API conventions, bindings, extensibility, performance and security architecture | draft, pending decisions |
| [`03-design/`](03-design/README.md) | per-module detailed designs | planned |
| [`04-implementation/`](04-implementation/README.md) | repository layout, coding standards, build system | planned |
| [`05-testing/`](05-testing/README.md) | the quality bar (accepted, binding) and the test and conformance plans | quality bar accepted; plans planned |
| [`06-cicd/`](06-cicd/README.md) | pipelines, release engineering, packaging per binding | planned |
| [`07-roadmap/`](07-roadmap/00-roadmap.md) | phases, milestones, deliverables and exit criteria | drafted |
| [`08-decisions/`](08-decisions/README.md) | ADRs | proposed |
| [`09-guidelines/`](09-guidelines/README.md) | how to write docs here, add a language, a module, a calendar; commit conventions; session continuity | drafted |

## Conventions for these documents

- Every page states its **status** at the top: `research`, `draft`,
  `proposed`, `accepted`, `planned`, `generated`. A page that claims a number
  says where the number came from.
- British spelling, as in Teimeris (`behaviour`, `optimise`, `licence` the
  noun). See `09-guidelines/01-docs-style.md`.
- A page is either derived from a source of truth or checked against one.
  Until the code exists, the source of truth for feature claims is the
  research pages, which cite what they were read from.
- Open questions are never buried in prose. They go to `QUESTIONS.md` with a
  recommendation, and the page that raised them links to the question.
