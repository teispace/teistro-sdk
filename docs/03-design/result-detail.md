# Brief and full answers

Status: `built`, 2026-11-08. Consumer request A2-2
(`docs/consumers/addendum-3.md`): "`detail: brief | full` on results,
with token sizes measured and documented". Every record entry point
(`crates/sdk/src/records.rs`) answers at either detail
(`Record::answer_at`), each record declares what its brief keeps
(`crates/sdk/src/records/detail.rs`), the agent server's record tools take
`detail` and answer brief by default, and the sizes are measured on a
generated page, [`result-detail-measured.md`](result-detail-measured.md),
held by `check-detail-tokens`. Every figure below is that page's; this
page carries none of its own.

## The question

An answer is written for a program: every field a consumer might read,
the hashes that make it verifiable, the provenance that reproduces it.
An agent reads it as text in a context window, where each token is paid
for and crowds out the conversation. The dossier (`dossier.md`) answered
this for a chart document by writing it differently. A record's answer
is answered here by writing **less of it**: the fields a reader needs to
answer a question, leaving out what only a program checking or caching
the answer reads.

## What others do

- **Anthropic's tool-writing guidance** (*Writing effective tools for
  agents*, 2025) recommends exactly this switch: a `response_format`
  argument whose `concise` value leaves out the identifiers a follow-up
  call would need and whose `detailed` value keeps them, and reports the
  concise form at about a third of the tokens. It also warns against
  low-signal technical identifiers (UUIDs, MIME types) in what an agent
  reads.
- **Google's API design guide** (AIP-157, partial responses) offers two
  mechanisms: a field mask the caller writes, and a **view** enum
  (`BASIC`, `FULL`) whose members the service defines. It prefers views
  where the useful subsets are few and known, because the service keeps
  them coherent and the caller need not know the schema.
- **JSON:API sparse fieldsets** (`fields[type]=a,b`) and **GraphQL
  selection sets** let the caller name each field. Both sit outside the
  resource body: a query parameter in one, the query document in the
  other, never a field of the resource itself.
- **The Model Context Protocol** lets a tool declare an `outputSchema`,
  and a server that declares one must answer structured content that
  conforms to it. A narrower answer therefore needs a schema it holds
  to, and the full answer must still hold to the same one.
- **OpenAI's function-calling guidance** asks for small, specific
  outputs and for tools that return what the model needs rather than
  raw records.

## Decisions

### D1. `detail` lives in the call's envelope, not in each request

A record's request names **what is computed**: its hash is the
provenance's `input_hash`, and the answer key is taken from it
(`serial-and-the-envelope.md`, "The answer key"). A brief answer is the
same computation written shorter. Were `detail` a field of every request,
a brief and a full answer would carry two input hashes and two answer
keys for one computation, every one of the fifty-odd request types in
every binding would grow a field, and every deep example and fuzz seed
with it. So `detail` sits where JSON:API and GraphQL put their
selection, beside the record and outside it: an argument of
`Record::answer_at`, and in the agent server one of a record tool's
arguments beside `request`, `profile`, `settings` and `locale`, which are
already the call's envelope.

### D2. A brief is a declared view, never a caller's field mask

Each record declares its brief beside its entry in the records table, as
`Brief::Keeps(&[…])`, an explicit list of field paths (`days.date`
keeps each day's date), or `Brief::Whole`, where nothing in the value is
beside what a reader needs. Nothing is inferred from the answer's
shape: a field added to an answer is left out of the brief until
someone adds it to the list. This is AIP-157's view rather than its
field mask, for three reasons measured or read here:

- a model choosing fields needs the answer's schema, which the lean
  tool list leaves out on purpose (`mcp-server.md` P1); a mask would put
  it back or have the model guess names;
- a mask costs tokens on every call and is one more argument a model can
  spell wrong; two named views cost one enum;
- the useful subsets are few: a reader asking a record wants its
  answer, and only a program wants the hashes, the per-item seals and
  the fields that restate the request.

What a brief keeps: the answer itself (dates and instants, names and
keys, verdicts, the numbers asked for), and what a follow-up call needs
(a key another record takes). What it leaves out: content hashes and
per-item seals, which a reader cannot check without the full answer;
fields that echo the request; and large supporting tables an answer
carries beside its result. The page lists every large answer's fields
with their tokens and marks what each brief keeps, so a list is chosen
against its cost.

A caller's own field mask is not built. It can come later as a third
mechanism beside the two views without changing them.

### D3. A brief is a projection, held to the full schema

A brief answer is the full answer with fields removed and nothing else:
every value it keeps is the full answer's own, unchanged, and nothing
is renamed or summarised (`Brief::of`). So:

- a caller holding a full answer derives the brief without asking again
  (`Record::brief_of`);
- the brief's schema is the full answer's with `required` loosened along
  the brief's paths (`Brief::narrow`). Only `required` changes, so a full
  answer holds to it too. A definition a path reaches is inlined as a
  copy and narrowed there, so a type every tool shares keeps one
  definition (`every_schema_definition_names_its_own_type`);
- the agent server lists that one schema as each tool's `outputSchema`
  (under `--schemas full`), and a test holds every example's brief
  answer and full answer to it with a JSON Schema validator
  (`every_example_and_its_answer_hold_to_the_tools_schemas`).

The measured page holds the declarations to their schemas: every path
names a property of its record's answer schema, and no brief leaves out
the tag a `oneOf` branch is told apart by (a required property whose
schema is one constant), without which an instance could match two
branches or none. A path may run into a value its schema leaves open, a
section an answer carries as any JSON (`almanac.days`'s `muhurta` and
`sky`); the page lists each such path, since nothing below it is
checked. The claims are ones the page refuses to write when they fail,
so `check-detail-tokens` goes red.

### D4. The provenance and the answer key

In Rust the provenance stays whole under either detail
(`Answered::provenance`), so a caller keeps everything it had. Written
as the envelope an agent reads (`Answered::envelope`), a brief answer's
provenance keeps only `Brief::PROVENANCE`: the profile, the confidence,
the fallbacks used, the warnings and the `answer_key`; and the envelope
carries `detail: "brief"` beside it.

The answer key is unchanged by the detail: it is taken from the
request, the settings, the engine and its data, never from the value
(`serial-and-the-envelope.md`). It therefore **identifies the full
answer**, and the envelope's `detail` says that the value is the brief of
it. The content hash is the full value's, which a brief reader cannot
check, so the brief provenance leaves it out rather than carry a hash
that does not match the value beside it. A cache keeps the full answer
under its key and serves either detail from it; it never stores a brief
answer under the key alone.

An answer with no provenance (a numerology profile, a date converted)
is written as its value alone at either detail, as it was.

### D5. How each surface exposes it

- **Rust**: `teistro::records::{Detail, Brief}`, `Record::answer_at`,
  `Record::answer_against_at`, `Record::brief`, `Record::brief_of`,
  `Record::answer_schema_at` and `Answered::envelope`. `Detail` defaults
  to `Full`: an application computing a record reads the whole answer as
  before.
- **The agent server**: each record tool takes `detail` (`brief` or
  `full`), refused by `arguments.detail` when it is neither; the server
  answers **brief** when a call names none, and `--answers full` (or
  `Server::with_answers`) changes that for an operator whose clients
  validate or store full answers. Brief is the default because an
  agent's context is the resource an answer spends, and the measured
  page shows what each record's brief saves on its example; a model that
  needs a hash or a full table asks for `full` by name. An engine's own
  operation tool answers at the server's detail, since its arguments are
  the engine's and carry no `detail`.
- **The bindings** (C, Node, Python, Dart, Java) keep their typed
  answers whole. A typed answer is read by field name in code, where a
  field left out costs nothing to skip and a partial type would double
  every answer class; tokens matter where an answer becomes a model's
  text, which is the agent server's place. A binding that hands answers
  to a model it drives itself has no SDK call for the brief yet; it is
  listed under *Open*.

## Open

- A binding's brief: one generic boundary function answering any record
  by name at a detail, as canonical JSON, read by every binding as an
  untyped JSON value (the agent server's door, offered to applications
  that drive a model themselves). Not built; nothing asks for it yet.
- A caller's own field mask beside the two views (D2).
- The tokens are the dependency-free approximation's (C1165), a ruler
  for comparing the two details, not a model's bill.
