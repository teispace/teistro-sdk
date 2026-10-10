# teistro-mcp

The Teistro SDK as tools an agent calls: a Model Context Protocol server
over stdio (`docs/03-design/mcp-server.md`). A model asked for a
nakshatra answers from memory with no ayanamsha named; a tool answers
with the settings hash, the input hash and every convention applied.

```sh
teistro-mcp                       # the built-in ephemeris
teistro-mcp --ephemeris SURYA_SIDDHANTA
```

A host starts it as a child process and speaks either revision:

- **2026-07-28**, stateless: each request carries
  `_meta["io.modelcontextprotocol/protocolVersion"]`, and
  `server/discover` answers the revisions, the capabilities and the
  server's identity;
- **2025-11-25**, after `initialize`, for the rest of the process.

Each record tool takes `{request, profile, settings, locale}`. `request`
is the record its binding sends, refused by the same reader under the
same field names; `settings` is a patch over the profile, and
`settings.describe` answers its JSON Schema with every knob's
documentation. An answer is `{value, provenance}`. A refusal is a tool
error whose structured content is the SDK's error record, naming the
field, the range and a hint.

`chart.found` takes a whole chart request: the births (`instant` or
`instants`), the place and clock, the sections as `true` flags, and a
record for each table read off the charts, under the names every
binding writes (`kp`, `fortitudes`, `varsha`, …). It answers the chart
documents as `charts` and each table with a row a chart.

`almanac.days` takes a range of days: `first` and optional `last`, the
place and clock, and beside them a `muhurta` search, a `festivals`
reckoning, and `true` flags for the lunar `years`, the `eclipses` and
each day's `nepalSambat` date. It answers every day's panchanga, the
days' own hashes, and each section asked as its own
`{value, provenance}`.

`time.resolve`, `time.civil`, `time.convert` and `calendar.convert`
resolve a civil time in a zone to an instant, read an instant on a
zone's clock, carry an instant between UT1, TT and UTC with what was
applied, and write a date in another calendar with its weekday.

Nothing reaches the file system or the network: the ephemeris is chosen
on the command line and never by a tool argument.
