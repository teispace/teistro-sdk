# Session continuity

Status: `draft`, 2026-09-04. This project is long and worked on across many
sessions, some of which lose their context. These rules make sure nothing
is lost.

## The three files

| file | role | updated |
|---|---|---|
| `docs/STATUS.md` | done, now, next, session log | at the start and end of every session; whenever a milestone moves |
| `docs/QUESTIONS.md` | open questions with recommendations; decisions log | whenever a question is raised or answered |
| `docs/08-decisions/` | accepted decisions as ADRs | when a question is decided |

## At the start of a session

1. Read `STATUS.md` (where are we), then `QUESTIONS.md` (what is
   undecided), then the pages the "Now" section names.
2. Do not re-open a decided question; a new record supersedes an old one
   only with new evidence.

## During a session

- Write research and design into the docs as it is produced, not at the
  end.
- When a page is changed materially, update its status line and date.
- When a number is produced (a measurement, a count), put it where it will
  be gated, not only in prose.

## At the end of a session

1. Update `STATUS.md`: move items between done, now and next; add a
   session-log row with what happened and what the next step is.
2. Update `QUESTIONS.md` with anything raised.
3. Update the assistant's memory notes (outside the repository) with the
   pointer to `STATUS.md` and any non-obvious fact the next session needs.

## What "in track" means

Anyone, or any session, can answer "what did we do, where are we, what are
we doing, what comes next" from `STATUS.md` alone in under five minutes.
If that stops being true, fixing it is the first task of the next session.
